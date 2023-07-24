use crate::backend::util::GpuImage;
use crate::backend::util::UniformsBuffer;
use crate::backend::window::Window;
use crate::backend::window::WindowUniforms;
use crate::background_thread::BackgroundThread;
use crate::ImageView;
use crate::WindowId;
use crate::WindowOptions;
use anyhow::anyhow;
use glam::Affine2;
use std::process::ExitCode;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopWindowTarget};

use super::gpu::GpuContext;

impl From<crate::Color> for wgpu::Color {
    fn from(other: crate::Color) -> Self {
        Self {
            r: other.red,
            g: other.green,
            b: other.blue,
            a: other.alpha,
        }
    }
}

/// The global context managing all windows and the main event loop.
pub struct Context {
    /// Marker to make context !Send.
    pub unsend: std::marker::PhantomData<*const ()>,

    /// The wgpu instance to create surfaces with.
    pub instance: wgpu::Instance,

    /// GPU related context that can not be initialized until we have a valid surface.
    pub gpu: Option<GpuContext>,

    /// The swap chain format to use.
    pub swap_chain_format: wgpu::TextureFormat,

    /// The windows.
    pub windows: Vec<Window>,

    /// Cache for mouse state.
    pub mouse_cache: super::mouse_cache::MouseCache,

    /// If true, exit the program when the last window closes.
    pub exit_with_last_window: bool,

    /// Background tasks, like saving images.
    pub background_tasks: Vec<BackgroundThread<()>>,
}

