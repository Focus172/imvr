// Module for reading evnets from socket and emitting requests

use crate::{logic::msg::Msg, prelude::*};
use std::{
    io::{Read, Write},
    os::unix::net::UnixListener,
    sync::mpsc::Receiver,
    time::Duration,
};

use tokio::task::JoinHandle;

#[derive(Debug)]
pub enum SocketEventError {
    SocketClosed,
    CantConnect(&'static str),
    NoMoreHandle,
    JoinError(&'static str),
}
impl fmt::Display for SocketEventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SocketClosed => f.write_str("Socket closed."),
            Self::CantConnect(path) => write!(f, "unable to connect to unix socket at {path:?}"),
            Self::NoMoreHandle => f.write_str("tried to take join handle but it was missing"),
            Self::JoinError(name) => f.write_str("failed to join {name} task"),
        }
    }
}
impl Context for SocketEventError {}

#[derive(Debug)]
pub struct SocketEventHandler {
    pub commands: Receiver<Msg>,
    pub handle: Option<JoinHandle<Result<!, SocketEventError>>>,
}

impl SocketEventHandler {
    pub fn new() -> Result<Self, SocketEventError> {
        const IMVR_PATH: &str = "/tmp/imvr.sock";
        let listener = UnixListener::bind(IMVR_PATH)
            .change_context(SocketEventError::CantConnect(IMVR_PATH))?;
        // drop(listener);

        let (tx, commands) = std::sync::mpsc::channel();

        let handle = tokio::spawn(async move {
            let _ = tx;
            let mut handles = Vec::new();

            let mut inner = || {
                // listener.set_nonblocking(true)?;
                for res in listener.incoming() {
                    let mut stream = res
                        .attach_printable("Socket connection closed")
                        .change_context(SocketEventError::SocketClosed)?;

                    let h = tokio::spawn(async move {
                        // read the stream line by line and use serde to parse it as json
                        // then send the event to the listener where it can be parsed to
                        // a request
                        let _ = stream.read(&mut []);
                        // let _ = stream.write(b"hello world");
                        let _ = stream.write(b"");
                    });
                    handles.push(h);
                }

                unreachable!("socket listener failed to wait for new connections")
            };

            let res: Result<!, SocketEventError> = inner();
            let e = res.unwrap_err();
            for handle in handles {
                if !handle.is_finished() {
                    handle.abort();
                }
                // let a = handle.await.unwrap_err();
                // e.extend_one(a)
            }

            Err(Report::new(SocketEventError::SocketClosed))
        });

        let handle = Some(handle);

        Ok(Self { commands, handle })
    }

    pub async fn close(&mut self) -> Result<(), SocketEventError> {
        let h = self
            .handle
            .take()
            .ok_or(Report::new(SocketEventError::NoMoreHandle))?;
        h.abort();
        let e = h
            .await
            .attach_printable("failed to join thread for reasons.")
            .change_context(SocketEventError::JoinError("spawner"))?
            .unwrap_err(); // can never be ok variant beacuse `Ok(!)` cant be constructed

        log::error!("socket threw error (as expected): {e}");

        Ok(())
    }
}

impl Iterator for SocketEventHandler {
    type Item = Msg;

    fn next(&mut self) -> Option<Self::Item> {
        if self.handle.as_ref().is_some_and(|h| h.is_finished()) {
            None
            // let a = self.handle.await;
        } else {
            std::thread::sleep(Duration::from_secs(3));
            None
        }
    }
}

impl Drop for SocketEventHandler {
    fn drop(&mut self) {
        block_on(async { self.close().await }).unwrap()
    }
}
