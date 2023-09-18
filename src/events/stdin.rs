// Module for reading from stdin and adding requests to the context
//

use crate::prelude::*;

#[derive(Default)]
struct TerminalState {
    in_raw_mode: bool,
    locked: bool,
}

#[derive(Default)]
pub struct StdinEventHandler {
    term: TerminalState,
    // reader: JoinHandle<()>,
    // rx: std::sync::mpsc::Receiver<E>,
}

impl StdinEventHandler {
    pub fn new() -> Self {
        // let han = std::thread::spawn({});
        Self::default()
    }

    pub fn next(&mut self) -> Option<Request> {
        todo!()
    }

    pub fn exit(&mut self) {
        if self.term.locked {
            // unlock the terminal
        }
        if self.term.in_raw_mode {
            // exit raw mode
        }

        println!("Exiting!")
    }
}

impl Drop for StdinEventHandler {
    fn drop(&mut self) {
        self.exit()
    }
}
