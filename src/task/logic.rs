use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::logic::msg::EventHandler;

use crate::prelude::*;

#[derive(Debug)]
pub struct LogicalError;

impl fmt::Display for LogicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Logic thread encountered and unrecoverable error")
    }
}

impl Context for LogicalError {}

/// Main logic task and root of tokio runtime.
///
/// takes a proxy to the event loop and an interupt handle.
/// when any data is sent on the handle the programe exits
pub async fn logic(
    elp: crate::ImvrEventLoopProxy,
    mut cls: oneshot::Receiver<()>,
) -> Result<(), LogicalError> {
    let (tx, mut rx) = mpsc::channel(4);

    // spawns the tasks
    let mut handlrs = EventHandler::spawn(tx);

    loop {
        // this cant be done with `select` beacuse oneshot's future takes 
        // ownership

        use tokio::sync::mpsc::error::TryRecvError as MTRE;
        match rx.try_recv() {
            Ok(mut msg) => {
                if let Some(msg) = msg.as_window() {
                    elp.send_event(msg)
                        .attach_printable("Failed to send request to render thread.")
                        .change_context(LogicalError)?;
                }

                if let Some(_msg) = msg.as_terminal() {
                    // log::warn!("send {msg:?} to terminal")
                }
            }
            Err(MTRE::Disconnected) => break,
            Err(MTRE::Empty) => {}
        }

        use tokio::sync::oneshot::error::TryRecvError as OTRE;
        match cls.try_recv() {
            Ok(_) | Err(OTRE::Closed) => break,
            Err(OTRE::Empty) => {}
        }
    }

    rx.close();
    handlrs.close().await.change_context(LogicalError)?;

    Ok(())
}
