#![feature(exitcode_exit_method)]

mod buffers;
mod context;
mod gpu;
mod image_info;
// add this back in when needed
// mod mouse;
mod events;
mod window;

use std::{path::PathBuf, process::ExitCode};

use crate::{
    context::Context,
    events::Request,
    image_info::{ImageInfo, ImageView},
    window::WindowOptions,
};

use image::io::Reader as ImageReader;

const IMG_DIR: &str = "/Users/evan/Pictures/anime";

fn main() {
    env_logger::init();

    let (mut context, event_loop) = Context::new().unwrap();

    let mut files: Vec<PathBuf> = std::fs::read_dir(IMG_DIR)
        .unwrap()
        .map(|f| f.unwrap().path())
        .collect();

    let mut raw_img = ImageReader::open(files.pop().unwrap())
        .unwrap()
        .decode()
        .unwrap();
    // image::open(files.pop().unwrap()).unwrap();
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
                Request::NextImage => {
                    log::warn!("imvr: moving to next image");
                    raw_img = ImageReader::open(files.pop().unwrap())
                        .unwrap()
                        .decode()
                        .unwrap();
                    image_set = false;
                }
                Request::Exit => {
                    // join all the processing threads
                    ExitCode::from(0).exit_process()
                }
                _ => {}
            }
        }

        if !image_set {
            log::warn!("imvr: setting the image thing");
            let h = raw_img.height();
            let w = raw_img.width();
            // let ctype = img.color();
            let color_type = raw_img.color();

            log::info!("Color type is: {:?}", color_type);

            let buf: Vec<u8> = match color_type {
                image::ColorType::L8 => todo!(),
                image::ColorType::La8 => todo!(),
                image::ColorType::Rgb8 => raw_img.to_rgb8().pixels().flat_map(|p| p.0).collect(),
                image::ColorType::Rgba8 => raw_img.to_rgba8().pixels().flat_map(|p| p.0).collect(),
                image::ColorType::L16 => todo!(),
                image::ColorType::La16 => todo!(),
                image::ColorType::Rgb16 => todo!(),
                image::ColorType::Rgba16 => todo!(),
                image::ColorType::Rgb32F => todo!(),
                image::ColorType::Rgba32F => todo!(),
                _ => todo!(),
            };

            let image = match color_type {
                image::ColorType::L8 => todo!(),
                image::ColorType::La8 => todo!(),
                image::ColorType::Rgb8 => {
                    let info = ImageInfo::rgb8(w, h);
                    ImageView::new(info, &buf)
                }
                image::ColorType::Rgba8 => {
                    let info = ImageInfo::rgba8(w, h);
                    ImageView::new(info, &buf)
                }
                image::ColorType::L16 => todo!(),
                image::ColorType::La16 => todo!(),
                image::ColorType::Rgb16 => todo!(),
                image::ColorType::Rgba16 => todo!(),
                image::ColorType::Rgb32F => todo!(),
                image::ColorType::Rgba32F => todo!(),
                _ => todo!(),
            };

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
            context.request_queue.push_back(Request::Exit);
        }
    });
}
