use crate::ContextHandle;
use crate::image::Image;
use crate::image::AsImageView;
use crate::backend::window::WindowHandle;
use crate::WindowId;
use crate::error::{InvalidWindowId, SetImageError};
use crate::oneshot;

use std::process::ExitCode;

/// Proxy object to interact with a window from a user thread.
///
/// The proxy object only exposes a small subset of the functionality of a window.
/// However, you can use [`run_function()`][Self::run_function]
/// to get access to the underlying [`WindowHandle`] from the context thread.
/// With [`run_function_wait()`][Self::run_function_wait`] you can also get the return value of the function back:
///
/// ```no_run
/// # fn foo(window_proxy: show_image::WindowProxy) -> Result<(), show_image::error::InvalidWindowId> {
/// let inner_size = window_proxy.run_function_wait(|window| window.inner_size())?;
/// # Ok(())
/// # }
/// ```
///
/// You should not use proxy objects from withing the global context thread.
/// The proxy objects often wait for the global context to perform some action.
/// Doing so from within the global context thread would cause a deadlock.
#[derive(Clone)]
pub struct WindowProxy {
	window_id: WindowId,
	context_proxy: ContextProxy,
}

/// Proxy object to interact with the global context from a user thread.
///
/// You should not use proxy objects from withing the global context thread.
/// The proxy objects often wait for the global context to perform some action.
/// Doing so from within the global context thread would cause a deadlock.
#[derive(Clone)]
pub struct ContextProxy {
	event_loop: EventLoopProxy,
	context_thread: std::thread::ThreadId,
}

/// Dynamic function that can be run by the global context.
pub type ContextFunction = Box<dyn FnOnce(&mut ContextHandle) + Send>;

/// Internal shorthand for the correct `winit::event::EventLoopProxy`.
///
/// Not for use in public APIs.
type EventLoopProxy = winit::event_loop::EventLoopProxy<ContextFunction>;

impl ContextProxy {
	/// Wrap an [`EventLoopProxy`] in a [`ContextProxy`].
	pub fn new(event_loop: EventLoopProxy, context_thread: std::thread::ThreadId) -> Self {
		Self {
			event_loop,
			context_thread,
		}
	}

	/// Post a function for execution in the context thread without waiting for it to execute.
	///
	/// This function returns immediately, without waiting for the posted function to start or complete.
	/// If you want to get a return value back from the function, use [`Self::run_function_wait`] instead.
	///
	/// *Note:*
	/// You should not post functions to the context thread that block for a long time.
	/// Doing so will block the event loop and will make the windows unresponsive until the event loop can continue.
	/// Consider using [`Self::run_background_task`] for long blocking tasks instead.
	///
	/// # Panics
	/// This function will panic if called from within the context thread.
	pub fn run_function<F>(&self, function: F)
	where
		F: 'static + FnOnce(&mut ContextHandle) + Send,
	{
		let function = Box::new(function);
		if self.event_loop.send_event(function).is_err() {
			panic!("global context stopped running but somehow the process is still alive");
		}
	}

	/// Post a function for execution in the context thread and wait for the return value.
	///
	/// If you do not need a return value from the posted function,
	/// you can use [`Self::run_function`] to avoid blocking the calling thread until it completes.
	///
	/// *Note:*
	/// You should not post functions to the context thread that block for a long time.
	/// Doing so will block the event loop and will make the windows unresponsive until the event loop can continue.
	/// Consider using [`Self::run_background_task`] for long blocking tasks instead.
	///
	/// # Panics
	/// This function will panic if called from within the context thread.
	pub fn run_function_wait<F, T>(&self, function: F) -> T
	where
		F: FnOnce(&mut ContextHandle) -> T + Send + 'static,
		T: Send + 'static,
	{
		self.assert_thread();

		let (result_tx, result_rx) = oneshot::channel();
		self.run_function(move |context| result_tx.send((function)(context)));
		result_rx.recv()
			.expect("global context failed to send function return value back, which can only happen if the event loop stopped, but that should also kill the process")
	}

	/// Join all background tasks and then exit the process.
	///
	/// If you use [`std::process::exit`], running background tasks may be killed.
	/// To ensure no data loss occurs, you should use this function instead.
	///
	/// Background tasks are spawned when an image is saved through the built-in Ctrl+S or Ctrl+Shift+S shortcut, or by user code.
	///
	/// # Panics
	/// This function will panic if called from within the context thread.
	pub fn exit(&self, code: ExitCode) -> ! {
		self.assert_thread();
		self.run_function(move |context| context.exit(code));
		loop {
			std::thread::park();
		}
	}

	/// Check that the current thread is not running the context event loop.
	///
	/// # Panics
	/// This function will panic if called from within the context thread.
	#[track_caller]
	fn assert_thread(&self) {
		if std::thread::current().id() == self.context_thread {
			panic!("ContextProxy used from within the context thread, which would cause a deadlock. Use ContextHandle instead.");
		}
	}
}

impl WindowProxy {
	/// Create a new window proxy from a context proxy and a window ID.
	pub fn new(window_id: WindowId, context_proxy: ContextProxy) -> Self {
		Self { window_id, context_proxy }
	}


	/// Set the displayed image of the window.
	///
	/// The real work is done in the context thread.
	/// This function blocks until the context thread has performed the action.
	///
	/// Note that you can not change the overlays with this function.
	/// To modify those, you can use [`Self::run_function`] or [`Self::run_function_wait`]
	/// to get access to the [`WindowHandle`].
	///
	/// # Panics
	/// This function will panic if called from within the context thread.
	pub fn set_image(&self, name: impl Into<String>, image: impl Into<Image>) -> Result<(), SetImageError> {
		let name = name.into();
		let image = image.into();
		self.run_function_wait(move |mut window| -> Result<(), SetImageError> {
			window.set_image(name, &image.as_image_view()?);
			Ok(())
		})?
	}

	/// Post a function for execution in the context thread and wait for the return value.
	///
	/// If you do not need a return value from the posted function,
	/// you can use [`Self::run_function`] to avoid blocking the calling thread until it completes.
	///
	/// *Note:*
	/// You should not use this to post functions that block for a long time.
	/// Doing so will block the event loop and will make the windows unresponsive until the event loop can continue.
	/// Consider using [`self.context_proxy().run_background_task(...)`][ContextProxy::run_background_task] for long blocking tasks instead.
	///
	/// # Panics
	/// This function will panic if called from within the context thread.
	pub fn run_function_wait<F, T>(&self, function: F) -> Result<T, InvalidWindowId>
	where
		F: FnOnce(WindowHandle) -> T + Send + 'static,
		T: Send + 'static,
	{
		let window_id = self.window_id;
		self.context_proxy.run_function_wait(move |context| {
			let window = context.window(window_id)?;
			Ok(function(window))
		})
	}
}
