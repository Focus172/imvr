#![feature(exitcode_exit_method)]

// mod buffer;
mod context;
mod gpu;
mod image_info;
// add this back in when needed
// mod mouse;
mod events;
mod window;

use std::{collections::VecDeque, process::ExitCode, sync::Arc};

use events::EventHandler;
use gpu::GpuContext;
use image::GenericImageView;
use window::WindowIdent;

use crate::{
    context::Context,
    events::Request,
    gpu::GpuImage,
    image_info::{ImageInfo, ImageView, PixelFormat},
};

const TEST_IMG: &str = "/Users/evan/Pictures/anime/shade.jpg";

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let (mut context, event_loop) = Context::new()?;

    let mut current_win_id: Option<usize> = None;
    let mut gpu: Option<Arc<GpuContext>> = None;

    // Current Requests to for actions
    let mut request_queue: VecDeque<Request> = VecDeque::new();

    request_queue.push_back(Request::OpenWindow);
    request_queue.push_back(Request::ShowImage {
        path: TEST_IMG.into(),
        window_ident: WindowIdent::any(),
    });

    let mut handlrs = EventHandler::new(context.identity_map.clone());

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Wait;

        if let Some(e) = handlrs.winit_event(event) {
            request_queue.push_back(e);
        }

        // background_thread.cli.dump_reqests(&mut context.request_queue)
        // background_thread.socket.dump_reqests(&mut context.request_queue)

        // context.run_requests(event_loop, control_flow);

        if let Some(req) = request_queue.pop_front() {

        match req {
            Request::Multiple(reqs) => {
                for req in reqs {
                }
            }
            Request::ShowImage { path, window_ident } => {
                log::warn!("Got a next image event");

                let img = image::open(path).unwrap();

                let (w, h) = img.dimensions();

                // let ctype = img.color();
                let color_type = img.color();

                log::warn!("Color type is: {:?}", color_type);

                let buf: Vec<u8> = img.into_bytes();

                let image = match color_type {
                    image::ColorType::L8 => todo!(),
                    image::ColorType::La8 => todo!(),
                    image::ColorType::Rgb8 => {
                        let info = ImageInfo::new(PixelFormat::Rgb8, w, h);
                        ImageView::new(info, &buf)
                    }
                    image::ColorType::Rgba8 => todo!(),
                    _ => todo!(),
                };

                let gpu_im = GpuImage::from_data(
                    "basic_dumb_name".into(),
                    &gpu.as_ref().unwrap().device,
                    &gpu.as_ref().unwrap().image_bind_group_layout,
                    &image,
                );

                log::info!("rendering thing image");
                let window = &mut context.windows[window_ident.index];

                window.image = Some(gpu_im);
                window.uniforms.mark_dirty(true);
                log::info!("Ready to redraw");
                window.window.request_redraw();
            }
            Request::Exit => {
                // join all the processing threads
                ExitCode::from(0).exit_process()
            }
            Request::Resize { size, window_ident} => {
                if size.x > 0 && size.y > 0 {
                    let size = glam::UVec2::from_array([size.x, size.y]);
                    let _ = context.resize_window(window_ident, size, gpu.as_ref().unwrap());
                }
            }
            Request::Redraw { window_ident } => {
                log::info!("Redrawing");
                context
                    .render_window(window_ident, gpu.as_ref().unwrap())
                    .unwrap();
            }
            Request::OpenWindow => {
                log::info!("imvr: creating main window");

                // TODO: currently this doesn't support making multipul windows which is sad
                if gpu.is_some() {
                    unimplemented!()
                }
                let (window_id, new_gpu) =
                    context.create_window(event_loop, "image", None).unwrap();

                let a_gpu = Arc::new(new_gpu);
                log::info!("Created gpu thing");

                _ = gpu.insert(a_gpu);
                _ = current_win_id.insert(window_id);
            }
            Request::CloseWindow { window_ident } => {
                log::error!("This is really unsafe as it doesn't update any of the idents and so they end up pointing");
                log::error!("garbage and can be used for evil. Eh i will fix it later");
                context.windows.remove(window_ident.index);
            }
        }
        }

        if context.windows.is_empty() {
            request_queue.push_back(Request::Exit);
        }
    });
}
