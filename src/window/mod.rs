pub mod event;
pub mod uniforms;

use self::uniforms::WindowUniforms;

use crate::gpu::{GpuContext, GpuImage, UniformsBuffer};
use glam::{Affine2, Vec2};
use wgpu::{Color, Instance};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::WindowId;

/// Window capable of displaying images using wgpu.
pub struct Window {
    /// The winit window.
    pub window: winit::window::Window,

    /// If true, preserve the aspect ratio of images.
    pub preserve_aspect_ratio: bool,

    /// The background color of the window.
    pub background_color: Color,

    /// The wgpu surface to render to.
    pub surface: wgpu::Surface,

    /// The window specific uniforms for the render pipeline.
    pub uniforms: UniformsBuffer<WindowUniforms>,

    /// The image to display (if any).
    pub image: Option<GpuImage>,

    /// Transformation to apply to the image, in virtual window space.
    ///
    /// Virtual window space goes from (0, 0) in the top left to (1, 1) in the bottom right.
    pub user_transform: Affine2,
    // The event handlers for this specific window.
    // pub event_handlers: Vec<Box<DynWindowEventHandler>>,
}

use crate::prelude::*;


impl Window {
    /// Create a new window.
    pub fn new(
        event_loop: &EventLoopWindowTarget<()>,
        title: impl Into<String>,
        gpu: Option<GpuContext>,
        instance: &Instance,
    ) -> Result<(Self, GpuContext)> {
        let window = winit::window::WindowBuilder::new()
            .with_title(title)
            .with_visible(true)
            .with_resizable(true)
            // .with_decorations(true)
            // .with_window_icon(Some(Icon::from_rgba(rgba, width, height)))
            // .with_transparent(true)
            // .with_enabled_buttons(WindowButtons::empty())
            .build(event_loop)?;

        // window.request_redraw();
        // window.pre_present_notify();

        let surface = unsafe { instance.create_surface(&window) }?;

        let gpu = match gpu {
            Some(x) => x,
            None => GpuContext::new(instance, SWAP_CHAIN_FORMAT, &surface)?,
        };

        let size = glam::UVec2::new(window.inner_size().width, window.inner_size().height);

        configure_surface(size, &surface, &gpu.device);

        let uniforms = UniformsBuffer::from_value(
            &gpu.device,
            &WindowUniforms::no_image(),
            &gpu.window_bind_group_layout,
        );

        Ok((
            Window {
                window,
                preserve_aspect_ratio: true,
                background_color: wgpu::Color::default(),
                surface,
                uniforms,
                image: None,
                user_transform: Affine2::IDENTITY,
            },
            gpu,
        ))
    }

    /// Get the window ID.
    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    /// Recalculate the uniforms for the render pipeline from the window state.
    pub fn calculate_uniforms(&self) -> WindowUniforms {
        if let Some(image) = &self.image {
            let image_size = image.info().size.as_vec2();
            if !self.preserve_aspect_ratio {
                WindowUniforms::stretch(image_size).pre_apply_transform(self.user_transform)
            } else {
                let window_size = glam::UVec2::new(
                    self.window.inner_size().width,
                    self.window.inner_size().height,
                )
                .as_vec2();
                WindowUniforms::fit(window_size, image_size)
                    .pre_apply_transform(self.user_transform)
            }
        } else {
            WindowUniforms {
                transform: self.user_transform,
                image_size: Vec2::new(0.0, 0.0),
            }
        }
    }
}

const SWAP_CHAIN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

/// Create a swap chain for a surface.
fn configure_surface(size: glam::UVec2, surface: &wgpu::Surface, device: &wgpu::Device) {
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: SWAP_CHAIN_FORMAT,
        width: size.x,
        height: size.y,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: Default::default(),
        view_formats: Default::default(),
    };
    surface.configure(device, &config);
}

