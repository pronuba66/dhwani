use std::thread::JoinHandle;

use crate::{Error, buffer::RingBufReceiver, controller::CtrlSender};

/// The controller ring buffer recevier; which is a wrapper on top of
///[`crate::buffer::RingBufReceiver`] and implementing the [`Drop`] trait.
pub struct CtrlRingBufReceiver {
    ctrl_sender: CtrlSender,
    ring_buf_receiver: RingBufReceiver<f32>,
    handle: Option<JoinHandle<()>>,
}

impl Drop for CtrlRingBufReceiver {
    fn drop(&mut self) {
        let _ = self.ctrl_sender.quit();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl CtrlRingBufReceiver {
    pub fn new(
        ctrl_sender: CtrlSender,
        ring_buf_receiver: RingBufReceiver<f32>,
        handle: Option<JoinHandle<()>>,
    ) -> Self {
        Self {
            ctrl_sender,
            ring_buf_receiver,
            handle,
        }
    }

    /// Get the audio buffer. The buffer is in interleaved format.
    ///
    /// Returns the [`Result`] of the total number of bytes available.
    ///
    /// # Errors
    /// See [`crate::buffer::RingBufReceiver::get`]
    pub fn get(&self, output: &mut [f32]) -> Result<usize, Error> {
        self.ring_buf_receiver.get(output)
    }
}
