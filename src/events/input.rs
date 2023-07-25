// Module for reading from stdin and adding requests to the context
//

use std::{io::Stdin, thread::JoinHandle};

use super::EventParser;

struct TerminalState {
    in_raw_mode: bool,
    locked: bool,
}

struct InputParser {
    term: TerminalState,
    reader: JoinHandle<()>,
    rx: std::sync::mpsc::Receiver<E>,
}

impl EventParser for InputParser {
    type E = String;

    fn new(req_handle: super::RequestQueueHandle) -> Self {
        let han = std::thread::spawn({});

        todo!();
    }

    fn parse(event: E) -> anyhow::Result<usize> {}

    fn close() -> ! {
        todo!()
    }
}
