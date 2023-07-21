use core::num::NonZeroU64;
use std::process::ExitCode;
use crate::backend::proxy::ContextFunction;
use crate::backend::util::GpuImage;
use crate::backend::util::{ToStd140, UniformsBuffer};
use crate::backend::window::Window;
use crate::backend::window::WindowUniforms;
use crate::background_thread::BackgroundThread;
use crate::error::CreateWindowError;
use crate::error::GetDeviceError;
use crate::error::InvalidWindowId;
use crate::error::NoSuitableAdapterFound;
use crate::event::{self, Event, EventHandlerControlFlow, WindowEvent};
use crate::ContextProxy;
use crate::ImageView;
use crate::backend::window::WindowHandle;
use crate::WindowId;
use crate::WindowOptions;
use glam::Affine2;

/// Internal shorthand type-alias for the correct [`winit::event_loop::EventLoop`].
///
/// Not for use in public APIs.
type EventLoop = winit::event_loop::EventLoop<ContextFunction>;

/// Internal shorthand for context event handlers.
///
/// Not for use in public APIs.
type DynContextEventHandler = dyn FnMut(&mut ContextHandle, &mut Event, &mut event::EventHandlerControlFlow);

/// Internal shorthand type-alias for the correct [`winit::event_loop::EventLoopWindowTarget`].
///
/// Not for use in public APIs.
type EventLoopWindowTarget = winit::event_loop::EventLoopWindowTarget<ContextFunction>;

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
	pub event_loop: Option<EventLoop>,

	/// A proxy object to clone for new requests.
	pub proxy: ContextProxy,

	/// The swap chain format to use.
	pub swap_chain_format: wgpu::TextureFormat,

	/// The windows.
	pub windows: Vec<Window>,

	/// Cache for mouse state.
	pub mouse_cache: super::mouse_cache::MouseCache,

	/// If true, exit the program when the last window closes.
	pub exit_with_last_window: bool,

	/// The global event handlers.
	pub event_handlers: Vec<Box<DynContextEventHandler>>,

	/// Background tasks, like saving images.
	pub background_tasks: Vec<BackgroundThread<()>>,
}

/// Handle to the global context.
///
/// You can interact with the global context through a [`ContextHandle`] only from the global context thread.
/// To interact with the context from a different thread, use a [`ContextProxy`].
pub struct ContextHandle<'a> {
	pub(crate) context: &'a mut Context,
	pub(crate) event_loop: &'a EventLoopWindowTarget,
}

impl GpuContext {
	pub fn new(instance: &wgpu::Instance, swap_chain_format: wgpu::TextureFormat, surface: &wgpu::Surface) -> Result<Self, GetDeviceError> {
		let (device, queue) = futures::executor::block_on(get_device(instance, surface))?;
		device.on_uncaptured_error(Box::new(|error| {
			panic!("Unhandled WGPU error: {}", error);
		}));

		let window_bind_group_layout = create_window_bind_group_layout(&device);
		let image_bind_group_layout = create_image_bind_group_layout(&device);

		let vertex_shader = device.create_shader_module(wgpu::include_spirv!("../../shaders/shader.vert.spv"));
		let fragment_shader_unorm8 = device.create_shader_module(wgpu::include_spirv!("../../shaders/unorm8.frag.spv"));

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
		let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor {
                backends: select_backend(),
                dx12_shader_compiler: Default::default()
            });
		let event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();
		let proxy = ContextProxy::new(event_loop.create_proxy(), std::thread::current().id());

		Ok(Self {
			unsend: Default::default(),
			instance,
			gpu: None,
			event_loop: Some(event_loop),
			proxy,
			swap_chain_format,
			windows: Vec::new(),
			mouse_cache: Default::default(),
			exit_with_last_window: false,
			event_handlers: Vec::new(),
			background_tasks: Vec::new(),
		})
	}

}

