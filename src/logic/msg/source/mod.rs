mod args;
mod socket;
mod stdin;

use crate::prelude::*;

use self::{args::ArgEventHandler, socket::SocketEventHandler};
// use socket::SocketEventHandler;
// use stdin::StdinEventHandler;

use ext::{collections::ArrayVec, parse::MoveIt};

use super::Msg;

#[derive(Debug)]
pub enum EventSenderError {
    InitError,
    JoinError,
    PollError,
}

#[derive(Debug)]
enum EventReader {
    Socket,
}

impl fmt::Display for EventReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Socket => f.write_str("Socket"),
        }
    }
}

impl fmt::Display for EventSenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InitError => f.write_str("Nonfatal: Failed to initalize an event reader."),
            Self::JoinError => f.write_str("Could not close/finish an event reader."),
            Self::PollError => f.write_str("Nonfatal: Error while generating an event."),
        }
    }
}

impl Context for EventSenderError {}

pub struct EventHandler {
    handle: ArrayVec<tokio::task::JoinHandle<Result<(), EventSenderError>>, 2>,
    receiv: mpsc::Receiver<Msg>,
}

impl EventHandler {
    pub fn spawn() -> Self {
        let mut handle = ArrayVec::new();
        let (tx, receiv) = mpsc::channel(4);

        fn map_send_err<T>(
            e: std::result::Result<T, mpsc::error::SendError<Msg>>,
        ) -> Result<T, EventSenderError> {
            e.attach_printable("failed to send request")
                .change_context(EventSenderError::PollError)
        }

        {
            // --- Args ---------
            let tx = tx.clone();
            let h = tokio::spawn(async move {
                let args = ArgEventHandler::new();
                for req in args {
                    tx.send(req).await.move_it(map_send_err)?;
                }
                log::info!("no more cli argument events");
                Ok(())
            });
            handle.push(h);
        }

        {
            // --- Socket ------
            let tx = tx.clone();
            let h = tokio::spawn(async move {
                let res = SocketEventHandler::new();
                // .attach_printable("could not listen to events on socket.")
                // .change_context(EventSenderError::InitError);

                let reqs = match res {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("{}", e);
                        return Ok(());
                    }
                };

                for msg in reqs.take_while(|_| !tx.is_closed()) {
                    tx.send(msg).await.move_it(map_send_err)?;
                }
                log::info!("no more socket events");
                Ok(())
            });
            handle.push(h);
        }

        EventHandler { handle, receiv }
    }

    pub async fn close(&mut self) -> Result<(), EventSenderError> {
        self.receiv.close();

        while let Some(h) = self.handle.pop() {
            h.await
                .attach_printable(
                    "Failed to join event reader task (see task backtrace for details).",
                )
                .change_context(EventSenderError::JoinError)??;
        }
        Ok(())
    }

    pub async fn next(&mut self) -> Option<Msg> {
        self.receiv.recv().await
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        block_on(async { self.close().await }).unwrap()
    }
}
