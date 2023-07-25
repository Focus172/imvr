#![feature(exitcode_exit_method)]

mod buffer;
mod context;
mod gpu;
mod image_info;
// add this back in when needed
// mod mouse;
mod events;
mod window;

use std::{path::PathBuf, process::ExitCode, sync::Arc};

use buffer::ImagePrebuffer;
use gpu::GpuContext;

use crate::{
    buffer::PrebufferMessage,
    context::Context,
    events::Request,
    image_info::{ImageInfo, ImageView},
};

const IMG_DIR: &str = "/Users/evan/Pictures/anime";

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let (mut context, event_loop) = Context::new()?;

    let mut files: Vec<PathBuf> = std::fs::read_dir(IMG_DIR)?
        .map(|f| f.unwrap().path())
        .collect();

    let mut next_image = None;

    let mut current_win_id: Option<usize> = None;
    let mut gpu: Option<Arc<GpuContext>> = None;

    // TODO: Parse args to create an some initial requests

    context.request_queue.push_back(Request::OpenWindow);

    // HACK: nothing happens unless i do this
    context.request_queue.push_back(Request::NextImage);

    let (mut prebuffer, (tx, rx)) = ImagePrebuffer::new();
    std::thread::spawn(move || {
        _ = prebuffer.run();
    });

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Wait;

        context.handle_event(event, event_loop);

        // background_thread.cli.dump_reqests(&mut context.request_queue)
        // background_thread.socket.dump_reqests(&mut context.request_queue)

        // context.run_requests(event_loop, control_flow);

        match context.request_queue.pop_front().unwrap_or(Request::None) {
            // match req {
            Request::NextImage => {
                log::warn!("Got a next image event");
                _ = tx.send(PrebufferMessage::LoadPath(files.pop().unwrap()));
                _ = next_image.insert(rx.recv().unwrap());
            }
            Request::Exit => {
                // join all the processing threads
                ExitCode::from(0).exit_process()
            }
            Request::Resize { size, window_id } => {
                if size.x > 0 && size.y > 0 {
                    let size = glam::UVec2::from_array([size.x, size.y]);
                    let _ = context.resize_window(window_id, size, gpu.as_ref().unwrap());
                }
            }
            Request::Redraw { window_id } => {
                context
                    .render_window(window_id, gpu.as_ref().unwrap())
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
                _ = tx.send(PrebufferMessage::InitGpu(a_gpu.clone()));
                log::info!("Created gpu thing");

                // needed to get reader one step ahead writer
                _ = tx.send(PrebufferMessage::LoadPath(files.pop().unwrap()));

                _ = gpu.insert(a_gpu);
                _ = current_win_id.insert(window_id);
            }
            Request::ShowImage(_) => todo!(),
            Request::None => {}
        }

        if let Some(gpu_img) = next_image.take() {
            log::info!("rendering thing image");
            let window = &mut context.windows[current_win_id.unwrap()];
            let s = gpu_img.info().size;
            context.request_queue.push_back(Request::Resize {
                size: s,
                window_id: window.id(),
            });

            window.image = Some(gpu_img);
            window.uniforms.mark_dirty(true);
            window.window.request_redraw();
        }

        if context.windows.is_empty() {
            context.request_queue.push_back(Request::Exit);
        }
    });
}
