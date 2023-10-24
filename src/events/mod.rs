mod args;
// mod socket;
// mod stdin;

use args::ArgEventHandler;
use fallible_iter_ext::prelude::FallibleIteratorExt;
use futures::{executor::block_on, Stream};
// use socket::SocketEventHandler;
// use stdin::StdinEventHandler;
use tokio::sync::{mpsc, oneshot};

// use crate::prelude::*;

use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

use crate::util::key::Key;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Multiple(Vec<Request>),
    ShowImage {
        path: PathBuf,
        window_id: u64,
    },
    #[serde(skip)]
    OpenWindow {
        res: oneshot::Sender<u64>,
    },
    CloseWindow {
        window_id: u64,
    },
    Exit,
    Resize {
        size: glam::UVec2,
        window_id: u64,
    },
    Redraw {
        window_id: u64,
    },
    Tick
}

impl Request {
    pub fn redraw(window_id: winit::window::WindowId) -> Self {
        Self::Redraw {
            window_id: window_id.into(),
        }
    }

    pub fn resize(
        physical_size: winit::dpi::PhysicalSize<u32>,
        window_id: winit::window::WindowId,
    ) -> Self {
        Self::Resize {
            size: (physical_size.width, physical_size.height).into(),
            window_id: window_id.into(),
        }
    }

    pub fn close(window_id: winit::window::WindowId) -> Self {
        Self::CloseWindow {
            window_id: window_id.into(),
        }
    }
}

pub struct EventHandler {
    handles: Vec<tokio::task::JoinHandle<()>>,
    reqs: mpsc::Receiver<Request>,
}

impl EventHandler {
    pub async fn new() -> Self {
        let mut handles = vec![];

        let (tx, reqs) = mpsc::channel(4);

        {
            // --- Args ---------
            let tx = tx.clone();
            let h = tokio::spawn(async move {
                let args = ArgEventHandler::new();
                for req in args.fuse_err() {
                    tx.send(req).await.unwrap();
                }
            });
            handles.push(h);
        }

        {
            // --- Keep Alive ---------
            let tx = tx.clone();
            let h = tokio::spawn(async move {
                loop {
                    let timeout = tokio::time::sleep(Duration::from_secs(1));
                    tokio::select! {
                        _ = tx.closed() => {
                            return;
                        }
                        _ = timeout => {
                            tx.send(Request::Tick).await.unwrap();
                        }

                    }
                }
            });
            handles.push(h);
        }

        EventHandler { handles, reqs }
    }

    pub async fn close(&mut self) {
        self.reqs.close();
        while let Some(h) = self.handles.pop() {
            h.await.unwrap();
        }
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        block_on(async { self.close().await })
    }
}

impl Stream for EventHandler {
    type Item = Request;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.reqs.poll_recv(cx)
    }
}

impl EventHandler {
    // pub fn new() -> Self {
    //     let args_event_handler = ArgEventHandler::new();
    //     // let socket_event_handler = SocketEventHandler::new();
    //     let stdin_event_handler = StdinEventHandler::new();
    //     let window_event_handler = WindowEventHandler::new();
    //
    //     Self {
    //         args_event_handler,
    //         // socket_event_handler,
    //         stdin_event_handler,
    //         window_event_handler,
    //         queued_reqs: VecDeque::new(),
    //     }
    // }

    // pub fn add_window_event(&mut self, event: WEvent<()>) {
    //     self.window_event_handler.add(event)
    // }

    // pub fn next(&mut self) -> Option<Request> {
    //     self.yeild();
    //     self.queued_reqs.pop_front()
    // }
}

// trait EventParser<E> {
//     /// Takes in an event and returns the amount of requests generated
//     /// from the event wrapped in a result.
//     fn parse(&mut self, event: E) -> Option<Request>;
//
//     /// Closes the event handler haulting any events
//     fn close(&mut self) -> !;
// }

pub fn parse_key(key: Key) -> Result<Request, ()> {
    match key {
        Key::Char('q') => Ok(Request::Exit),
        Key::Char('l') => todo!("Select Next Image"),
        Key::Char(_) => todo!(),
        Key::Ctrl('c') => Ok(Request::Exit),
        Key::Ctrl(_) => todo!(),
        Key::Alt(_) => todo!(),
        Key::Nothing => todo!(),
    }
}