impl<'a> ContextHandle<'a> {
	/// Create a new context handle.
	fn new(context: &'a mut Context, event_loop: &'a EventLoopWindowTarget) -> Self {
		Self { context, event_loop }
	}

	/// Reborrow self with a shorter lifetime.
	pub(crate) fn reborrow(&mut self) -> ContextHandle {
		ContextHandle {
			context: self.context,
			event_loop: self.event_loop,
		}
	}

	/// Get a proxy for the context to interact with it from a different thread.
	///
	/// You should not use proxy objects from withing the global context thread.
	/// The proxy objects often wait for the global context to perform some action.
	/// Doing so from within the global context thread would cause a deadlock.
	pub fn proxy(&self) -> ContextProxy {
		self.context.proxy.clone()
	}

	/// Get a window handle for the given window ID.
	pub fn window(&mut self, window_id: WindowId) -> Result<WindowHandle, InvalidWindowId> {
		let index = self.context.windows.iter().position(|x| x.id() == window_id).ok_or(InvalidWindowId { window_id })?;
		Ok(WindowHandle::new(self.reborrow(), index, None))
	}

	/// Create a new window.
	pub fn create_window(&mut self, title: impl Into<String>, options: WindowOptions) -> Result<WindowHandle, CreateWindowError> {
		let index = self.context.create_window(self.event_loop, title, options)?;
		Ok(WindowHandle::new(self.reborrow(), index, None))
	}

	/// Join all background tasks and then exit the process.
	///
	/// If you use [`std::process::exit`], running background tasks may be killed.
	/// To ensure no data loss occurs, you should use this function instead.
	///
	/// Background tasks are spawned when an image is saved through the built-in Ctrl+S or Ctrl+Shift+S shortcut, or by user code.
	pub fn exit(&mut self, code: ExitCode) -> ! {
		self.context.exit(code);
	}
}

impl Context {
	/// Create a window.
	fn create_window(
		&mut self,
		event_loop: &EventLoopWindowTarget,
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
		let uniforms = UniformsBuffer::from_value(&gpu.device, &WindowUniforms::no_image(), &gpu.window_bind_group_layout);

		let window = Window {
			window,
			preserve_aspect_ratio: options.preserve_aspect_ratio,
			background_color: options.background_color,
			surface,
			uniforms,
			image: None,
			user_transform: Affine2::IDENTITY,
			overlays: Default::default(),
			event_handlers: Vec::new(),
		};

		self.windows.push(window);
		let index = self.windows.len() - 1;
		if options.default_controls {
			self.windows[index].event_handlers.push(Box::new(super::window::default_controls_handler));
		}
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
		GpuImage::from_data(name.into(), &gpu.device, &gpu.image_bind_group_layout, image)
	}

