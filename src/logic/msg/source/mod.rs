mod args;
mod socket;
mod stdin;

use crate::prelude::*;

use self::{args::ArgEventHandler, socket::SocketEventHandler};
// use socket::SocketEventHandler;
// use stdin::StdinEventHandler;

use ext::collections::ArrayVec;

use super::Msg;

#[derive(Debug)]
pub struct EventSenderError;

impl fmt::Display for EventSenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Failed to create request for window.")
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

        {
            // --- Args ---------
            let tx = tx.clone();
            let h = tokio::spawn(async move {
                let args = ArgEventHandler::new();
                for req in args {
                    tx.send(req)
                        .await
                        .attach_printable("failed to send request")
                        .change_context(EventSenderError)?;
                }
                Ok(())
            });
            handle.push(h);
        }

        // {
        //     // --- Socket ------
        //     let tx = tx.clone();
        //     let h = tokio::spawn(async move {
        //         let reqs = SocketEventHandler::new();
        //         for req in reqs {
        //             tx.send(req).await.unwrap();
        //         }
        //         Ok(())
        //     });
        //     handle.push(h);
        // }

        EventHandler { handle, receiv }
    }

    pub async fn close(&mut self) -> Result<(), EventSenderError> {
        self.receiv.close();

        while let Some(h) = self.handle.pop() {
            h.await
                .attach_printable("failed to join event task.")
                .change_context(EventSenderError)??;
        }
        Ok(())
    }

    pub async fn next(&mut self) -> Option<Msg> {
        self.receiv.recv().await
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        block_on(async { _ = self.close().await })
    }
}
