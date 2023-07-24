use crate::backend::util::GpuImage;
use crate::backend::util::{ToStd140, UniformsBuffer};
use crate::backend::window::Window;
use crate::backend::window::WindowUniforms;
use crate::background_thread::BackgroundThread;
use crate::error::CreateWindowError;
use crate::error::GetDeviceError;
use crate::error::InvalidWindowId;
use crate::error::NoSuitableAdapterFound;
use crate::event::{Event, WindowEvent};
use crate::ImageView;
use crate::WindowId;
use crate::WindowOptions;
use core::num::NonZeroU64;
use glam::Affine2;
use std::process::ExitCode;
use winit::event_loop::{EventLoop, EventLoopWindowTarget};

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

pub struct GpuContext {
    /// The wgpu device to use.
    pub device: wgpu::Device,

    /// The wgpu command queue to use.
    pub queue: wgpu::Queue,

    /// The bind group layout for the window specific bindings.
    pub window_bind_group_layout: wgpu::BindGroupLayout,

    /// The bind group layout for the image specific bindings.
    pub image_bind_group_layout: wgpu::BindGroupLayout,

    /// The render pipeline to use for windows.
    pub window_pipeline: wgpu::RenderPipeline,
}

/// The global context managing all windows and the main event loop.
pub struct Context {
    /// Marker to make context !Send.
    pub unsend: std::marker::PhantomData<*const ()>,

    /// The wgpu instance to create surfaces with.
    pub instance: wgpu::Instance,

    /// GPU related context that can not be initialized until we have a valid surface.
    pub gpu: Option<GpuContext>,

    /// The event loop to use.
    ///
    /// Running the event loop consumes it,
    /// so from that point on this field is `None`.
    pub event_loop: Option<EventLoop<()>>,

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

impl GpuContext {
    pub fn new(
        instance: &wgpu::Instance,
        swap_chain_format: wgpu::TextureFormat,
        surface: &wgpu::Surface,
    ) -> Result<Self, GetDeviceError> {
        let (device, queue) = futures::executor::block_on(get_device(instance, surface))?;
        device.on_uncaptured_error(Box::new(|error| {
            panic!("Unhandled WGPU error: {}", error);
        }));

        let window_bind_group_layout = create_window_bind_group_layout(&device);
        let image_bind_group_layout = create_image_bind_group_layout(&device);

        let vertex_shader =
            device.create_shader_module(wgpu::include_spirv!("../../shaders/shader.vert.spv"));
        let fragment_shader_unorm8 =
            device.create_shader_module(wgpu::include_spirv!("../../shaders/unorm8.frag.spv"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("show-image-pipeline-layout"),
            bind_group_layouts: &[&window_bind_group_layout, &image_bind_group_layout],
            push_constant_ranges: &[],
        });

        let window_pipeline = create_render_pipeline(
            &device,
            &pipeline_layout,
            &vertex_shader,
            &fragment_shader_unorm8,
            swap_chain_format,
        );

        Ok(Self {
            device,
            queue,
            window_bind_group_layout,
            image_bind_group_layout,
            window_pipeline,
        })
    }
}

impl Context {
    /// Create a new global context.
    ///
    /// You can theoreticlly create as many contexts as you want,
    /// but they must be run from the main thread and the [`run`](Self::run) function never returns.
    /// So it is not possible to *run* more than one context.
    pub fn new(swap_chain_format: wgpu::TextureFormat) -> Result<Self, GetDeviceError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: select_backend(),
            dx12_shader_compiler: Default::default(),
        });

        //let event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();
        let event_loop = winit::event_loop::EventLoop::new();

        Ok(Self {
            unsend: Default::default(),
            instance,
            gpu: None,
            event_loop: Some(event_loop),
            swap_chain_format,
            windows: Vec::new(),
            mouse_cache: Default::default(),
            exit_with_last_window: false,
            background_tasks: Vec::new(),
        })
    }
}

