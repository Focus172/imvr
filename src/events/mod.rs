mod input;
mod rpc;
mod system;

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub struct BackgroundThread<T> {
    done: Arc<AtomicBool>,
    handle: std::thread::JoinHandle<T>,
}

impl<T> BackgroundThread<T> {
    pub fn is_done(&self) -> bool {
        self.done.load(Ordering::Acquire)
    }

    pub fn join(self) -> std::thread::Result<T> {
        self.handle.join()
    }
}

use std::path::PathBuf;

pub enum Request {
    NextImage,
    OpenWindow,
    ShowImage(PathBuf),
    Exit,
}
