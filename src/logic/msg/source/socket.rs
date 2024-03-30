// Module for reading evnets from socket and emitting requests

use crate::{logic::msg::Msg, prelude::*};
use std::io;
use tokio::sync::mpsc;

// #[derive(Debug)]
// pub enum SocketEventError {
//     SocketClosed,
//     CantConnect(&'static str),
//     NoMoreHandle,
//     JoinError(&'static str),
// }
// impl fmt::Display for SocketEventError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Self::SocketClosed => f.write_str("Socket closed."),
//             Self::CantConnect(path) => write!(f, "unable to connect to unix socket at {path:?}"),
//             Self::NoMoreHandle => f.write_str("tried to take join handle but it was missing"),
//             Self::JoinError(_) => f.write_str("failed to join {name} task"),
//         }
//     }
// }
// impl Context for SocketEventError {}

pub(super) async fn events(tx: mpsc::Sender<Msg>) -> Result<(), super::EventSendError> {
    const IMVR_PATH: &str = "/tmp/imvr.sock";

    let listener = tokio::net::UnixListener::bind(IMVR_PATH)
        .attach_printable("could not connect to socket")
        .change_context(super::EventSendError::Init)?;

                // let res = tx.send(msg).await;
                // .attach_printable("failed to send request")
                // .change_context(EventSenderError::PollError);
                // non_fatal!(res);

    async fn handle(res: io::Result<(tokio::net::UnixStream, tokio::net::unix::SocketAddr)>) {
        // let end = tx.closed();
        // let res = listener.accept().await;
        //
        // dbg!(b);
        // let mut buf = [0; 1028];
        // tokio::io::AsyncReadExt::read(&mut a, &mut buf);
        dbg!(&res);
        res.unwrap();

        // let mut handles = Vec::new();
        //
        // let mut inner = || {
        //     // listener.set_nonblocking(true)?;
        //     for res in listener.incoming() {
        //         let mut stream = res
        //             .attach_printable("Socket connection closed")
        //             .change_context(SocketEventError::SocketClosed)?;
        //
        //         let h = tokio::spawn(async move {
        //             // read the stream line by line and use serde to parse it as json
        //             // then send the event to the listener where it can be parsed to
        //             // a request
        //             let _ = stream.read(&mut []);
        //             // let _ = stream.write(b"hello world");
        //             let _ = stream.write(b"");
        //         });
        //         handles.push(h);
        //     }
        //
        //     unreachable!("socket listener failed to wait for new connections")
        // };
        //
        // let res: Result<Infallible, SocketEventError> = inner();
        // let e = res.unwrap_err();
        // for handle in handles {
        //     if !handle.is_finished() {
        //         let _ = handle.await.unwrap_err();
        //         // e.extend_one(a)
        //     }
        // }
        //
        // return e;

        // close
        // let h = self
        //     .handle
        //     .take()
        //     .ok_or(Report::new(SocketEventError::NoMoreHandle))?;
        // h.abort();
        // let e = h
        //     .await
        //     .attach_printable("failed to join thread for reasons.")
        //     .change_context(SocketEventError::JoinError("spawner"))?;
        //
        // log::error!("socket threw error (as expected): {e}");
        //
        // Ok(())
    }

    loop {
        tokio::select! {
            _ = tx.closed() => {
                break
            }
            r = listener.accept() => handle(r).await,
        }
    }

    Ok(())
}
