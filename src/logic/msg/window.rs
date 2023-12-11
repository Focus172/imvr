use crate::prelude::*;

use super::{Msg, ReturnAddress};
use crate::logic::event::device::Key;
use ext::glam::UVec2;
use ext::parse::MoveIt;
use winit::window::WindowId;

/// A message that closely resemblems the final Requested.
/// Must be [`Send`] and relativly small on the stack
#[rustfmt::skip]
#[derive(Debug)]
pub enum WindowMsg {
    Many(Vec<WindowMsg>),
    ShowImage { image: image::DynamicImage, id: WindowId },
    OpenWindow { resp: ReturnAddress },
    CloseWindow { id: WindowId },
    Resize { size: UVec2, id: WindowId },
    WindowRedraw { id: WindowId },
    Exit,
}

impl Msg {
    pub fn as_window(&mut self) -> Option<WindowMsg> {
        match self {
            Msg::ShowImage { path, id } => {
                let id = id.as_id()?.into();
                let image = image::open(path).unwrap();
                Some(WindowMsg::ShowImage { image, id })
            }
            Msg::OpenWindow { resp } => {
                let resp = resp.take()?;
                Some(WindowMsg::OpenWindow { resp })
            }
        }
    }
}

impl SomeFrom<winit::event::Event<WindowMsg>> for WindowMsg {
    fn some_from(value: winit::event::Event<WindowMsg>) -> Option<Self> {
        use winit::event::Event as W;
        use winit::event::StartCause as SrtC;
        use winit::event::WindowEvent as We;

        match value {
            W::WindowEvent { window_id, event } => match event {
                We::Resized(size) => Some(WindowMsg::Resize {
                    id: window_id,
                    size: size.move_it(|s| UVec2::new(s.width, s.height)),
                }),
                We::Moved(_) => todo!(),
                We::CloseRequested => Some(WindowMsg::CloseWindow { id: window_id }),
                We::Destroyed => Some(WindowMsg::CloseWindow { id: window_id }),
                We::Focused(_) => None,
                We::KeyboardInput { event, .. } => {
                    Key::some_from(event).and_then(SomeInto::some_into)
                }
                We::RedrawRequested => Some(WindowMsg::WindowRedraw { id: window_id }),
                We::ScaleFactorChanged { .. } => None,
                We::ModifiersChanged(_) => None,
                We::CursorMoved { .. } => None,
                We::CursorEntered { .. } => None,
                We::CursorLeft { .. } => None,
                We::MouseWheel { .. } => None,
                We::MouseInput { .. } => None,
                We::Ime(_) => None,
                e => unimplemented!("event not handled yet: {e:?}"),
            },
            // TODO: have some init code ran
            W::NewEvents(SrtC::Init) => None,
            W::NewEvents(_) => None,

            W::UserEvent(e) => Some(e),
            W::LoopExiting => Some(WindowMsg::Exit),

            W::Suspended => None,
            W::Resumed => None,
            W::AboutToWait => None,

            // We dont care to know about specific implimitaion details of how our events
            // get to use so these can be ignored
            W::DeviceEvent { .. } => None,

            W::MemoryWarning => unimplemented!(),
        }
    }
}
