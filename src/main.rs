#![feature(exitcode_exit_method)]

mod backend;
mod background_thread;
mod color;
mod error;
mod event;
mod image;
mod image_info;
mod oneshot;

use crate::backend::{
    context::Context,
    proxy::{ContextProxy, WindowProxy},
    window::WindowOptions,
};
use crate::color::Color;
use crate::event::Event;
use crate::image::ImageView;
use crate::image_info::ImageInfo;

use winit::window::WindowId;

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

    let img = // unsafe { match &PATH {
    //     Some(p) => image_out::open(p).unwrap(),
    //     None => {
    //         eprintln!("this should not be supported");
            image_out::open("/Users/evan/Pictures/anime/cat.jpg").unwrap()
    //     }
    // }}
    ;

    let mut image_set = false;

    let event_loop = context.event_loop.take().unwrap();
    event_loop.run(move |event, event_loop, control_flow| {
        context.handle_event(event, event_loop, control_flow);

        if !image_set {
            let h = img.height();
            let w = img.width();
            // let ctype = img.color();
            let buf: Vec<u8> = img.to_rgb8().pixels().flat_map(|p| p.0).collect();
            let image = ImageView::new(ImageInfo::rgb8(w, h), &buf);

            // the window needs to be made before the image bc it creates the gpu info
            let window_id = context
                .create_window(event_loop, "image", Default::default())
                .unwrap();

            let im = context.make_gpu_image("image-001", &image);

            let win = &mut context.windows[window_id];
            win.image = Some(im);
            win.uniforms.mark_dirty(true);
            win.window.request_redraw();

            image_set = true;
        }

        // Check if the event handlers caused the last window(s) to close.
        // If so, generate an AllWIndowsClosed event for the event handlers.
        if context.windows.is_empty() {
            context.run_event_handlers(&mut Event::AllWindowsClosed, event_loop);
            context.exit(0.into());
        }
    });
}
