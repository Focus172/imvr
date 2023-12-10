use crate::prelude::*;

use crate::logic::req::WindowRequest;
use crate::render::gpu::image::GpuImage;
use crate::render::gpu::image::{ImageInfo, ImageView};
use crate::window::Window;
use crate::ImvrEventLoopHandle;
use ext::glam::UVec2;
use image::GenericImageView;
use winit::window::WindowId;

/// The global context managing all windows and producing winit events
#[derive(Debug)]
pub struct GlobalContext {
    /// The wgpu instance to create surfaces with.
    pub instance: wgpu::Instance,

    /// The windows.
    pub windows: Vec<Window>,
    // pub gpu: OnceCell<GpuContext>,
}

impl Default for GlobalContext {
    fn default() -> Self {
        Self {
            instance: wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                ..Default::default()
            }),
            windows: Default::default(),
            // gpu: Default::default(),
        }
    }
}

const SWAP_CHAIN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

impl GlobalContext {
    /// Creates a new global context returning the event loop for it
    pub fn new() -> Self {
        GlobalContext::default()
    }

    pub fn handle_request(
        &mut self,
        req: WindowRequest,
        event_loop: &ImvrEventLoopHandle,
    ) -> Result<(), GlobalContextError> {
        match req {
            WindowRequest::Many(reqs) => {
                for req in reqs {
                    self.handle_request(req, event_loop)?;
                }
            }
            WindowRequest::ShowImage { image, id } => {
                let window = self
                    .windows
                    .iter_mut()
                    .find(|win| win.id() == id)
                    .ok_or_else(|| {
                        Report::new(GlobalContextError)
                            .attach_printable(format!("No open window matches id: {:?}", id))
                    })?;

                let img = image;
                let (w, h) = img.dimensions();
                let color_type = img.color();

                log::info!("Image color type is: {:?}", color_type);

                let buf = img.into_bytes();

                let image = ImageView::new(ImageInfo::new(color_type.into(), w, h), &buf);

                let gpu = &window.context;
                let gpu_im = GpuImage::from_data(
                    "imvr_gpu_image".into(),
                    &gpu.device,
                    &gpu.image_bind_group_layout,
                    &image,
                );

                // const WINDOW_NOT_FOUND_CTX = |id| {
                //     Report::new(GlobalContextError)
                //             .attach_printable(format!("No open window matches id: {}", id))
                // };

                window.image = Some(gpu_im);
                window.uniforms.mark_dirty(true);
                window.window.request_redraw();
            }
            WindowRequest::Exit => {
                // TODO: join all the processing threads
                event_loop.exit();
            }
            WindowRequest::Resize { size, window_id } => {
                log::trace!("resize: ({},{})", size.x, size.y);
                if size.x > 0 && size.y > 0 {
                    let size = UVec2::from_array([size.x, size.y]);
                    let _ = self.resize_window(window_id.into(), size);
                }
            }
            WindowRequest::WindowRedraw { id } => {
                self.render_window(id.into()).unwrap();
            }
            WindowRequest::OpenWindow { resp } => {
                let id = self.create_window(event_loop)?;

                resp.send(id)
                    .attach_printable_lazy(|| {
                        format!("Could not send id {id:?} back to requester.")
                    })
                    .change_context(GlobalContextError)?;
            }
            WindowRequest::CloseWindow { window_id } => {
                log::debug!("imvr: closing window {}", window_id);
                let idx = self.index_from_id(window_id).unwrap_or(0);
                self.windows.remove(idx);
                log::info!("imvr: window {} closed", window_id);

                if self.windows.is_empty() {
                    event_loop.exit()
                }
            }
        }
        Ok(())
    }

    /// Creates a new window
    fn create_window(
        &mut self,
        event_loop: &ImvrEventLoopHandle,
    ) -> Result<u64, GlobalContextError> {
        log::debug!("imvr: creating window");

        let window = Window::new(event_loop, "image", &self.instance).unwrap();
        let id = window.id().into();

        log::info!("Setting up window");
        window.window.set_visible(true);
        self.windows.push(window);

        log::info!("imvr: created window {}", id);

        Ok(id)
    }

    fn index_from_id(&self, window_id: u64) -> Option<usize> {
        self.windows
            .iter()
            .position(|win| win.id() == window_id.into())
    }

    /// Resize a window.
    pub fn resize_window(
        &mut self,
        window_id: WindowId,
        new_size: UVec2,
    ) -> Result<(), GlobalContextError> {
        let window = self
            .windows
            .iter_mut()
            .find(|win| win.id() == window_id)
            .unwrap();

        window
            .surface
            .configure(&window.context.device, &surface_config(new_size));

        window.uniforms.mark_dirty(true);

        Ok(())
    }

    /// Render the contents of a window.
    pub fn render_window(&mut self, window_id: WindowId) -> Result<(), GlobalContextError> {
        log::info!("STARTING RENDER.");

        let window = self
            .windows
            .iter_mut()
            .find(|win| win.id() == window_id)
            .unwrap();

        let image = match &window.image {
            Some(x) => x,
            None => return Ok(()),
        };

        let frame = window
            .surface
            .get_current_texture()
            .expect("Failed to acquire next frame");

        let device = &window.context.device;
        let mut encoder = device.create_command_encoder(&Default::default());

        if window.uniforms.is_dirty() {
            window
                .uniforms
                .update_from(device, &mut encoder, &window.calculate_uniforms());
        }

        // --------------- RENDER PASS BEGIN ------------------- //
        {
            let load = wgpu::LoadOp::Clear(window.background_color);

            let surface = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render-image"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load,
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                ..Default::default()
            });

            render_pass.set_pipeline(&window.context.window_pipeline);
            render_pass.set_bind_group(0, window.uniforms.bind_group(), &[]);
            render_pass.set_bind_group(1, image.bind_group(), &[]);
            render_pass.draw(0..6, 0..1);
        }
        // --------------- RENDER PASS END ------------------- //

        window
            .context
            .queue
            .submit(std::iter::once(encoder.finish()));

        frame.present();
        Ok(())
    }
}

/// Create a swap chain for a surface.
const fn surface_config(size: UVec2) -> wgpu::SurfaceConfiguration {
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: SWAP_CHAIN_FORMAT,
        width: size.x,
        height: size.y,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: Vec::new(),
    }
}

#[derive(Debug)]
pub struct GlobalContextError;

impl fmt::Display for GlobalContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Error in context.")
    }
}

impl Context for GlobalContextError {}
