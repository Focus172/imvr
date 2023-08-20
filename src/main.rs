#![feature(exitcode_exit_method)]

// mod buffer;
mod context;
mod gpu;
mod image_info;
// add this back in when needed
// mod mouse;
mod events;
mod window;

use std::collections::VecDeque;

use events::EventHandler;

use crate::{
    context::Context,
    events::Request,
    image_info::{ImageInfo, ImageView},
};

const TEST_IMG: &str = "/Users/evan/Pictures/anime/shade.jpg";

fn main() -> anyhow::Result<()> {
    env_logger::init();

    log::trace!("Trying to create a new context");
    let (mut context, event_loop) = Context::new()?;

    // creates and async task
    let mut handlrs = EventHandler::new(context.identity_map.clone());

    // Current Requests to for actions
    let mut request_queue: VecDeque<Request> = VecDeque::new();

    log::trace!("Creating initial requests");
    request_queue.push_back(Request::OpenWindow);
    request_queue.push_back(Request::ShowImage {
        path: TEST_IMG.into(),
        window_index: 0,
    });
    



    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Wait;

        // handlrs.add_winit_event(evnet);
        if let Some(e) = handlrs.winit_event(event) {
            request_queue.push_back(e);
        }
        // handlrs.next();
        

        // request_queue.extend(handlrs.dump_reqests());

        // context.run_requests(event_loop, control_flow);

        while let Some(req) = request_queue.pop_front() {
            context.handle_request(req, event_loop);
        }

        // Hack seeing as this is there can be events on the stack that would
        // create the window this is bad. however, it works for now.
        if context.windows.is_empty() {
            request_queue.push_back(Request::Exit);
        }
    });
}
