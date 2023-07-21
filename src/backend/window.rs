use crate::Color;
use crate::ContextHandle;
use crate::ImageView;
use crate::WindowId;
use crate::WindowProxy;
use crate::backend::context::Context;
use crate::backend::util::GpuImage;
use crate::backend::util::UniformsBuffer;
use crate::event::EventHandlerControlFlow;
use crate::event::WindowEvent;
use glam::Vec3;
use glam::{Affine2, Vec2};
use indexmap::IndexMap;

/// Internal shorthand for window event handlers.
type DynWindowEventHandler = dyn FnMut(WindowHandle, &mut WindowEvent, &mut EventHandlerControlFlow);

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

	/// Overlays for the window.
	pub overlays: IndexMap<String, Overlay>,

	/// Transformation to apply to the image, in virtual window space.
	///
	/// Virtual window space goes from (0, 0) in the top left to (1, 1) in the bottom right.
	pub user_transform: Affine2,

	/// The event handlers for this specific window.
	pub event_handlers: Vec<Box<DynWindowEventHandler>>,
}

/// An overlay added to a window.
pub struct Overlay {
	/// The image to show.
	pub image: GpuImage,

	/// If true, show the overlay, otherwise do not.
	pub visible: bool,
}

/// Handle to a window.
///
/// A [`WindowHandle`] can be used to interact with a window from within the global context thread.
/// To interact with a window from another thread, you need a [`WindowProxy`].
pub struct WindowHandle<'a> {
	/// The context handle to use.
	context_handle: ContextHandle<'a>,

	/// The index of the window in [`Context::windows`].
	index: usize,
	/// Flag to signal to the handle creator that the window was destroyed.
	destroy_flag: Option<&'a mut bool>,
}

impl<'a> WindowHandle<'a> {
	/// Create a new window handle from a context handle and a window ID.
	pub fn new(context_handle: ContextHandle<'a>, index: usize, destroy_flag: Option<&'a mut bool>) -> Self {
		Self { context_handle, index, destroy_flag }
	}

	/// Get a reference to the context.
	fn context(&self) -> &Context {
		self.context_handle().context
	}

	/// Get a mutable reference to the context.
	///
	/// # Safety
	/// The current window may not be moved or removed through the returned reference.
	/// In practise, this means that you may not create or destroy any windows.
	unsafe fn context_mut(&mut self) -> &mut Context {
		self.context_handle.context
	}

	/// Get a reference to the window.
	fn window(&self) -> &Window {
		&self.context().windows[self.index]
	}

	/// Get a mutable reference to the window.
	fn window_mut(&mut self) -> &mut Window {
		let index = self.index;
		unsafe { &mut self.context_mut().windows[index] }
	}

	/// Get the window ID.
	pub fn id(&self) -> WindowId {
		self.window().id()
	}

	/// Get a proxy object for the window to interact with it from a different thread.
	///
	/// You should not use proxy objects from withing the global context thread.
	/// The proxy objects often wait for the global context to perform some action.
	/// Doing so from within the global context thread would cause a deadlock.
	pub fn proxy(&self) -> WindowProxy {
		WindowProxy::new(self.id(), self.context_handle.proxy())
	}

