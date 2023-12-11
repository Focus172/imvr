use crate::prelude::*;

use crate::window::Window;
use crate::ImvrEventLoopHandle;
use winit::window::WindowId;

#[derive(Debug)]
pub enum GlobalContextError {
    Fatal,
    SendError,
    NoMatchingWindow(WindowId),
}

impl GlobalContextError {
    /// Returns true if this error should hault execution
    pub fn is_fatal(&self) -> bool {
        // TODO: add more variants as they come
        matches!(self, Self::Fatal)
    }
}

impl fmt::Display for GlobalContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GlobalContextError::Fatal => f.write_str("fatal error, see trace"),
            GlobalContextError::SendError => f.write_str("unable to send data back to requester"),
            GlobalContextError::NoMatchingWindow(id) => {
                write!(f, "no matching window for id {id:?}")
            }
        }
    }
}
impl Context for GlobalContextError {}

/// The Global Context managing the windows and msgs to them
#[derive(Debug, Default)]
pub struct GlobalContext {
    /// The wgpu instance to create surfaces with.
    pub instance: wgpu::Instance,

    /// The windows.
    pub windows: Vec<Window>,
}

impl GlobalContext {
    /// Creates a new global context
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle(
        &mut self,
        msg: WindowMsg,
        evwt: &ImvrEventLoopHandle,
    ) -> Result<(), GlobalContextError> {
        use WindowMsg as W;
        match msg {
            W::Many(reqs) => {
                for req in reqs {
                    self.handle(req, evwt)?;
                }
            }
            W::ShowImage { image, id } => self.get_window_mut(id)?.set_image(image),
            W::Exit => {
                // TODO: join all the processing threads
                evwt.exit();
            }
            W::Resize { size, id } => self.get_window_mut(id)?.resize(size),
            W::WindowRedraw { id } => self
                .get_window_mut(id)?
                .render()
                .change_context(GlobalContextError::Fatal)?,
            W::OpenWindow { resp } => {
                log::debug!("imvr: creating window");

                let window = Window::new("image", evwt, &self.instance)
                    .attach_printable("unable to make a new window")
                    .change_context(GlobalContextError::Fatal)?;

                let id = window.id().into();
                self.windows.push(window);

                log::info!("imvr: created window {}", id);

                resp.send(id)
                    .attach_printable_lazy(|| format!("Could not send id ({id}) on channel."))
                    .change_context(GlobalContextError::SendError)?;
            }
            W::CloseWindow { id } => {
                let index = self
                    .windows
                    .iter()
                    .enumerate()
                    .find(|(_, w)| w.id() == id)
                    .ok_or(Report::new(GlobalContextError::NoMatchingWindow(id)))
                    .attach_printable("cant remove window")?
                    .0;
                log::debug!("closing window {:?}", id);

                let window = self.windows.swap_remove(index);
                // TODO: do clean up the window
                drop(window);

                if self.windows.is_empty() {
                    evwt.exit()
                }
            }
        }
        Ok(())
    }

    #[inline]
    pub fn get_window_mut(&mut self, id: WindowId) -> Result<&mut Window, GlobalContextError> {
        self.windows
            .iter_mut()
            .find(|window| window.id() == id)
            .ok_or(Report::new(GlobalContextError::NoMatchingWindow(id)))
    }

    #[inline]
    pub fn get_window(&self, id: WindowId) -> Result<&Window, GlobalContextError> {
        self.windows
            .iter()
            .find(|window| window.id() == id)
            .ok_or(Report::new(GlobalContextError::NoMatchingWindow(id)))
    }
}
