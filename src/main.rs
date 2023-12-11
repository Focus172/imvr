#![feature(never_type)]
#![feature(inline_const)]
#![feature(const_option)]
#![feature(yeet_expr)]

pub mod logic;
pub mod prelude;
pub mod render;
pub mod task;
pub mod window;
// mod mouse;

pub type ImvrEventLoop = winit::event_loop::EventLoop<WindowMsg>;
pub type ImvrEventLoopHandle = winit::event_loop::EventLoopWindowTarget<WindowMsg>;
pub type ImvrEventLoopProxy = winit::event_loop::EventLoopProxy<WindowMsg>;

use crate::prelude::*;

#[derive(Debug)]
struct ImvrError;
impl fmt::Display for ImvrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("imvr: encountered an unrecoverable error.")
    }
}
impl Context for ImvrError {}

fn main() -> Result<(), ImvrError> {
    // res::install()?;
    ext::log::init();

    let event_loop = winit::event_loop::EventLoopBuilder::with_user_event()
        .build()
        .attach_printable("failed to create winit event loop")
        .change_context(ImvrError)?;

    let proxy = event_loop.create_proxy();

    // run our tokio rt on a different base thread as the main thread is reserved
    // for ui on mac
    let tokio = std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(crate::task::logic(proxy))
    });

    crate::task::window(event_loop)
        .attach_printable("window thread panic. this is unrecoverable on MacOs so if you are reaing this good job")
        .change_context(ImvrError)?;

    ext::log::info!("Waiting on tokio rt");

    tokio
        .join()
        .expect("tokio runtime paniced.")
        .attach_printable("event handlrs encountered an error")
        .change_context(ImvrError)?;

    Ok(())
}
