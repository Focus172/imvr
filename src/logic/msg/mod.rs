use crate::prelude::*;

mod source;
mod terminal;
mod window;

use serde::Deserialize;
use std::os::fd::RawFd;
use std::path::PathBuf;
use tokio::sync::oneshot;

pub use self::source::{EventHandler, EventSenderError};
pub use self::{terminal::TerminalMsg, window::WindowMsg};
use super::SurfaceId;

#[derive(Deserialize)]
pub enum Msg {
    ShowImage { path: PathBuf, id: SurfaceId },
    OpenWindow { resp: Option<ReturnAddress> },
}

impl Msg {
    #[inline]
    pub fn open(sender: oneshot::Sender<u64>) -> Self {
        Self::OpenWindow {
            resp: Some(ReturnAddress::Memory(sender)),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(from = "RawFd")]
pub enum ReturnAddress {
    Memory(oneshot::Sender<u64>),
    File(RawFd),
}

impl ReturnAddress {
    pub fn send(self, value: u64) -> Result<(), ReturnerError> {
        match self {
            ReturnAddress::Memory(s) => s
                .send(value)
                .map_err(|_| Report::new(ReturnerError::SenderError)),
            ReturnAddress::File(f) => do yeet Report::new(ReturnerError::FileError(f)),
        }
    }
}

impl From<RawFd> for ReturnAddress {
    fn from(value: RawFd) -> Self {
        Self::File(value)
    }
}

#[derive(Debug)]
pub enum ReturnerError {
    SenderError,
    FileError(RawFd),
}

impl fmt::Display for ReturnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SenderError => {
                f.write_str("Data was either already sent on this channel or consumer hung up")
            }
            Self::FileError(fd) => {
                write!(f, "Failed to write data to specified fd: {fd:?}")
            }
        }
    }
}

impl Context for ReturnerError {}