impl Context {
    pub fn new(swap_chain_format: wgpu::TextureFormat) -> anyhow::Result<(Self, EventLoop<()>)> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: select_backend(),
            dx12_shader_compiler: Default::default(),
        });

        // Consider adding user events
        let event_loop = winit::event_loop::EventLoop::new();

        Ok((
            Self {
                unsend: Default::default(),
                instance,
                gpu: None,
                swap_chain_format,
                windows: Vec::new(),
                mouse_cache: Default::default(),
                exit_with_last_window: false,
                background_tasks: Vec::new(),
            },
            event_loop,
        ))
    }

    /// Create a window.
    pub fn create_window(
        &mut self,
        event_loop: &EventLoopWindowTarget<()>,
        title: impl Into<String>,
        options: WindowOptions,
    ) -> anyhow::Result<usize> {
        let fullscreen = if options.fullscreen {
            Some(winit::window::Fullscreen::Borderless(None))
        } else {
            None
        };
        let mut window = winit::window::WindowBuilder::new()
            .with_title(title)
            .with_visible(!options.start_hidden)
            .with_resizable(options.resizable)
            .with_decorations(!options.borderless)
            .with_fullscreen(fullscreen);

        if let Some(size) = options.size {
            window = window.with_inner_size(winit::dpi::PhysicalSize::new(size[0], size[1]));
        }

        let window = window.build(event_loop)?;
        let surface = unsafe { self.instance.create_surface(&window) }.unwrap();

        let gpu = match &self.gpu {
            Some(x) => x,
            None => {
                let gpu = GpuContext::new(&self.instance, self.swap_chain_format, &surface)?;
                self.gpu.insert(gpu)
            }
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
            preserve_aspect_ratio: options.preserve_aspect_ratio,
            background_color: options.background_color,
            surface,
            uniforms,
            image: None,
            user_transform: Affine2::IDENTITY,
            overlays: Default::default(),
            // event_handlers: Vec::new(),
        };

        self.windows.push(window);
        let index = self.windows.len() - 1;
        Ok(index)
    }

    /// Destroy a window.
    #[allow(unused)]
    fn destroy_window(&mut self, window_id: WindowId) -> anyhow::Result<()> {
        let index = self
            .windows
            .iter()
            .position(|w| w.id() == window_id)
            .ok_or(anyhow!("Invalid window id: {:?}", window_id))?;
        self.windows.remove(index);
        Ok(())
    }

    /// Upload an image to the GPU.
    pub fn make_gpu_image(&self, name: impl Into<String>, image: &ImageView) -> GpuImage {
        let gpu = self.gpu.as_ref().unwrap();
        GpuImage::from_data(
            name.into(),
            &gpu.device,
            &gpu.image_bind_group_layout,
            image,
        )
    }

    /// Resize a window.
    #[allow(unused)]
    fn resize_window(&mut self, window_id: WindowId, new_size: glam::UVec2) -> anyhow::Result<()> {
        let window = self
            .windows
            .iter_mut()
            .find(|w| w.id() == window_id)
            .ok_or(anyhow!("Invalid window id: {:?}", window_id))?;

        let gpu = self.gpu.as_ref().unwrap();
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
    fn render_window(&mut self, window_id: WindowId) -> anyhow::Result<()> {
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

        let gpu = self.gpu.as_ref().unwrap();
        let mut encoder = gpu.device.create_command_encoder(&Default::default());

        if window.uniforms.is_dirty() {
            window
                .uniforms
                .update_from(&gpu.device, &mut encoder, &window.calculate_uniforms());
        }

        render_pass(
            &mut encoder,
            &gpu.window_pipeline,
            &window.uniforms,
            image,
            Some(window.background_color),
            &frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
        );
        for (_name, overlay) in &window.overlays {
            if overlay.visible {
                render_pass(
                    &mut encoder,
                    &gpu.window_pipeline,
                    &window.uniforms,
                    &overlay.image,
                    None,
                    &frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                );
            }
        }
        gpu.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }

    /// Handle an event from the event loop.
    pub fn handle_event(
        &mut self,
        event: winit::event::Event<()>,
        _event_loop: &EventLoopWindowTarget<()>,
    ) {
        self.mouse_cache.handle_event(&event);

        // If we have nothing more to do, clean the background tasks.
        if let winit::event::Event::MainEventsCleared = &event {
            self.clean_background_tasks();
        }

        // Run window event handlers.
        // let run_context_handlers = match &mut event {
        //     Event::WindowEvent(event) => self.run_window_event_handlers(event, event_loop),
        //     _ => true,
        // };

        // Perform default actions for events.
        match event {
            Event::WindowEvent { window_id, event } => match event {
                WindowEvent::Resized(new_size) => {
                    if new_size.width > 0 && new_size.height > 0 {
                        let size = glam::UVec2::from_array([new_size.width, new_size.height]);
                        let _ = self.resize_window(window_id, size);
                    }
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    // if event.input.state.is_pressed() && event.input.key_code == Some(event::VirtualKeyCode::S) {
                    // 	let overlays = event.input.modifiers.alt();
                    // 	let modifiers = event.input.modifiers & !event::ModifiersState::ALT;
                    // 	if modifiers == event::ModifiersState::CTRL {
                    // 		self.save_image_prompt(event.window_id, overlays);
                    // 	} else if modifiers == event::ModifiersState::CTRL | event::ModifiersState::SHIFT {
                    // 		self.save_image(event.window_id, overlays);
                    // 	}
                }
                WindowEvent::CloseRequested => {
                    let _ = self.destroy_window(window_id);
                }
                _ => {}
            },
            Event::RedrawRequested(window_id) => self.render_window(window_id).unwrap(),
            _ => {}
        }
    }

    /// Clean-up finished background tasks.
    fn clean_background_tasks(&mut self) {
        self.background_tasks.retain(|task| !task.is_done());
    }

    /// Join all background tasks.
    fn join_background_tasks(&mut self) {
        for task in std::mem::take(&mut self.background_tasks) {
            task.join().unwrap();
        }
    }

    /// Join all background tasks and then exit the process.
    pub fn exit(&mut self, code: ExitCode) -> ! {
        self.join_background_tasks();
        code.exit_process()
    }
}

fn select_backend() -> wgpu::Backends {
    wgpu::Backends::PRIMARY
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

/// Perform a render pass of an image.
fn render_pass(
    encoder: &mut wgpu::CommandEncoder,
    render_pipeline: &wgpu::RenderPipeline,
    window_uniforms: &UniformsBuffer<WindowUniforms>,
    image: &GpuImage,
    clear: Option<crate::Color>,
    target: &wgpu::TextureView,
) {
    let load = match clear {
        Some(color) => wgpu::LoadOp::Clear(color.into()),
        None => wgpu::LoadOp::Load,
    };

    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("render-image"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
            resolve_target: None,
            ops: wgpu::Operations { load, store: true },
        })],
        depth_stencil_attachment: None,
    });

    render_pass.set_pipeline(render_pipeline);
    render_pass.set_bind_group(0, window_uniforms.bind_group(), &[]);
    render_pass.set_bind_group(1, image.bind_group(), &[]);
    render_pass.draw(0..6, 0..1);
    drop(render_pass);
}
