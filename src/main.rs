#![feature(exitcode_exit_method)]

mod backend;
mod background_thread;
mod color;
// mod error;
// mod event;
mod image;
mod image_info;

use crate::backend::{context::Context, window::WindowOptions};
use crate::color::Color;
use crate::image::ImageView;
use crate::image_info::ImageInfo;

use winit::window::WindowId;

fn main() {
    let (mut context, event_loop) = Context::new(wgpu::TextureFormat::Bgra8Unorm).unwrap();

    let img = image_out::open("/Users/evan/Pictures/anime/cat.jpg").unwrap();
    let mut image_set = false;

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Wait;

        context.handle_event(event, event_loop);

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

        if context.windows.is_empty() {
            context.exit(0.into());
        }
    });
}
