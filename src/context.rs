use crate::window::WindowIdent;
use crate::{
    gpu::{GpuContext, UniformsBuffer},
    window::{Window, WindowUniforms},
};
use glam::Affine2;
use std::collections::BTreeMap;
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

    pub identity_map: Arc<Mutex<BTreeMap<WindowId, WindowIdent>>>,
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
                identity_map: Arc::new(Mutex::new(BTreeMap::new())),
            },
            event_loop,
        ))
    }

    /// Create a window.
    pub fn create_window(
        &mut self,
        event_loop: &EventLoopWindowTarget<()>,
        title: impl Into<String>,
        gpu: Option<GpuContext>,
    ) -> anyhow::Result<(usize, GpuContext)> {
        let window = winit::window::WindowBuilder::new()
            .with_title(title)
            .with_visible(true)
            .with_resizable(true)
            .with_decorations(false)
            .with_transparent(true)
            .with_enabled_buttons(WindowButtons::empty())
            .with_fullscreen(None);
        // .with_inner_size(winit::dpi::PhysicalSize::new(size[0], size[1]));

        let window = window.build(event_loop)?;
        let surface = unsafe { self.instance.create_surface(&window) }.unwrap();

        let gpu = match gpu {
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
        let ident = WindowIdent::new(
            //Some(title.into()),
            index,
        );

        self.identity_map.lock().unwrap().insert(window.id(), ident);
        self.windows.push(window);

        Ok((index, gpu))
    }

    /// Resize a window.
    pub fn resize_window(
        &mut self,
        ident: WindowIdent,
        new_size: glam::UVec2,
        gpu: &GpuContext,
    ) -> anyhow::Result<()> {
        let window = self.windows.get_mut(ident.index).unwrap();

        configure_surface(
            new_size,
            &window.surface,
            self.swap_chain_format,
            &gpu.device,
        );

        window.uniforms.mark_dirty(true);

        Ok(())
    }

    /// Render the contents of a window.
    pub fn render_window(&mut self, ident: WindowIdent, gpu: &GpuContext) -> anyhow::Result<()> {
        let window = self.windows.get_mut(ident.index).unwrap();

        let image = match &window.image {
            Some(x) => x,
            None => return Ok(()),
        };

        let frame = window
            .surface
            .get_current_texture()
            .expect("Failed to acquire next frame");

        let mut encoder = gpu.device.create_command_encoder(&Default::default());

        if window.uniforms.is_dirty() {
            window
                .uniforms
                .update_from(&gpu.device, &mut encoder, &window.calculate_uniforms());
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

        render_pass.set_pipeline(&gpu.window_pipeline);
        render_pass.set_bind_group(0, window.uniforms.bind_group(), &[]);
        render_pass.set_bind_group(1, image.bind_group(), &[]);
        render_pass.draw(0..6, 0..1);
        drop(render_pass);
        // --------------- RENDER PASS END ------------------- //

        gpu.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
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
