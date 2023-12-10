use crate::prelude::*;

use crate::ImvrEventLoop;

#[derive(Debug)]
pub struct WindowError;

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Window thread enocuntered and error. Likely the result of an OS error.")
    }
}
impl Context for WindowError {}

pub fn window(event_loop: ImvrEventLoop) -> Result<(), WindowError> {
    let mut context = GlobalContext::new();

    // let mut count: usize = 0;

    event_loop
        .run(move |event, event_loop_target| {
            // count += 1;
            // log::info!("start event loop {}", count);
            if let winit::event::Event::UserEvent(ref e) = event {
                log::info!("user event: {:?}", &e);
            }

            if let Some(req) = event.some_into() {
                // log::info!("Handling next request: {:?}", &req);
                context.handle_request(req, event_loop_target).unwrap();
            }

            // if context.windows.is_empty() {
            //     log::warn!("Exiting beacuse no windows are open.");
            //     event_loop_target.exit();
            // }

            // log::info!("ended event loop {}", count);
        })
        .attach_printable("event loop returned unexpected error.")
        .change_context(WindowError)?;

    log::warn!("Event Loop Ended.");

    Ok(())
}
