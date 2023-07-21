#![feature(exitcode_exit_method)]

mod backend;
mod background_thread;
mod error;
mod event;
mod image;
mod image_info;
mod oneshot;
mod color;

use crate::backend::{
    proxy::WindowProxy,
    window::WindowOptions,
    context::{ContextHandle, Context}
};
use crate::image::ImageView;
use crate::image_info::ImageInfo;
use crate::backend::proxy::ContextProxy;
use crate::color::Color;
use crate::event::Event;

use winit::window::WindowId;
use std::time::Duration;

// static mut PATH: Option<String> = None;

fn main() {

    // let args = std::env::args().collect::<Vec<String>>();
    // match args.get(1).map(|s| s.as_str()) {
    //     Some("command") => {
    //
    //     },
    //     Some(_p) => {
    //         unsafe { PATH = Some(_p.to_string()); }
    //     },
    //     None => {
    //         eprintln!("this should fail");
    //     },
    // }

	let mut context = Context::new(wgpu::TextureFormat::Bgra8Unorm).unwrap();
    let handle = context.proxy.clone();

    std::thread::spawn(move || {
        let code = match user_main(handle.clone()) {
            Ok(_) => 0.into(),
            Err(e) => {
                eprint!("imvr: {}", e);
                1.into()
            }
        };
	    handle.exit(code);
    });

    let event_loop = context.event_loop.take().unwrap();
    event_loop.run(move |event, event_loop, control_flow| {
        let initial_window_count = context.windows.len();
        context.handle_event(event, event_loop, control_flow);

        // Check if the event handlers caused the last window(s) to close.
        // If so, generate an AllWIndowsClosed event for the event handlers.
        if context.windows.is_empty() && initial_window_count > 0 {
            context.run_event_handlers(&mut Event::AllWindowsClosed, event_loop);
            if context.exit_with_last_window {
                context.exit(0.into());
            }
        }
    });

}

fn user_main(handle: ContextProxy) -> Result<(), Box<dyn std::error::Error>> {
    let img = // unsafe { match &PATH {
    //     Some(p) => image_out::open(p).unwrap(),
    //     None => {
    //         eprintln!("this should not be supported");
            image_out::open("/Users/evan/Pictures/anime/cat.jpg").unwrap()
    //     }
    // }}
    ;

    let h = img.height();
    let w = img.width();
    // let ctype = img.color();
    let buf: Vec<u8> = img.to_rgb8().pixels().map(|p| p.0).flatten().collect();

    let image = ImageView::new(ImageInfo::rgb8(w, h), &buf);

    // Create a window with default options and display the image.
    let window = handle.run_function_wait(move |context| {
		let window = context.create_window("image", Default::default()).unwrap();
		window.proxy()
	});
    window.set_image("image-001", image)?;

    std::thread::sleep(Duration::from_secs(1));

    Ok(())
}
