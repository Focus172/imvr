pub mod logic;
pub mod prelude;
pub mod render;
pub mod task;
pub mod util;
pub mod window;
// mod mouse;

pub type ImvrEventLoop = winit::event_loop::EventLoop<WindowMsg>;
pub type ImvrEventLoopHandle = winit::event_loop::EventLoopWindowTarget<WindowMsg>;
pub type ImvrEventLoopProxy = winit::event_loop::EventLoopProxy<WindowMsg>;

use crate::prelude::*;

#[derive(Debug)]
enum ImvrError {
    Resource,
    Cleanup,
}
impl fmt::Display for ImvrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Imvr encountered an unrecoverable error")?;
        match self {
            ImvrError::Resource => f.write_str("failed to init a nessisary resource"),
            ImvrError::Cleanup => f.write_str("failed to cleanup a resource"),
        }
    }
}
impl Context for ImvrError {}

fn main() -> Result<(), ImvrError> {
    log::init();

    let event_loop = winit::event_loop::EventLoopBuilder::with_user_event()
        .build()
        .attach_printable("failed to create winit event loop")
        .change_context(ImvrError::Resource)?;

    let proxy = event_loop.create_proxy();

    let rt = tokio::runtime::Runtime::new()
        .attach_printable("failed to create tokio runtime")
        .change_context(ImvrError::Resource)?;

    let (t, r) = tokio::sync::oneshot::channel();
    // run our tokio rt on a different base thread as the main thread is reserved
    // for ui on mac
    let tokio = std::thread::spawn(|| {
        let rt = rt;

        rt.block_on(crate::task::logic(proxy, r))
    });

    crate::task::window(event_loop)
        .attach_printable("Window thread panicd. this is unrecoverable on MacOs so if you are reading this good job")
        .change_context(ImvrError::Cleanup)?;

    log::info!("Waiting on tokio rt");

    let _ = t.send(());

    tokio
        .join()
        .expect("tokio runtime paniced.")
        .attach_printable("event handlrs encountered an error")
        .change_context(ImvrError::Cleanup)?;

    Ok(())
}
