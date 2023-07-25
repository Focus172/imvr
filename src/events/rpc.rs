// Module for reading evnets from socket and emitting requests

use std::{
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    sync::mpsc::{Receiver, Sender},
};

use crate::events::Request;

use super::EventParser;

pub struct RpcParser {
    pub commands: Receiver<String>,
    socket_tx: UnixStream,
    socket_rx: UnixListener,
}

impl EventParser for RpcParser {
    fn new() -> Self {
        let listener = UnixListener::bind("/tmp/imvr.sock").unwrap();
        let mut stream = UnixStream::connect("/tmp/imvr.sock")?;

        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let (stream, addr) = sock.accept().unwrap();

            println!("Got a client: {:?} - {:?}", sock, addr);
            // stream.write_all(b"hello world")?;
            let mut response = String::new();
            let input = stream
                .bytes()
                .filter(|b| b.is_ok())
                .map(|b| b.unwrap())
                .take_while(|b| b == b'\n')
                .collect::<Vec<u8>>();

            let string = String::from_utf8(input).unwrap();

            // read_to_string(&mut response)?;
            println!("{}", string);
        });

        Self {
            commands: rx,
            socket_tx: stream,
            socket_rx: listener,
        }
    }

    fn parse(event: E) -> Option<Request> {}

    fn close() -> ! {}
}

pub fn run(tx: Sender<Request>) {}