impl Context {
    /// Create a window.
    pub fn create_window(
        &mut self,
        event_loop: &EventLoopWindowTarget<()>,
        title: impl Into<String>,
        options: WindowOptions,
    ) -> Result<usize, CreateWindowError> {
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
    fn destroy_window(&mut self, window_id: WindowId) -> Result<(), InvalidWindowId> {
        let index = self
            .windows
            .iter()
            .position(|w| w.id() == window_id)
            .ok_or(InvalidWindowId { window_id })?;
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
    fn resize_window(
        &mut self,
        window_id: WindowId,
        new_size: glam::UVec2,
    ) -> Result<(), InvalidWindowId> {
        let window = self
            .windows
            .iter_mut()
            .find(|w| w.id() == window_id)
            .ok_or(InvalidWindowId { window_id })?;

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
    fn render_window(&mut self, window_id: WindowId) -> Result<(), InvalidWindowId> {
        let window = self
            .windows
            .iter_mut()
            .find(|w| w.id() == window_id)
            .ok_or(InvalidWindowId { window_id })?;

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
        event_loop: &EventLoopWindowTarget<()>,
        control_flow: &mut winit::event_loop::ControlFlow,
    ) {
        *control_flow = winit::event_loop::ControlFlow::Wait;

        // Split between Event<ContextFunction> and ContextFunction commands.
        let event = match super::event::map_nonuser_event(event) {
            Ok(event) => event,
            Err(function) => {
                panic!("idk bro");
            }
        };

        self.mouse_cache.handle_event(&event);

        // Convert to own event type.
        let mut event = match super::event::convert_winit_event(event, &self.mouse_cache) {
            Some(x) => x,
            None => return,
        };

        // If we have nothing more to do, clean the background tasks.
        if let Event::MainEventsCleared = &event {
            self.clean_background_tasks();
        }

        // Run window event handlers.
        // let run_context_handlers = match &mut event {
        //     Event::WindowEvent(event) => self.run_window_event_handlers(event, event_loop),
        //     _ => true,
        // };

        // Perform default actions for events.
        match event {
            Event::WindowEvent(WindowEvent::KeyboardInput(event)) => {
                // if event.input.state.is_pressed() && event.input.key_code == Some(event::VirtualKeyCode::S) {
                // 	let overlays = event.input.modifiers.alt();
                // 	let modifiers = event.input.modifiers & !event::ModifiersState::ALT;
                // 	if modifiers == event::ModifiersState::CTRL {
                // 		self.save_image_prompt(event.window_id, overlays);
                // 	} else if modifiers == event::ModifiersState::CTRL | event::ModifiersState::SHIFT {
                // 		self.save_image(event.window_id, overlays);
                // 	}
                // }
            }
            Event::WindowEvent(WindowEvent::Resized(event)) => {
                if event.size.x > 0 && event.size.y > 0 {
                    let _ = self.resize_window(event.window_id, event.size);
                }
            }
            Event::WindowEvent(WindowEvent::RedrawRequested(event)) => {
                let _ = self.render_window(event.window_id);
            }
            Event::WindowEvent(WindowEvent::CloseRequested(event)) => {
                let _ = self.destroy_window(event.window_id);
            }
            _ => {}
        }
    }

    /// Run window-specific event handlers.
    fn run_window_event_handlers(
        &mut self,
        event: &mut WindowEvent,
        event_loop: &EventLoop<()>,
    ) -> bool {
        let window_index = match self
            .windows
            .iter()
            .position(|x| x.id() == event.window_id())
        {
            Some(x) => x,
            None => return true,
        };

        let mut stop_propagation = false;
        let mut window_destroyed = false;

        !stop_propagation && !window_destroyed
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

fn select_power_preference() -> wgpu::PowerPreference {
    wgpu::PowerPreference::LowPower
}

/// Get a wgpu device to use.
async fn get_device(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface,
) -> Result<(wgpu::Device, wgpu::Queue), GetDeviceError> {
    // Find a suitable display adapter.
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: select_power_preference(),
        compatible_surface: Some(surface),
        force_fallback_adapter: false,
    });

    let adapter = adapter.await.ok_or(NoSuitableAdapterFound)?;

    // Create the logical device and command queue
    let device = adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("show-image"),
            limits: wgpu::Limits::default(),
            features: wgpu::Features::default(),
        },
        None,
    );

    let (device, queue) = device.await?;

    Ok((device, queue))
}

/// Create the bind group layout for the window specific bindings.
fn create_window_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("window_bind_group_layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            count: None,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(NonZeroU64::new(WindowUniforms::STD140_SIZE).unwrap()),
            },
        }],
    })
}

/// Create the bind group layout for the image specific bindings.
fn create_image_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("image_bind_group_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size:
                        Some(
                            NonZeroU64::new(
                                std::mem::size_of::<super::util::GpuImageUniforms>() as u64
                            )
                            .unwrap(),
                        ),
                },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            },
        ],
    })
}

/// Create a render pipeline with the specified device, layout, shaders and swap chain format.
fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    vertex_shader: &wgpu::ShaderModule,
    fragment_shader: &wgpu::ShaderModule,
    swap_chain_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("show-image-pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: vertex_shader,
            entry_point: "main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: fragment_shader,
            entry_point: "main",
            targets: &[Some(wgpu::ColorTargetState {
                format: swap_chain_format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
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
