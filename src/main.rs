#![feature(exitcode_exit_method)]

mod background_thread;
mod buffers;
mod color;
mod context;
mod gpu;
mod image_info;
// mod mouse;
mod request;
mod window;

use std::path::PathBuf;

use crate::{
    context::Context,
    image_info::{ImageInfo, ImageView},
    request::Request,
    window::WindowOptions,
};

const IMG_DIR: &str = "/Users/evan/Pictures/anime";

fn main() {
    env_logger::init();

    let (mut context, event_loop) = Context::new().unwrap();

    let mut files: Vec<PathBuf> = std::fs::read_dir(IMG_DIR)
        .unwrap()
        .map(|f| f.unwrap().path())
        .collect();

    let mut raw_img = image::open(files.pop().unwrap()).unwrap();
    let mut image_set = false;

    let mut current_win_id: Option<usize> = None;

    // TODO: Parse args to create an some initial requests

    context.request_queue.push_back(Request::OpenWindow);
    context
        .request_queue
        .push_back(Request::ShowImage(files.pop().unwrap()));

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Wait;

        if current_win_id.is_none() {
            log::info!("imvr: creating main window");
            let window_id = context
                .create_window(event_loop, "image", Default::default())
                .unwrap();
            _ = current_win_id.insert(window_id);
        }

        context.handle_event(event, event_loop);
        // background_thread.cli.dump_reqests(&mut context.request_queue)
        // background_thread.socket.dump_reqests(&mut context.request_queue)

        // context.run_requests(event_loop, control_flow);

        while let Some(req) = context.request_queue.pop_front() {
            match req {
                request::Request::NextImage => {
                    log::warn!("imvr: moving to next image");
                    raw_img = image::open(files.pop().unwrap()).unwrap();
                    image_set = false;
                }
                request::Request::Exit => context.exit(0.into()),
                _ => {}
            }
        }

        if !image_set {
            log::warn!("imvr: setting the image thing");
            let h = raw_img.height();
            let w = raw_img.width();
            // let ctype = img.color();
            let buf: Vec<u8> = raw_img.to_rgb8().pixels().flat_map(|p| p.0).collect();
            let image = ImageView::new(ImageInfo::rgb8(w, h), &buf);

            log::info!("Read image to vec");
            let im = context.make_gpu_image("image-001", &image);
            log::info!("Created gpu image");

            let window = &mut context.windows[current_win_id.unwrap()];

            window.image = Some(im);
            window.uniforms.mark_dirty(true);
            window.window.request_redraw();

            image_set = true;
        }

        if context.windows.is_empty() {
            context.exit(0.into());
        }
    });
}