	/// Get a reference to the context handle.
	///
	/// If you need mutable access to the context, use [`release()`](Self::release) instead.
	pub fn context_handle(&self) -> &ContextHandle<'a> {
		&self.context_handle
	}

	/// Get the inner size of the window in physical pixels.
	///
	/// This returns the size of the window contents, excluding borders, the title bar and other decorations.
	pub fn inner_size(&self) -> glam::UVec2 {
		let size = self.window().window.inner_size();
		glam::UVec2::new(size.width, size.height)
	}

	/// Set the image to display on the window.
	pub fn set_image(&mut self, name: impl Into<String>, image: &ImageView) {
		let image = self.context().make_gpu_image(name, image);
		self.window_mut().image = Some(image);
		self.window_mut().uniforms.mark_dirty(true);
		self.window_mut().window.request_redraw();
	}

	/// Get the image transformation.
	///
	/// The image transformation is applied to the image and all overlays in virtual window space.
	///
	/// Virtual window space goes from `(0, 0)` in the top left corner of the window to `(1, 1)` in the bottom right corner.
	///
	/// This transformation does not include scaling introduced by the [`Self::preserve_aspect_ratio()`] property.
	/// Use [`Self::effective_transform()`] if you need that.
	pub fn transform(&self) -> Affine2 {
		self.window().user_transform
	}

	/// Set the image transformation to a value.
	///
	/// The image transformation is applied to the image and all overlays in virtual window space.
	///
	/// Virtual window space goes from `(0, 0)` in the top left corner of the window to `(1, 1)` in the bottom right corner.
	///
	/// This transformation should not include any scaling related to the [`Self::preserve_aspect_ratio()`] property.
	pub fn set_transform(&mut self, transform: Affine2) {
		self.window_mut().user_transform = transform;
		self.window_mut().uniforms.mark_dirty(true);
		self.window().window.request_redraw();
	}

	/// Pre-apply a transformation to the existing image transformation.
	///
	/// This is equivalent to:
	/// ```
	/// # use show_image::{glam::Affine2, WindowHandle};
	/// # fn foo(window: &mut WindowHandle, transform: Affine2) {
	/// window.set_transform(transform * window.transform())
	/// # }
	/// ```
	///
	/// See [`Self::set_transform`] for more information about the image transformation.
	pub fn pre_apply_transform(&mut self, transform: Affine2) {
		self.set_transform(transform * self.transform());
	}
}

/// Options for creating a new window.
#[derive(Debug, Clone)]
pub struct WindowOptions {
	/// Preserve the aspect ratio of the image when scaling.
	pub preserve_aspect_ratio: bool,

	/// The background color for the window.
	///
	/// This is used to color areas without image data if `preserve_aspect_ratio` is true.
	pub background_color: Color,

	/// Create the window hidden.
	///
	/// The window can manually be made visible at a later time.
	pub start_hidden: bool,

	/// The initial size of the window in pixel.
	///
	/// This may be ignored by some window managers.
	pub size: Option<[u32; 2]>,

	/// If true allow the window to be resized.
	///
	/// This may be ignored by some window managers.
	pub resizable: bool,

	/// Make the window borderless.
	///
	/// This may be ignored by some window managers.
	pub borderless: bool,

	/// Make the window fullscreen.
	///
	/// This may be ignored by some window managers.
	pub fullscreen: bool,

	/// If true, draw overlays on the image.
	///
	/// Defaults to true.
	pub overlays_visible: bool,

	/// If true, enable default mouse based controls for panning and zooming the image.
	///
	/// Defaults to true.
	pub default_controls: bool,
}

impl Default for WindowOptions {
	fn default() -> Self {
		Self::new()
	}
}

impl WindowOptions {
	/// Create new window options with default values.
	pub fn new() -> Self {
		Self {
			preserve_aspect_ratio: true,
			background_color: Color::black(),
			start_hidden: false,
			size: None,
			resizable: true,
			borderless: false,
			fullscreen: false,
			overlays_visible: true,
			default_controls: true,
		}
	}
}

impl Window {
	/// Get the window ID.
	pub fn id(&self) -> WindowId {
		self.window.id()
	}

