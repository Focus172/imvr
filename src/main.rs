#![feature(exitcode_exit_method)]

mod backend;
mod background_thread;
mod error;
mod event;
mod image;
mod image_info;
mod oneshot;
mod rectangle;
mod color;

use crate::backend::proxy::WindowProxy;
use crate::backend::window::WindowOptions;
use crate::backend::context::ContextHandle;
use crate::image_info::ImageInfo;
use crate::image::ImageView;
use crate::backend::proxy::ContextProxy;
use crate::color::Color;
use crate::event::Event;
use crate::backend::context::Context;

use winit::window::WindowId;
use std::time::Duration;
use std::process::ExitCode;

static mut PATH: Option<String> = None;

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
        user_main(handle.clone()).unwrap();
        exit(handle, 0.into());
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
    let img = unsafe { match &PATH {
        Some(p) => image_out::open(p).unwrap(),
        None => {
            eprintln!("this should not be supported");
            image_out::open("/Users/evan/Pictures/anime/cat.jpg").unwrap()
        }
    }};
    let mut buf = Vec::new();
    let h = img.height();
    let w = img.width();
    // let ctype = img.color();
    img.to_rgb8().pixels().for_each(|p| {
        p.0.iter().for_each(|v| {
            buf.push(*v)
        });
    });

    let image = ImageView::new(ImageInfo::rgb8(w, h), &buf);

    // Create a window with default options and display the image.
    let window = create_window(handle, "image", Default::default())?;
    window.set_image("image-001", image)?;

    std::thread::sleep(Duration::from_secs(1));

    Ok(())
}

pub fn create_window(handle: ContextProxy, title: impl Into<String>, options: WindowOptions) -> Result<WindowProxy, error::CreateWindowError> {
	let title = title.into();
	handle.run_function_wait(move |context| {
		let window = context.create_window(title, options)?;
		Ok(window.proxy())
	})
}

pub fn exit(handle: ContextProxy, code: ExitCode) -> ! {
	handle.exit(code);
}
