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
}
impl fmt::Display for SocketEventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SocketClosed => f.write_str("Sockted closed."),
        }
    }
}
impl Context for SocketEventError {}

#[derive(Debug)]
pub struct SocketEventHandler {
    pub commands: Receiver<Msg>,
    pub handle: JoinHandle<Result<!, SocketEventError>>,
}

impl SocketEventHandler {
    pub fn new() -> Self {
        let listener = UnixListener::bind("/tmp/imvr.sock").unwrap();
        let (tx, commands) = std::sync::mpsc::channel();

        let handle = tokio::spawn(async move {
            let _ = tx;

            for res in listener.incoming() {
                let mut stream = res
                    .attach_printable("Socket connection closed")
                    .change_context(SocketEventError::SocketClosed)?;

                tokio::spawn(async move {
                    // read the stream line by line and use serde to parse it as json
                    // then send the event to the listener where it can be parsed to
                    // a request
                    let _ = stream.read(&mut []);
                    let _ = stream.write(b"hello world");
                });
            }
            unreachable!("socket listener failed to wait for new connections")
        });

        Self { commands, handle }
    }
}

impl Iterator for SocketEventHandler {
    type Item = Msg;

    fn next(&mut self) -> Option<Self::Item> {
        if self.handle.is_finished() {
            None
            // let a = self.handle.await;
        } else {
            std::thread::sleep(Duration::from_secs(3));
            None
        }
    }
}

impl Drop for SocketEventHandler {
    fn drop(&mut self) {}
}
