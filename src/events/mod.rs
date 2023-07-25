// mod input;
// pub mod rpc;
pub mod system;

use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use winit::{event::Event, window::WindowId};

use self::system::WinitEventHandler;

pub struct EventHandler {
    // rpc_event_handler:
    winit_event_handler: WinitEventHandler,
}

impl EventHandler {
    pub fn new(identity_map: Arc<Mutex<BTreeMap<WindowId, usize>>>) -> Self {
        let winit_event_handler = WinitEventHandler::new(identity_map);
        Self {
            winit_event_handler,
        }
    }
    pub fn winit_event(&mut self, event: Event<()>) -> Option<Request> {
        self.winit_event_handler.parse(event)
    }
}

#[derive(Serialize, Deserialize)]
pub enum Request {
    Multiple(Vec<Request>),
    ShowImage {
        path: PathBuf,
        window_index: usize,
    },
    OpenWindow,
    CloseWindow {
        window_index: usize,
    },
    Exit,
    Resize {
        size: glam::UVec2,
        window_index: usize,
    },
    Redraw {
        window_index: usize,
    },
}

trait EventParser<E> {
    /// Takes in an event and returns the amount of requests generated
    /// from the event wrapped in a result.
    fn parse(&mut self, event: E) -> Option<Request>;

    /// Closes the event handler haulting any events
    fn close(&mut self) -> !;
}
