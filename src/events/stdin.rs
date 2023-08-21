// Module for reading from stdin and adding requests to the context
//

use crate::prelude::*;

struct TerminalState {
    in_raw_mode: bool,
    locked: bool,
}

pub struct StdinEventHandler {
    term: TerminalState,
    // reader: JoinHandle<()>,
    // rx: std::sync::mpsc::Receiver<E>,
}

impl StdinEventHandler {
    pub fn new() -> Self {
        // let han = std::thread::spawn({});
        todo!();
    }

    pub fn next(&mut self) -> Option<Request> {
        todo!()
    }

    pub fn exit(&mut self) {
        println!("Exiting!")
    }
}

impl Drop for StdinEventHandler {
    fn drop(&mut self) {
        self.exit()
    }
}
