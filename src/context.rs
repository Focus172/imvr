use crate::events::Request;
use crate::gpu::GpuImage;
use crate::image_info::{ImageInfo, ImageView, PixelFormat};
use crate::{
    gpu::{GpuContext, UniformsBuffer},
    window::{Window, WindowUniforms},
};
use glam::Affine2;
use image::GenericImageView;
use std::cell::OnceCell;
use std::collections::BTreeMap;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use winit::window::WindowButtons;
use winit::{
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::WindowId,
};

/// The global context managing all windows and producing winit events
pub struct Context {
    /// The wgpu instance to create surfaces with.
    pub instance: wgpu::Instance,

    /// The swap chain format to use.
    pub swap_chain_format: wgpu::TextureFormat,

    /// The windows.
    pub windows: Vec<Window>,

    pub gpu: OnceCell<GpuContext>,

    pub identity_map: Arc<Mutex<BTreeMap<WindowId, usize>>>,
}

impl Context {
    /// Creates a new global context returning the event loop for it
    pub fn new() -> anyhow::Result<(Self, EventLoop<()>)> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: Default::default(),
        });

        // Consider adding user events
        let event_loop = winit::event_loop::EventLoop::new();

        Ok((
            Self {
                instance,
                swap_chain_format: wgpu::TextureFormat::Bgra8Unorm,
                windows: Vec::new(),
                gpu: OnceCell::new(),
                identity_map: Arc::new(Mutex::new(BTreeMap::new())),
            },
            event_loop,
        ))
    }

    pub fn handle_request(&mut self, req: Request, event_loop: &EventLoopWindowTarget<()>) {
        match req {
            Request::Multiple(reqs) => {
                for req in reqs {
                    self.handle_request(req, event_loop);
                }
            }
            Request::ShowImage { path, window_index } => {
                if self.gpu.get().is_none() || self.windows.is_empty() {
                    log::warn!("Don't try to set the image before you have a valid context");
                    return;
                }
                let img = image::open(path).unwrap();
                let (w, h) = img.dimensions();
                let color_type = img.color();
                log::info!("Image color type is: {:?}", color_type);

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
                    "imvr_gpu_image".into(),
                    &self.gpu.get().unwrap().device,
                    &self.gpu.get().unwrap().image_bind_group_layout,
                    &image,
                );

                let window = &mut self.windows[window_index];

                window.image = Some(gpu_im);
                window.uniforms.mark_dirty(true);
                window.window.request_redraw();
            }
            Request::Exit => {
                // join all the processing threads
                ExitCode::from(0).exit_process()
            }
            Request::Resize { size, window_index } => {
                if size.x > 0 && size.y > 0 {
                    let size = glam::UVec2::from_array([size.x, size.y]);
                    let _ = self.resize_window(window_index, size);
                }
            }
            Request::Redraw { window_index } => {
                self.render_window(window_index).unwrap();
            }
            Request::OpenWindow => {
                log::info!("imvr: creating main window");
                self.create_window(event_loop, "image").unwrap();
            }
            Request::CloseWindow { window_index } => {
                log::error!("This is really unsafe as it doesn't update any of the idents and so they end up pointing");
                log::error!("garbage and can be used for evil. Eh i will fix it later");
                self.windows.remove(window_index);
            }
        }
    }

    /// Create a window.
    pub fn create_window(
        &mut self,
        event_loop: &EventLoopWindowTarget<()>,
        title: impl Into<String>,
    ) -> anyhow::Result<()> {
        let window = winit::window::WindowBuilder::new()
            .with_title(title)
            .with_visible(true)
            // .with_resizable(true)
            // .with_decorations(false)
            // .with_transparent(true)
            .with_enabled_buttons(WindowButtons::empty())
            // .with_inner_size(winit::dpi::PhysicalSize::new(size[0], size[1]))
            .with_fullscreen(None);

        let window = window.build(event_loop)?;
        let surface = unsafe { self.instance.create_surface(&window) }.unwrap();

        let gpu = match self.gpu.take() {
            Some(x) => x,
            None => GpuContext::new(&self.instance, self.swap_chain_format, &surface)?,
        };

        let size = glam::UVec2::new(window.inner_size().width, window.inner_size().height);
        configure_surface(size, &surface, self.swap_chain_format, &gpu.device);
        let uniforms = UniformsBuffer::from_value(
            &gpu.device,
            &WindowUniforms::no_image(),
            &gpu.window_bind_group_layout,
        );

        let window = Window {
            window,
            preserve_aspect_ratio: true,
            background_color: wgpu::Color::default(),
            surface,
            uniforms,
            image: None,
            user_transform: Affine2::IDENTITY,
        };

        let index = self.windows.len();

        self.identity_map.lock().unwrap().insert(window.id(), index);
        self.windows.push(window);

        self.gpu.set(gpu).unwrap();

        Ok(())
    }

    /// Resize a window.
    pub fn resize_window(&mut self, index: usize, new_size: glam::UVec2) -> anyhow::Result<()> {
        let window = self.windows.get_mut(index).unwrap();

        configure_surface(
            new_size,
            &window.surface,
            self.swap_chain_format,
            &self.gpu.get().unwrap().device,
        );

        window.uniforms.mark_dirty(true);

        Ok(())
    }

    /// Render the contents of a window.
    pub fn render_window(&mut self, index: usize) -> anyhow::Result<()> {
        let window = self.windows.get_mut(index).unwrap();

        let image = match &window.image {
            Some(x) => x,
            None => return Ok(()),
        };

        let frame = window
            .surface
            .get_current_texture()
            .expect("Failed to acquire next frame");

        let device = &self.gpu.get().unwrap().device;
        let mut encoder = device.create_command_encoder(&Default::default());

        if window.uniforms.is_dirty() {
            window
                .uniforms
                .update_from(device, &mut encoder, &window.calculate_uniforms());
        }

        // --------------- RENDER PASS BEGIN ------------------- //
        let load = wgpu::LoadOp::Clear(window.background_color);

        let surface = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render-image"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &surface,
                resolve_target: None,
                ops: wgpu::Operations { load, store: true },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.gpu.get().unwrap().window_pipeline);
        render_pass.set_bind_group(0, window.uniforms.bind_group(), &[]);
        render_pass.set_bind_group(1, image.bind_group(), &[]);
        render_pass.draw(0..6, 0..1);
        drop(render_pass);
        // --------------- RENDER PASS END ------------------- //

        self.gpu().queue.submit(std::iter::once(encoder.finish()));

        frame.present();
        Ok(())
    }

    fn gpu(&self) -> &GpuContext {
        self.gpu
            .get()
            .expect("This should only be called after the first screen is maade.")
    }
}

/// Create a swap chain for a surface.
fn configure_surface(
    size: glam::UVec2,
    surface: &wgpu::Surface,
    format: wgpu::TextureFormat,
    device: &wgpu::Device,
) {
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.x,
        height: size.y,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: Default::default(),
        view_formats: Default::default(),
    };
    surface.configure(device, &config);
}
