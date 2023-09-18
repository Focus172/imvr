mod args;
mod socket;
mod stdin;
mod window;

use args::ArgEventHandler;
use socket::SocketEventHandler;
use stdin::StdinEventHandler;
use window::WindowEventHandler;

// use crate::prelude::*;

use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, path::PathBuf};
use winit::event::Event as WEvent;

#[derive(Serialize, Deserialize)]
pub enum Request {
    Multiple(Vec<Request>),
    ShowImage {
        path: PathBuf,
        window_id: u64,
    },
    #[serde(skip)]
    OpenWindow {
        res: oneshot::Sender<u64>,
    },
    CloseWindow {
        window_id: u64,
    },
    Exit {
        code: Option<u8>,
    },
    Resize {
        size: glam::UVec2,
        window_id: u64,
    },
    Redraw {
        window_id: u64,
    },
}

pub struct EventHandler {
    args_event_handler: ArgEventHandler,
    socket_event_handler: SocketEventHandler,
    stdin_event_handler: StdinEventHandler,
    window_event_handler: WindowEventHandler,

    queued_reqs: VecDeque<Request>,
}

impl EventHandler {
    pub fn new() -> Self {
        let args_event_handler = ArgEventHandler::new();
        let socket_event_handler = SocketEventHandler::new();
        let stdin_event_handler = StdinEventHandler::new();
        let window_event_handler = WindowEventHandler::new();

        Self {
            args_event_handler,
            socket_event_handler,
            stdin_event_handler,
            window_event_handler,
            queued_reqs: VecDeque::new(),
        }
    }

    pub fn add_window_event(&mut self, event: WEvent<()>) {
        self.window_event_handler.add(event)
    }

    pub fn make_request(&mut self, req: Request) {
        self.queued_reqs.push_back(req);
    }


    // pub fn disable_source(&mut self, source_type: Source) {
    //     if source_type == Source::Manual {
    //         return;
    //     }
    //
    //     let new_queue = self
    //         .queued_reqs
    //         .iter()
    //         .filter(|(_e, source)| source != &source_type)
    //         .cloned()
    //         .collect();
    //
    //     match source_type {
    //         Source::Arg => self.args_enabled = false,
    //         Source::Socket => self.socket_enabled = false,
    //         Source::Stdin => self.stdin_enabled = false,
    //         Source::Window => self.window_enabled = false,
    //         _ => unreachable!(),
    //     }
    //
    //     self.queued_reqs = new_queue;
    // }
    //
    // pub fn enable_source(&mut self, source_type: Source) {
    //     match source_type {
    //         Source::Arg => self.args_enabled = true,
    //         Source::Socket => self.socket_enabled = true,
    //         Source::Stdin => self.stdin_enabled = true,
    //         Source::Window => self.window_enabled = true,
    //         _ => unreachable!(),
    //     }
    // }

    pub fn next(&mut self) -> Option<Request> {
        self.yeild();
        self.queued_reqs.pop_front()
    }

    fn yeild(&mut self) {
        if let Some(e) = self.args_event_handler.next() {
            self.queued_reqs.push_back(e);
        }

        if let Some(e) = self.window_event_handler.next() {
            self.queued_reqs.push_back(e);
        }
    }
}

// trait EventParser<E> {
//     /// Takes in an event and returns the amount of requests generated
//     /// from the event wrapped in a result.
//     fn parse(&mut self, event: E) -> Option<Request>;
//
//     /// Closes the event handler haulting any events
//     fn close(&mut self) -> !;
// }