	/// Recalculate the uniforms for the render pipeline from the window state.
	pub fn calculate_uniforms(&self) -> WindowUniforms {
		if let Some(image) = &self.image {
			let image_size = image.info().size.as_vec2();
			if !self.preserve_aspect_ratio {
				WindowUniforms::stretch(image_size)
					.pre_apply_transform(self.user_transform)
			} else {
				let window_size = glam::UVec2::new(self.window.inner_size().width, self.window.inner_size().height).as_vec2();
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

/// The window specific uniforms for the render pipeline.
#[derive(Debug, Copy, Clone)]
pub struct WindowUniforms {
	/// The transformation applied to the image.
	///
	/// With the identity transform, the image is stretched to the inner window size,
	/// without preserving the aspect ratio.
	pub transform: Affine2,

	/// The size of the image in pixels.
	pub image_size: Vec2,
}

impl WindowUniforms {
	pub fn no_image() -> Self {
		Self::stretch(Vec2::new(0.0, 0.0))
	}

	pub fn stretch(image_size: Vec2) -> Self {
		Self {
			transform: Affine2::IDENTITY,
			image_size,
		}
	}

	pub fn fit(window_size: Vec2, image_size: Vec2) -> Self {
		let ratios = image_size / window_size;

		let w;
		let h;
		if ratios.x >= ratios.y {
			w = 1.0;
			h = ratios.y / ratios.x;
		} else {
			w = ratios.x / ratios.y;
			h = 1.0;
		}

		let transform = Affine2::from_scale_angle_translation(Vec2::new(w, h), 0.0, 0.5 * Vec2::new(1.0 - w, 1.0 - h));
		Self {
			transform,
			image_size,
		}
	}

	/// Pre-apply a transformation.
	pub fn pre_apply_transform(mut self, transform: Affine2) -> Self {
		self.transform = transform * self.transform;
		self
	}
}

#[repr(C, align(8))]
#[derive(Debug, Copy, Clone)]
struct Vec2A8 {
	pub x: f32,
	pub y: f32,
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone)]
struct Vec3A16 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Mat3x3 {
	pub cols: [Vec3A16; 3]
}

impl Vec2A8 {
	pub const fn new(x: f32, y: f32) -> Self {
		Self { x, y }
	}
}

impl Vec3A16 {
	pub const fn new(x: f32, y: f32, z: f32) -> Self {
		Self { x, y, z }
	}
}

impl Mat3x3 {
	pub const fn new(col0: Vec3A16, col1: Vec3A16, col2: Vec3A16) -> Self {
		Self {
			cols: [col0, col1, col2],
		}
	}
}

impl From<Vec2> for Vec2A8 {
	fn from(other: Vec2) -> Self {
		Self::new(other.x, other.y)
	}
}

impl From<Vec3> for Vec3A16 {
	fn from(other: Vec3) -> Self {
		Self::new(other.x, other.y, other.z)
	}
}

impl From<Affine2> for Mat3x3 {
	fn from(other: Affine2) -> Self {
		let x_axis = other.matrix2.x_axis;
		let y_axis = other.matrix2.y_axis;
		let z_axis = other.translation;
		Self::new(
			Vec3A16::new(x_axis.x, x_axis.y, 0.0),
			Vec3A16::new(y_axis.x, y_axis.y, 0.0),
			Vec3A16::new(z_axis.x, z_axis.y, 1.0),
		)
	}
}

/// Window specific unfiforms, layout compatible with glsl std140.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct WindowUniformsStd140 {
	image_size: Vec2A8,
	transform: Mat3x3,
}

unsafe impl crate::backend::util::ToStd140 for WindowUniforms {
	type Output = WindowUniformsStd140;

	fn to_std140(&self) -> Self::Output {
		Self::Output {
			image_size: self.image_size.into(),
			transform: self.transform.into(),
		}
	}
}

/// Event handler that implements the default controls.
pub(super) fn default_controls_handler(mut window: WindowHandle, event: &mut crate::event::WindowEvent, _control_flow: &mut crate::event::EventHandlerControlFlow) {
	match event {
		WindowEvent::MouseWheel(event) => {
			let delta = match event.delta {
				winit::event::MouseScrollDelta::LineDelta(_x, y) => y,
				winit::event::MouseScrollDelta::PixelDelta(delta) => delta.y as f32 / 20.0,
			};
			let scale = 1.1f32.powf(delta);

			let origin = event.position
				.map(|pos| pos / window.inner_size().as_vec2())
				.unwrap_or_else(|| glam::Vec2::new(0.5, 0.5));
			let transform = glam::Affine2::from_scale_angle_translation(glam::Vec2::splat(scale), 0.0, origin - scale * origin);
			window.pre_apply_transform(transform);
		},
		WindowEvent::MouseMove(event) => {
			if event.buttons.is_pressed(crate::event::MouseButton::Left) {
				let translation = (event.position - event.prev_position) / window.inner_size().as_vec2();
				window.pre_apply_transform(Affine2::from_translation(translation));
			}
		},
		_ => (),
	}
}
