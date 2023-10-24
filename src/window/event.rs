use winit::event::WindowEvent;

use crate::{prelude::*, WinitEvent};

impl TryFrom<WinitEvent> for Request {
    // type Error = res::eyre::Report;
    type Error = ();

    fn try_from(value: WinitEvent) -> std::result::Result<Self, Self::Error> {
        match value {
            WinitEvent::WindowEvent { window_id, event } => match event {
                WindowEvent::Resized(new_size) => Ok(Request::Multiple(vec![
                    Request::resize(new_size, window_id),
                    Request::redraw(window_id),
                ])),
                WindowEvent::KeyboardInput { event, .. } => crate::events::parse_key(event.into()),
                WindowEvent::CloseRequested => Ok(Request::close(window_id)),
                WindowEvent::Destroyed => Ok(Request::close(window_id)),
                WindowEvent::RedrawRequested => Ok(Request::redraw(window_id)),
                // WindowEvent::Focused(did) => todo!(),
                _ => Err(()),
            },

            // we only care what happens within the window
            WinitEvent::DeviceEvent { .. } => Err(()),
            // Where startup code would be added
            WinitEvent::NewEvents(_) => Err(()),
            WinitEvent::UserEvent(_) => todo!(),
            WinitEvent::Suspended => Err(()),
            WinitEvent::Resumed => Err(()),
            WinitEvent::AboutToWait => Err(()),
            WinitEvent::LoopExiting => Err(()),
            WinitEvent::MemoryWarning => unimplemented!(),
        }
    }
}

// self.mouse_cache.handle_event(&event);
