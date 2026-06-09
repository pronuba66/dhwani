mod ctrl_buffer;
mod ctrl_receiver;
mod ctrl_ring_buf_receiver;
mod ctrl_sender;

use std::{thread, time::Duration};

use crate::{
    Error, Processor,
    buffer::RingBuf,
    controller::{
        ctrl_buffer::CtrlBuffer,
        ctrl_receiver::{CtrlPendingReq, CtrlReceiver, CtrlRsp},
    },
};

pub use ctrl_ring_buf_receiver::CtrlRingBufReceiver;
pub use ctrl_sender::CtrlSender;

/// To start controller thread and return the control sender and ring buffer receiver
pub fn start_controller(
    sample_rate: u32,
    n_channels: u16,
    buffer_size: usize,
) -> (CtrlSender, CtrlRingBufReceiver) {
    let ctrl_buffer = CtrlBuffer::default();
    let (ctrl_sender, ctrl_receiver) = ctrl_buffer.split();
    let (ring_buf_sender, ring_buf_receiver) =
        RingBuf::<f32>::split(usize::from(n_channels), buffer_size);
    let handle = thread::spawn(move || {
        let mut processor = Processor::new(sample_rate, n_channels, buffer_size);
        let mut buffer: Vec<f32> = vec![0f32; usize::from(n_channels) * processor.buffer_size()];
        #[allow(clippy::cast_precision_loss)]
        let timeout =
            Duration::from_secs_f64(buffer.len() as f64 / f64::from(processor.sample_rate()));
        let buffer_len = buffer.len();
        let mut offset = 0usize;
        const PENDING_REQUEST_CAPACITIY: usize = 32usize;
        let mut pending_requests = Vec::<CtrlPendingReq>::with_capacity(PENDING_REQUEST_CAPACITIY);
        let mut pending_requests_temp =
            Vec::<CtrlPendingReq>::with_capacity(PENDING_REQUEST_CAPACITIY);
        /***
         * Too much complexity in below loop. Need revist
         * - First requests
         * - Process the request => supposed to modify [`Processor`]
         * - If the requst creates error, instantly send it to [`CtrlBuffer`]
         * - Else add the response as [`CtrlPendingReq`] in queue
         * - Run [`Processor::process`]
         * - Run through the queue of `CtrlPendingReq`] and if is done, send it to [`CtrlBuffer`]
         */
        let error = 'top_loop: loop {
            loop {
                // IFNO: Lets not hold
                // let req = if processor.is_playing() {
                //     ctrl_receiver
                //         .try_recv()
                //         .ok_or_else(|| Error::msg("No requests".into()))
                // } else {
                //     Ok(ctrl_receiver.recv())
                // };
                let req = ctrl_receiver
                    .try_recv()
                    .ok_or_else(|| Error::msg("No requests".into()));
                if let Ok(req) = req
                    && let Some(pending_req) = CtrlReceiver::process_request(&mut processor, req)
                {
                    pending_requests.push(pending_req);
                }
                if !processor.is_running() {
                    break 'top_loop Error::msg("Processor not running".into());
                }
                if !processor.is_playing() {
                    // Reset offset if not playing
                    offset = 0usize;
                }
                if ctrl_receiver.is_empty() {
                    // If more requests are present, lets process them all
                    // else break for processing
                    break;
                }
            }
            let processed = match processor.process(&mut buffer[offset..]) {
                Ok(processed) => offset + processed,
                Err(e) => break e,
            };
            for pending_request in pending_requests.drain(..) {
                match pending_request.rsp {
                    CtrlRsp::Play(enable) => {
                        if enable == true {
                            pending_request.send();
                        } else {
                            if processor.is_fader_done() {
                                pending_request.send();
                            } else {
                                pending_requests_temp.push(pending_request);
                            }
                        }
                    }
                    _ => {
                        // All these commands are executed at first run
                        // Push it immediately
                        pending_request.send();
                    }
                }
            }
            std::mem::swap(&mut pending_requests, &mut pending_requests_temp);
            let written = ring_buf_sender.push_timeout(&buffer[..processed], timeout);
            buffer.as_mut_slice().copy_within(written.., 0);
            offset = buffer_len - written;
        };
        // Reply to pending requests
        for pending_request in pending_requests.drain(..) {
            pending_request.send_err(error.clone());
        }
    });
    let ctrl_receiver =
        CtrlRingBufReceiver::new(ctrl_sender.clone(), ring_buf_receiver, Some(handle));
    (ctrl_sender, ctrl_receiver)
}

#[cfg(test)]
#[cfg(feature = "controller")]
mod tests {
    use std::{thread, time::Duration};

    use tokio::task::JoinHandle;

    use crate::{
        controller::{ctrl_buffer::CtrlBuffer, ctrl_receiver::CtrlRsp, ctrl_sender::CtrlReq},
        time::{TimeFrom, TimeUnit},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn ctrl_mpsc_test() {
        // MPSC parallel test
        let n_threads: usize = rand::random_range(16..=32) as usize;
        let n_iter_per_thread: usize = rand::random_range(512..=2048) as usize;
        println!("n_threads: {n_threads}");
        println!("n_iter_per_thread: {n_iter_per_thread}");
        let n_iter: usize = n_threads * n_iter_per_thread;
        let mut handlers: Vec<JoinHandle<()>> = vec![];
        let ctrl_buffer = CtrlBuffer::default();
        let (ctrl_sender, ctrl_receiver) = ctrl_buffer.split();
        for i in 0..n_threads {
            let ctrl_sender = ctrl_sender.clone();
            let h = tokio::spawn(async move {
                if i % 16 == 0 {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                for j in 0..n_iter_per_thread {
                    let time = (i + (j * n_iter_per_thread)) as f64;
                    ctrl_sender
                        .set_time(TimeFrom::Start(TimeUnit::Seconds(time)))
                        .await
                        .unwrap();
                }
            });
            handlers.push(h);
        }
        let t = thread::spawn(move || {
            for _ in 0..n_iter {
                let req = ctrl_receiver.recv();
                let (ctrl, req) = req.split();
                match ctrl {
                    CtrlReq::Time(time) => match time {
                        TimeFrom::Start(_) => {
                            req.send(Ok(CtrlRsp::Time(0f64)));
                        }
                        _ => {
                            panic!("Invalid time!");
                        }
                    },
                    _ => {
                        panic!("Invalid rsp!");
                    }
                }
            }
        });
        for h in handlers.drain(..) {
            assert!(h.await.is_ok());
        }
        assert!(t.join().is_ok());
    }
}
