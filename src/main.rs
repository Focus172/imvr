#![feature(exitcode_exit_method)]

mod context;
mod events;
mod gpu;
mod image_info;
mod prelude;
mod util;
mod window;

// mod mouse;

use events::EventHandler;

use crate::prelude::*;

use crate::image_info::{ImageInfo, ImageView};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    log::trace!("Trying to create a new context");
    let (mut context, event_loop) = Context::new()?;

    // creates and async task
    let mut handlrs = EventHandler::new();

    log::trace!("Creating initial requests");

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Wait;

        handlrs.add_window_event(event);

        handlrs.yeild();

        while let Some(req) = handlrs.next() {
            context.handle_request(req, event_loop);
        }

        if context.windows.is_empty() {
            handlrs.make_request(Request::Exit);
        }
    });
}
