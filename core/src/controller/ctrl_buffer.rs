use std::{
    cell::UnsafeCell,
    pin::Pin,
    ptr,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicPtr, Ordering},
    },
    task::{Context, Poll, Waker},
};

use crate::{
    Error,
    controller::{
        CtrlSender,
        ctrl_receiver::{CtrlReceiver, CtrlRsp},
        ctrl_sender::CtrlReq,
    },
};

struct CtrlShared {
    pub(crate) res: Option<Result<CtrlRsp, Error>>,
    pub(crate) waker: Option<Waker>,
    pub(crate) wait_cvar: Option<Arc<Condvar>>,
}

struct CtrlReqFuture {
    shared: Arc<Mutex<CtrlShared>>,
}

impl Future for CtrlReqFuture {
    type Output = Result<CtrlRsp, Error>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut guard = self.shared.lock().unwrap();
        if let Some(res) = guard.res.take() {
            Poll::Ready(res)
        } else {
            guard.waker = Some(ctx.waker().clone());
            Poll::Pending
        }
    }
}

pub(crate) struct CtrlRspParam {
    shared: Arc<Mutex<CtrlShared>>,
}

impl CtrlRspParam {
    pub(crate) fn send(&self, rsp: Result<CtrlRsp, Error>) {
        let mut guard = self.shared.lock().unwrap();
        guard.res = Some(rsp);
        if let Some(waker) = guard.waker.take() {
            waker.wake();
        }
        if let Some(wait_cvar) = guard.wait_cvar.take() {
            wait_cvar.notify_all();
        }
    }
}

pub(crate) struct CtrlReqParam {
    pub(crate) ctrl: CtrlReq,
    shared: Arc<Mutex<CtrlShared>>,
}

impl CtrlReqParam {
    pub(crate) fn split(self) -> (CtrlReq, CtrlRspParam) {
        (
            self.ctrl,
            CtrlRspParam {
                shared: self.shared,
            },
        )
    }
}

struct Block {
    next: AtomicPtr<Block>,
    req: UnsafeCell<Option<CtrlReqParam>>,
}

pub(crate) struct CtrlBuffer {
    head: AtomicPtr<Block>,
    tail: AtomicPtr<Block>,
    // for blocking
    wait_lock: Mutex<()>,
    wait_cvar: Condvar,
}

unsafe impl Send for CtrlBuffer {}
unsafe impl Sync for CtrlBuffer {}

impl Drop for CtrlBuffer {
    fn drop(&mut self) {
        let mut head = self.head.load(Ordering::Acquire);
        while !head.is_null() {
            let next = unsafe { (*head).next.load(Ordering::Acquire) };
            let block = unsafe { Box::from_raw(head) };
            // Unblock any waiting sender
            if let Some(req) = unsafe { (*block.req.get()).take() } {
                let (_, req) = req.split();
                req.send(Err(Error::msg("Controller dropped".to_string())));
            }
            drop(block);
            head = next;
        }
    }
}

impl Default for CtrlBuffer {
    fn default() -> Self {
        let block = Box::into_raw(Box::new(Block {
            next: AtomicPtr::new(ptr::null_mut()),
            req: UnsafeCell::new(None),
        }));
        Self {
            head: AtomicPtr::new(block),
            tail: AtomicPtr::new(block),
            wait_lock: Mutex::new(()),
            wait_cvar: Condvar::new(),
        }
    }
}

impl CtrlBuffer {
    /// Creates a ring buffer and returns the producer and consumer
    ///
    /// # Arguments
    /// * `n_channels` - Number of channels
    /// * `buffer_size_per_ch` - The buffer size per channel
    ///
    #[must_use]
    pub fn split(self) -> (CtrlSender, CtrlReceiver) {
        let buffer = Arc::new(self);
        (CtrlSender::new(buffer.clone()), CtrlReceiver::new(buffer))
    }

    pub(crate) fn is_empty(&self) -> bool {
        let head = self.head.load(Ordering::Acquire);
        let next = unsafe { (*head).next.load(Ordering::Acquire) };
        next.is_null()
    }

    /// Receives controller req. If not avaialble, the method waits till data available.
    ///
    pub(crate) fn recv(&self) -> CtrlReqParam {
        loop {
            let mut guard = self.wait_lock.lock().unwrap();
            if let Some(req) = self.try_recv() {
                break req;
            }
            guard = self.wait_cvar.wait(guard).unwrap();
            drop(guard);
        }
    }

    /// Receives controller req if data is avaialble.
    ///
    pub(crate) fn try_recv(&self) -> Option<CtrlReqParam> {
        // MPSC logic
        let head = self.head.load(Ordering::Acquire);
        let next = unsafe { (*head).next.load(Ordering::Acquire) };
        if next.is_null() {
            return None;
        }
        let req = unsafe { (*(*next).req.get()).take() };
        self.head.store(next, Ordering::Release);
        drop(unsafe { Box::from_raw(head) });
        req.map(|req| req)
    }

    pub(crate) fn send_blocking(&self, ctrl: CtrlReq) -> Result<CtrlRsp, Error> {
        let wait_cvar = Arc::new(Condvar::new());
        let req_param = CtrlReqParam {
            ctrl,
            shared: Arc::new(Mutex::new(CtrlShared {
                res: None,
                waker: None,
                wait_cvar: Some(wait_cvar.clone()),
            })),
        };
        let shared = req_param.shared.clone();
        // MPSC logic
        {
            let new_block = Box::into_raw(Box::new(Block {
                next: AtomicPtr::new(ptr::null_mut()),
                req: UnsafeCell::new(Some(req_param)),
            }));
            let old_tail = self.tail.swap(new_block, Ordering::AcqRel);
            unsafe {
                (*old_tail).next.store(new_block, Ordering::Release);
            }
        }
        self.wait_cvar.notify_all();
        let mut guard = shared.lock().unwrap();
        loop {
            if let Some(res) = guard.res.take() {
                break res;
            }
            guard = wait_cvar.wait(guard).unwrap()
        }
    }

    pub(crate) async fn send(&self, ctrl: CtrlReq) -> Result<CtrlRsp, Error> {
        let req_param = CtrlReqParam {
            ctrl,
            shared: Arc::new(Mutex::new(CtrlShared {
                res: None,
                waker: None,
                wait_cvar: None,
            })),
        };
        let req_future = CtrlReqFuture {
            shared: req_param.shared.clone(),
        };
        // MPSC logic
        {
            let new_block = Box::into_raw(Box::new(Block {
                next: AtomicPtr::new(ptr::null_mut()),
                req: UnsafeCell::new(Some(req_param)),
            }));
            let old_tail = self.tail.swap(new_block, Ordering::AcqRel);
            unsafe {
                (*old_tail).next.store(new_block, Ordering::Release);
            }
        }
        self.wait_cvar.notify_all();
        req_future.await
    }
}
