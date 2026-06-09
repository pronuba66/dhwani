//! Single produced, single consumer (SPSC) ring buffer

use std::{
    cell::{Cell, UnsafeCell},
    marker::PhantomData,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use crate::Error;

/// Ring buffer producer
pub struct RingBufSender<T>
where
    T: Default + Copy,
{
    rb: Arc<RingBuf<T>>,
    _not_sync: PhantomData<Cell<()>>,
}

impl<T> RingBufSender<T>
where
    T: Default + Copy,
{
    /// Attempts to push elements from `input` into the ring buffer.
    ///
    /// This function will try to write as many elements as possible into the buffer.
    /// If there is insufficient space, it will continue attempting to write for up to
    /// `timeout` duration.
    ///
    /// Returns the number of elements successfully written. This may be less than
    /// `input.len()` if the buffer does not have enough capacity within the timeout.
    ///
    /// # Arguments
    /// * `input` - Slice of elements to be written into the buffer
    /// * `timeout` - Maximum total duration to wait for available space
    ///
    /// # Panics
    /// Panics if the underlying lock is poisoned.
    ///
    pub fn push_timeout(&self, input: &[T], timeout: Duration) -> usize {
        self.rb.push_timeout(input, timeout)
    }
}

/// Ring buffer consumer
pub struct RingBufReceiver<T>
where
    T: Default + Copy,
{
    rb: Arc<RingBuf<T>>,
    _not_sync: PhantomData<Cell<()>>,
}

impl<T> RingBufReceiver<T>
where
    T: Default + Copy,
{
    /// Attempts to read elements from the ring buffer into `output`.
    ///
    /// Copies up to `output.len()` elements into `output`, depending on how many
    /// elements are currently available in the buffer.
    ///
    /// Returns `Ok(n)` where `n` is the number of elements actually written to
    /// `output`. This may be less than `output.len()` if the buffer does not
    /// contain enough elements. Returns `Ok(0)` if the buffer is empty.
    ///
    /// # Arguments
    /// * `output` - Mutable slice to store the retrieved elements
    ///
    /// # Errors
    /// Returns an error if `output.len()` is greater than buffer size or
    /// if `output.len()` is not a multiple of number of channels
    ///
    pub fn get(&self, output: &mut [T]) -> Result<usize, Error> {
        self.rb.get(output)
    }
}

pub struct RingBuf<T>
where
    T: Default + Copy,
{
    n_channels: usize,
    left: AtomicUsize,
    right: AtomicUsize,
    buffer_size: usize,
    // Use UnsafeCell instead of RefCell for removing runtime borrow checks
    #[allow(clippy::non_send_fields_in_send_ty)]
    buffer: UnsafeCell<Vec<T>>,
    // for blocking
    wait_lock: Mutex<()>,
    wait_cvar: Condvar,
}

unsafe impl<T> Send for RingBuf<T> where T: Default + Copy {}
unsafe impl<T> Sync for RingBuf<T> where T: Default + Copy {}

impl<T> RingBuf<T>
where
    T: Default + Copy,
{
    /// Creates a ring buffer and returns the producer and consumer
    ///
    /// # Arguments
    /// * `n_channels` - Number of channels
    /// * `buffer_size_per_ch` - The buffer size per channel
    ///
    #[must_use]
    pub fn split(
        n_channels: usize,
        buffer_size_per_ch: usize,
    ) -> (RingBufSender<T>, RingBufReceiver<T>) {
        let buffer_size = buffer_size_per_ch * n_channels;
        let rb = Arc::new(Self {
            n_channels,
            left: AtomicUsize::new(buffer_size),
            right: AtomicUsize::new(0usize),
            buffer_size,
            buffer: UnsafeCell::new(vec![T::default(); buffer_size]),
            wait_lock: Mutex::new(()),
            wait_cvar: Condvar::new(),
        });
        (
            RingBufSender {
                rb: rb.clone(),
                _not_sync: PhantomData,
            },
            RingBufReceiver {
                rb,
                _not_sync: PhantomData,
            },
        )
    }

    /// Returns the number of elements in the ring buffer
    fn len(&self) -> usize {
        let left = self.left.load(Ordering::Acquire);
        if left == self.buffer_size {
            return 0;
        }
        let right = self.right.load(Ordering::Acquire);
        if right > left {
            right - left
        } else {
            self.buffer_size - (left - right)
        }
    }

    ///
    /// # Panics
    /// Panics if underlying lock is poisoned
    ///
    fn push_timeout(&self, input: &[T], timeout: Duration) -> usize {
        debug_assert!(
            input.len().is_multiple_of(self.n_channels),
            "input length must be a multiple of n_channels"
        );
        let start = Instant::now();
        let mut written = 0;
        while written < input.len() {
            let available = self.buffer_size - self.len();
            let available = (available / self.n_channels) * self.n_channels;
            if available < self.n_channels {
                let elapsed = start.elapsed();
                if elapsed >= timeout {
                    break;
                }
                #[allow(clippy::unchecked_time_subtraction)]
                let remaining = timeout - elapsed;
                // Wait for the consumer to signal that it has read data
                let _ = self
                    .wait_cvar
                    .wait_timeout(self.wait_lock.lock().unwrap(), remaining)
                    .unwrap();
                continue;
            }
            let len = self.buffer_size - available;
            let chunk = (input.len() - written).min(available);
            let right = self.right.load(Ordering::Acquire);
            unsafe {
                let buffer = (*self.buffer.get()).as_mut_slice();
                for i in 0..chunk {
                    buffer[(right + i) % self.buffer_size] = input[written + i];
                }
            }
            debug_assert!(len + chunk <= self.buffer_size);
            self.right
                .store((right + chunk) % self.buffer_size, Ordering::Release);
            let _ = self.left.compare_exchange(
                self.buffer_size,
                right,
                Ordering::AcqRel,
                Ordering::Acquire,
            );
            written += chunk;
        }
        written
    }

    ///
    /// # Errors
    /// Return `Err` if output len is invalid. Output length must be <= `self.buffer_size`
    /// and multiple of `self.n_channels`
    ///
    fn get(&self, output: &mut [T]) -> Result<usize, Error> {
        if output.len() > self.buffer_size {
            return Err(Error::msg(format!(
                "Buffer must be a less than {}",
                self.buffer_size
            )));
        } else if !output.len().is_multiple_of(self.n_channels) {
            return Err(Error::msg(format!(
                "Buffer must be a multiple on number of channels ({})",
                self.n_channels
            )));
        }
        let available = self.len();
        let chunk = available.min(output.len());
        if chunk == 0 {
            return Ok(chunk);
        }
        let left = self.left.load(Ordering::Acquire);
        unsafe {
            let buffer = (*self.buffer.get()).as_slice();
            for i in 0..chunk {
                output[i] = buffer[(left + i) % self.buffer_size];
            }
        }
        if available == chunk {
            // To mark 0 length
            self.left.store(self.buffer_size, Ordering::Release);
        } else {
            self.left
                .store((left + chunk) % self.buffer_size, Ordering::Release);
        }
        self.wait_cvar.notify_one();
        Ok(chunk)
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    #[test]
    fn ring_buf_seq_test() {
        // Sequential test
        const N_ITER: usize = u16::MAX as usize;
        let (sender, receiver) = RingBuf::<f32>::split(2, 16);
        let input: Vec<f32> = (0..32).map(|i| i as f32).collect();
        let mut output = [0f32; 32];
        for i in 0..N_ITER {
            let written = sender.push_timeout(input.as_slice(), Duration::from_millis(0));
            assert!(written == input.len());
            let count = receiver.get(output.as_mut_slice()).unwrap();
            if count != input.len() {
                eprintln!("{}. count != input.len(), {} != {}", i, count, input.len());
                panic!("Failed");
            }
            for i in 0..count {
                if output[i] != i as f32 {
                    eprintln!("Invalid sequence, {} != {}", output[i], i as f32);
                    dbg!(output);
                    panic!("Failed");
                }
            }
        }
    }

    #[test]
    fn ring_buf_spsc_test() {
        // SPSC parallel test
        const TOTAL_SIZE: usize = 32 * 1024;
        const INPUT_SIZE: usize = 32;
        const INPUT_N_ITER: usize = TOTAL_SIZE / INPUT_SIZE;
        const OUTPUT_SIZE: usize = 64;
        const OUTPUT_N_ITER: usize = TOTAL_SIZE / OUTPUT_SIZE;
        let (sender, receiver) = RingBuf::<f32>::split(1, OUTPUT_SIZE * 4);
        let handle = {
            thread::spawn(move || {
                let mut input = [0f32; INPUT_SIZE];
                let mut n = 0;
                for _ in 0..INPUT_N_ITER {
                    for (j, i) in input.iter_mut().enumerate() {
                        *i = (n + j) as f32;
                    }
                    let written = sender.push_timeout(input.as_slice(), Duration::from_secs(1));
                    assert!(written == INPUT_SIZE, "Failed to push");
                    n += written;
                }
                println!("Total written {n}");
            })
        };
        let mut output = [0f32; OUTPUT_SIZE];
        let mut n = 0;
        for _ in 0..OUTPUT_N_ITER {
            match receiver.get(output.as_mut_slice()) {
                Ok(count) => {
                    for val in &output.as_slice()[0..count] {
                        assert!(*val == n as f32, "Failed");
                        n += 1;
                    }
                }
                Err(e) => {
                    eprintln!("{e:?}");
                    panic!("Failed");
                }
            }
            thread::sleep(Duration::from_millis(1));
        }
        println!("Total read {n}");
        assert!(handle.join().is_ok());
    }
}
