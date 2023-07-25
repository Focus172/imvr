use crate::events::Request;

use crate::{
    gpu::{GpuContext, UniformsBuffer},
    window::{Window, WindowUniforms},
};
use anyhow::anyhow;
use glam::Affine2;
use std::collections::VecDeque;
use winit::window::WindowButtons;
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::KeyCode,
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
        let mut window = winit::window::WindowBuilder::new()
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

        self.windows.push(window);
        let index = self.windows.len() - 1;

        Ok((index, gpu))
    }

    /// Resize a window.
    pub fn resize_window(
        &mut self,
        window_id: WindowId,
        new_size: glam::UVec2,
        gpu: &GpuContext,
    ) -> anyhow::Result<()> {
        let window = self
            .windows
            .iter_mut()
            .find(|w| w.id() == window_id)
            .ok_or(anyhow!("Invalid window id: {:?}", window_id))?;

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
    pub fn render_window(&mut self, window_id: WindowId, gpu: &GpuContext) -> anyhow::Result<()> {
        let window = self
            .windows
            .iter_mut()
            .find(|w| w.id() == window_id)
            .ok_or(anyhow!("Invalid window id: {:?}", window_id))?;

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

    /// Handle an event from the event loop and produce a list of events to
    /// append to the main list
    pub fn handle_event(
        &mut self,
        event: winit::event::Event<()>,
        _event_loop: &EventLoopWindowTarget<()>,
    ) -> Option<Request> {
        // self.mouse_cache.handle_event(&event);

        // Run window event handlers.
        // let run_context_handlers = match &mut event {
        //     Event::WindowEvent(event) => self.run_window_event_handlers(event, event_loop),
        //     _ => true,
        // };

        // Perform default actions for events.
        match event {
            Event::WindowEvent { window_id, event } => match event {
                WindowEvent::Resized(new_size) => Some(Request::Multiple(vec![
                    Request::Resize {
                        size: glam::UVec2::new(new_size.width, new_size.height),
                        window_id,
                    },
                    Request::Redraw { window_id },
                ])),
                WindowEvent::KeyboardInput { event, .. } => self.handle_keypress(event),
                WindowEvent::CloseRequested => Some(Request::CloseWindow { window_id }),
                _ => None,
            },
            Event::RedrawRequested(window_id) => Some(Request::Redraw { window_id }),
            // If we have nothing more to do, clean the background tasks.
            // Event::MainEventsCleared => self.background_tasks.retain(|task| !task.is_done()),
            _ => None,
        }
    }

    fn handle_keypress(&mut self, key: KeyEvent) -> Option<Request> {
        match (key.physical_key, key.state, key.repeat) {
            (KeyCode::KeyQ, ElementState::Pressed, _) => Some(Request::Exit),
            (KeyCode::KeyL, ElementState::Pressed, _) => {
                log::warn!("This key press is not handled rn");
                // self.request_queue.push_back(Request::NextImage)
                None
            }
            (_, _, _) => None,
        }
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