	/// Resize a window.
	fn resize_window(&mut self, window_id: WindowId, new_size: glam::UVec2) -> Result<(), InvalidWindowId> {
		let window = self
			.windows
			.iter_mut()
			.find(|w| w.id() == window_id)
			.ok_or(InvalidWindowId { window_id })?;

		let gpu = self.gpu.as_ref().unwrap();
		configure_surface(new_size, &window.surface, self.swap_chain_format, &gpu.device);
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
			&frame.texture.create_view(&wgpu::TextureViewDescriptor::default()),
		);
		for (_name, overlay) in &window.overlays {
			if overlay.visible {
				render_pass(
					&mut encoder,
					&gpu.window_pipeline,
					&window.uniforms,
					&overlay.image,
					None,
					&frame.texture.create_view(&wgpu::TextureViewDescriptor::default()),
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
		event: winit::event::Event<ContextFunction>,
		event_loop: &EventLoopWindowTarget,
		control_flow: &mut winit::event_loop::ControlFlow,
	) {
		*control_flow = winit::event_loop::ControlFlow::Wait;

		// Split between Event<ContextFunction> and ContextFunction commands.
		let event = match super::event::map_nonuser_event(event) {
			Ok(event) => event,
			Err(function) => {
				(function)(&mut ContextHandle::new(self, event_loop));
				return;
			},
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
		let run_context_handlers = match &mut event {
			Event::WindowEvent(event) => self.run_window_event_handlers(event, event_loop),
			_ => true,
		};

		// Run context event handlers.
		if run_context_handlers {
			self.run_event_handlers(&mut event, event_loop);
		}

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
			},
			Event::WindowEvent(WindowEvent::Resized(event)) => {
				if event.size.x > 0 && event.size.y > 0 {
					let _ = self.resize_window(event.window_id, event.size);
				}
			},
			Event::WindowEvent(WindowEvent::RedrawRequested(event)) => {
				let _ = self.render_window(event.window_id);
			},
			Event::WindowEvent(WindowEvent::CloseRequested(event)) => {
				let _ = self.destroy_window(event.window_id);
			},
			_ => {},
		}
	}

	/// Run global event handlers.
	pub fn run_event_handlers(&mut self, event: &mut Event, event_loop: &EventLoopWindowTarget) {
		use super::util::RetainMut;

		// Event handlers could potentially modify the list of event handlers.
		// Also, even if they couldn't we'd still need borrow self mutably multiple times to run the event handlers.
		// That's not allowed, of course, so temporarily swap the event handlers with a new vector.
		// When we've run all handlers, we add the new handlers to the original vector and place it back.
		// https://newfastuff.com/wp-content/uploads/2019/05/dVIkgAf.png
		let mut event_handlers = std::mem::take(&mut self.event_handlers);

		let mut stop_propagation = false;
		RetainMut::retain_mut(&mut event_handlers, |handler| {
			if stop_propagation {
				true
			} else {
				let mut context_handle = ContextHandle::new(self, event_loop);
				let mut control = EventHandlerControlFlow::default();
				(handler)(&mut context_handle, event, &mut control);
				stop_propagation = control.stop_propagation;
				!control.remove_handler
			}
		});

		let new_event_handlers = std::mem::take(&mut self.event_handlers);
		event_handlers.extend(new_event_handlers);
		self.event_handlers = event_handlers;
	}

	/// Run window-specific event handlers.
	fn run_window_event_handlers(&mut self, event: &mut WindowEvent, event_loop: &EventLoopWindowTarget) -> bool {
		use super::util::RetainMut;

		let window_index = match self.windows.iter().position(|x| x.id() == event.window_id()) {
			Some(x) => x,
			None => return true,
		};

		let mut event_handlers = std::mem::take(&mut self.windows[window_index].event_handlers);

		let mut stop_propagation = false;
		let mut window_destroyed = false;
		RetainMut::retain_mut(&mut event_handlers, |handler| {
			if window_destroyed || stop_propagation {
				true
			} else {
				let context_handle = ContextHandle::new(self, event_loop);
				let window_handle = WindowHandle::new(context_handle, window_index, Some(&mut window_destroyed));
				let mut control = EventHandlerControlFlow::default();
				(handler)(window_handle, event, &mut control);
				stop_propagation = control.stop_propagation;
				!control.remove_handler
			}
		});

		if !window_destroyed {
			let new_event_handlers = std::mem::take(&mut self.windows[window_index].event_handlers);
			event_handlers.extend(new_event_handlers);
			self.windows[window_index].event_handlers = event_handlers;
		}

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
async fn get_device(instance: &wgpu::Instance, surface: &wgpu::Surface) -> Result<(wgpu::Device, wgpu::Queue), GetDeviceError> {
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
					min_binding_size: Some(NonZeroU64::new(std::mem::size_of::<super::util::GpuImageUniforms>() as u64).unwrap()),
				},
			},
			wgpu::BindGroupLayoutEntry {
				binding: 1,
				visibility: wgpu::ShaderStages::FRAGMENT,
				count: None,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Storage {
						read_only: true,
					},
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
        view_formats: Default::default()
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
