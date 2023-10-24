#![feature(never_type)]

mod cli;
mod ctx;
mod events;
mod gpu;
mod image_info;
mod prelude;
mod util;
mod window;

// mod mouse;

use futures::StreamExt;
use std::thread;

use crate::events::EventHandler;
use crate::image_info::{ImageInfo, ImageView};
use crate::prelude::*;

use tokio::sync::mpsc::error::TryRecvError;
use winit::event_loop::{EventLoop, EventLoopProxy};

fn main() -> Result<()> {
    res::install()?;
    logger::init();

    let (tx_req, rx_req) = mpsc::channel(16);

    let event_loop = winit::event_loop::EventLoop::new()?;
    let proxy = event_loop.create_proxy();

    // run our tokio rt on a different base thread as the main thread is reserved
    // for ui on some platforms
    let tokio = thread::spawn(|| {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(logic_main(tx_req, proxy))
    });

    window_main(rx_req, event_loop)?;

    log::info!("Waiting on tokio rt");
    tokio.join().unwrap()?;

    Ok(())
}

fn window_main(mut rx_req: mpsc::Receiver<Request>, event_loop: EventLoop<()>) -> Result<()> {
    let mut context = Context::new()?;

    let mut count: usize = 0;
    event_loop.run(move |event, event_loop_target| {
        count += 1;
        log::info!("start event loop {}", count);
        log::info!("Event: {:?}", &event);

        let res: Option<Request> = event.try_into().ok().or_else(|| match rx_req.try_recv() {
            Ok(req) => Some(req),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                log::warn!("Exiting Beacuse receiver diconnected.");
                event_loop_target.exit();
                None
            }
        });

        if let Some(req) = res {
            log::info!("Handling next request: {:?}", &req);
            context.handle_request(req, event_loop_target).unwrap();
        }

        if context.windows.is_empty() {
            log::warn!("Exiting beacuse no windows are open.");
            event_loop_target.exit();
        }

        log::info!("ended event loop {}", count);
    })?;

    log::warn!("Event Loop Ended.");

    Ok(())
}

pub type WinitEvent = winit::event::Event<()>;

async fn logic_main(tx: mpsc::Sender<Request>, _event_loop: EventLoopProxy<()>) -> Result<()> {
    // creates and async task
    let mut handlrs = EventHandler::new().await;

    loop {
        log::debug!("Waiting on next event.");
        tokio::select! {
            Some(req) = handlrs.next() => {
                log::info!("New request: {:?}", &req);
                tx.send(req).await?;
            }
            _ = tx.closed() => {
                break;
            }

        }
        // event_loop.send_event(()).unwrap();
    }

    // tx_req.send(Request::Exit).await?;

    Ok(())
}
