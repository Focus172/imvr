use crate::{
    logic::msg::{EventHandler, WindowMsg},
    prelude::*,
};

use winit::event_loop::EventLoopProxy;

#[derive(Debug)]
pub struct LogicalError;

impl fmt::Display for LogicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Logic thread encountered and unrecoverable error")
    }
}

impl Context for LogicalError {}

pub async fn logic(elp: EventLoopProxy<WindowMsg>) -> Result<(), LogicalError> {
    // creates an async task
    let mut handlrs = EventHandler::spawn();

    while let Some(mut msg) = handlrs.next().await {
        // log::debug!("Waiting on next event.");

        if let Some(msg) = msg.as_window() {
            elp.send_event(msg)
                .attach_printable("Failed to send request to render thread.")
                .change_context(LogicalError)?;
        }

        if let Some(_msg) = msg.as_terminal() {
            // log::warn!("send {msg:?} to terminal")
        }
    }

    handlrs.close().await.change_context(LogicalError)?;

    Ok(())
}
