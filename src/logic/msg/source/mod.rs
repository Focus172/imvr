mod args;
mod socket;
mod stdin;

use crate::prelude::*;

use self::args::ArgEventHandler;
// use socket::SocketEventHandler;
// use stdin::StdinEventHandler;

use ext::collections::ArrayVec;
use tokio::sync::mpsc;

use super::Msg;

#[derive(Debug)]
pub enum EventSendError {
    Init,
    // Join,
    Poll,
}

// #[derive(Debug)]
// enum EventReader {
//     Socket,
// }
// impl fmt::Display for EventReader {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Self::Socket => f.write_str("Socket"),
//         }
//     }
// }

impl fmt::Display for EventSendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Init => f.write_str("Failed to create resource"),
            // Self::Join => f.write_str("Could not close/finish an event reader."),
            Self::Poll => f.write_str("Failed to poll next event"),
        }
    }
}

impl Context for EventSendError {}

pub struct EventHandler {
    handle: ArrayVec<tokio::task::JoinHandle<Result<(), EventSendError>>, 2>,
}

impl EventHandler {
    pub fn spawn(tx: mpsc::Sender<Msg>) -> Self {
        let mut handle = ArrayVec::new();

        {
            // --- Args ---------
            let tx = tx.clone();
            let h = tokio::spawn(async move {
                let mut ret = Option::<Report<EventSendError>>::None;
                let args = ArgEventHandler::new();
                for req in args {
                    match tx.send(req).await {
                        Ok(_) => {}
                        Err(e) => {
                            let e = Report::new(e);
                            let e = ext::error::Report::attach_printable(
                                e,
                                "failed to get argument request",
                            );
                            let e = ext::error::Report::change_context(e, EventSendError::Poll);
                            if let Some(ref mut r) = ret {
                                r.extend_one(e);
                            } else {
                                ret = Some(e);
                            }
                        }
                    }
                }
                log::info!("no more cli argument events");
                match ret {
                    Some(e) => Err(e),
                    None => Ok(()),
                }
            });
            handle.push(h);
        }

        {
            // --- Socket ------
            let tx = tx.clone();
            let h = tokio::spawn(async move {
                let res = self::socket::events(tx).await;

                non_fatal!(res);

                log::info!("no more socket events");
                Ok(())
            });
            handle.push(h);
        }

        EventHandler { handle}
    }

    pub async fn close(&mut self) -> Result<(), EventSendError> {
        while let Some(h) = self.handle.pop() {
            log::info!("waiting on next task");
            h.await.unwrap().unwrap();
            // .attach_printable(
            //     "Failed to join event reader task (see task backtrace for details).",
            // )
            // .change_context(EventSenderError::JoinError)??;
        }
        Ok(())
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        block_on(async { self.close().await }).unwrap()
    }
}
