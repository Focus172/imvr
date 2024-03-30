use crate::{prelude::*, ImvrEventLoopHandle};

use crate::render::gpu::image::{GpuImage, ImageInfo, ImageView};
use crate::render::gpu::{GpuContext, UniformsBuffer};
use crate::render::uniforms::WindowUniforms;
use ext::glam::{Affine2, UVec2, Vec2};
use wgpu::{Color, Instance};
use winit::window::WindowId;

/// Window capable of displaying images using wgpu.
#[derive(Debug)]
pub struct Window {
    /// The winit window.
    window: winit::window::Window,

    /// If true, preserve the aspect ratio of images.
    pub preserve_aspect_ratio: bool,

    /// The background color of the window.
    pub background_color: Color,

    /// The wgpu surface to render to.
    ///
    /// The life time here is that this borrows from the window feild
    /// it will remain valid for as long as the window is valid.
    pub surface: wgpu::Surface<'static>,

    /// The window specific uniforms for the render pipeline.
    pub uniforms: UniformsBuffer<WindowUniforms>,

    /// The image to display (if any).
    pub image: Option<GpuImage>,

    /// Transformation to apply to the image, in virtual window space.
    ///
    /// Virtual window space goes from (0, 0) in the top left to (1, 1) in the bottom right.
    pub user_transform: Affine2,

    /// The context to the gpu for this image
    pub context: GpuContext,

    adapter: wgpu::Adapter,
}

impl Window {
    /// Create a new window.
    pub fn new(
        title: impl Into<String>,
        event_loop: &ImvrEventLoopHandle,
        instance: &Instance,
    ) -> Result<Self, WindowError> {
        let window = winit::window::WindowBuilder::new()
            .with_title(title)
            .with_visible(true)
            .with_resizable(true)
            // .with_decorations(true)
            // .with_window_icon(Some(Icon::from_rgba(rgba, width, height)))
            // .with_transparent(true)
            // .with_enabled_buttons(WindowButtons::empty())
            .build(event_loop)
            .unwrap();

        // window.request_redraw();
        // window.pre_present_notify();

        let surface = instance.create_surface(&window).unwrap();
        let surface = unsafe { std::mem::transmute(surface) };

        let gpu = GpuContext::new(instance, wgpu::TextureFormat::Bgra8Unorm, &surface).unwrap();

        let a = futures::executor::block_on(
            instance.request_adapter(&wgpu::RequestAdapterOptions::default()),
        );
        let a = a.unwrap();

        let winit::dpi::PhysicalSize { width, height } = window.inner_size();
        // let size = UVec2::new(width, height);

        let mut config = surface
            .get_default_config(&a, width, height)
            .unwrap();
        config.format = wgpu::TextureFormat::Bgra8Unorm;
        surface.configure(&gpu.device, &config);

        let uniforms = UniformsBuffer::from_value(
            &gpu.device,
            &WindowUniforms::new_empty(),
            &gpu.window_bind_group_layout,
        );

        Ok(Window {
            window,
            preserve_aspect_ratio: true,
            background_color: wgpu::Color::default(),
            surface,
            uniforms,
            image: None,
            user_transform: Affine2::IDENTITY,
            context: gpu,
            adapter: a,
        })
    }

    /// Get the window ID.
    #[inline]
    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    /// Recalculate the uniforms for the render pipeline from the window state.
    pub fn calculate_uniforms(&self) -> WindowUniforms {
        if let Some(image) = &self.image {
            let image_size = image.info().size.as_vec2();
            if !self.preserve_aspect_ratio {
                WindowUniforms::new_stretched(image_size) // .pre_apply_transform(self.user_transform)
            } else {
                let window_size = UVec2::new(
                    self.window.inner_size().width,
                    self.window.inner_size().height,
                )
                .as_vec2();
                WindowUniforms::fit(window_size, image_size)
                // .pre_apply_transform(self.user_transform)
            }
        } else {
            WindowUniforms::new(self.user_transform, Vec2::ZERO)
        }
    }

    /// Resize a window.
    pub fn resize(&mut self, size: UVec2) {
        log::trace!("resize: ({},{})", size.x, size.y);
        debug_assert!(size.x > 0 && size.y > 0);

        // Create a swap chain for a surface.

        let mut config = self.surface.get_default_config(&self.adapter, size.x, size.y).unwrap();
        config.format = wgpu::TextureFormat::Bgra8Unorm;
        self.surface.configure(&self.context.device, &config);

        self.uniforms.mark_dirty(true);
    }

    /// Render the contents of a window.
    pub fn render(&mut self) -> Result<(), WindowError> {
        log::info!("STARTING RENDER.");

        let window = self;

        let image = match &window.image {
            Some(x) => x,
            None => {
                log::warn!("Skipping render beacuse there is no image for this window");
                return Ok(());
            }
        };

        let frame = window
            .surface
            .get_current_texture()
            .expect("Failed to acquire next frame");

        let device = &window.context.device;
        let mut encoder = device.create_command_encoder(&Default::default());

        if window.uniforms.is_dirty() {
            log::trace!("uniforms are dirty.");
            window
                .uniforms
                .update_from(device, &mut encoder, &window.calculate_uniforms());
        } else {
            // log::trace!("uniforms are not dirty.");
        }

        let surface = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        // --------------- RENDER PASS BEGIN ------------------- //
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render-image"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(window.background_color),
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

    pub fn set_image(&mut self, image: crate::util::RawImage) {
        let (w, h) = image.size;

        log::info!("Image color type is: {:?}", &image.color);

        let image = ImageView::new(ImageInfo::new(image.color.into(), w, h), &image.data);

        let gpu = &self.context;
        let gpu_im = GpuImage::from_data(
            "imvr_gpu_image".into(),
            &gpu.device,
            &gpu.image_bind_group_layout,
            &image,
        );

        self.image = Some(gpu_im);
        self.uniforms.mark_dirty(true);
        self.window.request_redraw();
    }
}

/// Create a surface configurations from a size
// const fn surface_config(size: UVec2) -> wgpu::SurfaceConfiguration {
//     wgpu::SurfaceConfiguration {
//         usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
//         format: wgpu::TextureFormat::Bgra8Unorm,
//         width: size.x,
//         height: size.y,
//         present_mode: wgpu::PresentMode::AutoVsync,
//         alpha_mode: wgpu::CompositeAlphaMode::Auto,
//         view_formats: Vec::new(),
//         desired_maximum_frame_latency: todo!(),
//     }
// }

#[derive(Debug)]
pub struct WindowError;
impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Window had an error.")
    }
}
impl Context for WindowError {}
