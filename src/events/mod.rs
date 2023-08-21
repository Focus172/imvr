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

#[derive(Serialize, Deserialize, Clone)]
pub enum Request {
    Multiple(Vec<Request>),
    ShowImage { path: PathBuf, window_id: u64 },
    OpenWindow,
    CloseWindow { window_id: u64 },
    Exit,
    Resize { size: glam::UVec2, window_id: u64 },
    Redraw { window_id: u64 },
}

pub struct EventHandler {
    args_event_handler: ArgEventHandler,
    args_enabled: bool,

    socket_event_handler: SocketEventHandler,
    socket_enabled: bool,

    stdin_event_handler: StdinEventHandler,
    stdin_enabled: bool,

    window_event_handler: WindowEventHandler,
    window_enabled: bool,

    queued_reqs: VecDeque<(Request, Source)>,
}

#[derive(PartialEq, Eq, Clone)]
pub enum Source {
    Arg,
    Window,
    Stdin,
    Socket,
    Manual,
}

impl EventHandler {
    pub fn new() -> Self {
        let args_event_handler = ArgEventHandler::new();
        let socket_event_handler = SocketEventHandler::new();
        let stdin_event_handler = StdinEventHandler::new();
        let window_event_handler = WindowEventHandler::new();

        Self {
            args_event_handler,
            args_enabled: true,

            socket_event_handler,
            socket_enabled: true,

            stdin_event_handler,
            stdin_enabled: true,

            window_event_handler,
            window_enabled: true,

            queued_reqs: VecDeque::new(),
        }
    }

    pub fn add_window_event(&mut self, event: WEvent<()>) {
        // self.window_event_handler;
    }

    pub fn make_request(&mut self, req: Request) {
        self.queued_reqs.push_back((req, Source::Manual));
    }

    pub fn yeild(&mut self) {
        if let Some(e) = self.args_event_handler.next() {
            if self.args_enabled {
                self.queued_reqs.push_back((e, Source::Arg));
            }
        }
    }

    pub fn disable_source(&mut self, source_type: Source) {
        if source_type == Source::Manual {
            return;
        }

        let new_queue = self
            .queued_reqs
            .iter()
            .filter(|(_e, source)| source != &source_type)
            .cloned()
            .collect();

        match source_type {
            Source::Arg => self.args_enabled = false,
            Source::Socket => self.socket_enabled = false,
            Source::Stdin => self.stdin_enabled = false,
            Source::Window => self.window_enabled = false,
            _ => unreachable!(),
        }

        self.queued_reqs = new_queue;
    }

    pub fn enable_source(&mut self, source_type: Source) {
        match source_type {
            Source::Arg => self.args_enabled = true,
            Source::Socket => self.socket_enabled = true,
            Source::Stdin => self.stdin_enabled = true,
            Source::Window => self.window_enabled = true,
            _ => unreachable!(),
        }
    }

    pub fn next(&mut self) -> Option<Request> {
        self.queued_reqs.pop_front().map(|(event, source)| event)
    }
}
