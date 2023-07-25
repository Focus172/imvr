// mod input;
// pub mod rpc;
// mod system;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::window::WindowIdent;

#[derive(Serialize, Deserialize)]
pub enum Request {
    Multiple(Vec<Request>),
    ShowImage {
        path: PathBuf,
        window_id: WindowIdent,
    },
    OpenWindow,
    CloseWindow {
        window_id: WindowIdent,
    },
    Exit,
    Resize {
        size: glam::UVec2,
        window_id: WindowIdent,
    },
    Redraw {
        window_id: WindowIdent,
    },
}

trait EventParser<E> {
    fn new() -> Self;

    /// Takes in an event and returns the amount of requests generated
    /// from the event wrapped in a result.
    fn parse(event: E) -> Option<Request>;

    /// Closes the event handler haulting any events
    fn close() -> !;
}
