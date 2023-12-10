use crate::logic::msg::ReturnAddress;
use crate::{logic::event::device::Key, prelude::*};
use ext::glam::UVec2;
use ext::parse::MoveIt;
use winit::window::WindowId;

#[derive(Debug)]
#[rustfmt::skip]
pub enum WindowRequest {
    Many(Vec<WindowRequest>),
    ShowImage { image: image::DynamicImage, id: WindowId },
    OpenWindow { resp: ReturnAddress },
    CloseWindow { window_id: u64 },
    Exit,
    Resize { size: UVec2, window_id: u64 },
    WindowRedraw { id: u64 },
}

impl SomeFrom<winit::event::Event<WindowMsg>> for WindowRequest {
    fn some_from(value: winit::event::Event<WindowMsg>) -> Option<Self> {
        use winit::event::Event as W;
        use winit::event::WindowEvent as We;
        match value {
            W::NewEvents(_start_cause) => None,
            W::WindowEvent { window_id, event } => match event {
                winit::event::WindowEvent::Resized(size) => Some(WindowRequest::Resize {
                    window_id: window_id.into(),
                    size: size.move_it(|s| UVec2::new(s.width, s.height)),
                }),
                winit::event::WindowEvent::Moved(_) => todo!(),
                winit::event::WindowEvent::CloseRequested => Some(WindowRequest::CloseWindow {
                    window_id: window_id.into(),
                }),
                winit::event::WindowEvent::Destroyed => todo!(),
                winit::event::WindowEvent::Focused(_) => None,
                winit::event::WindowEvent::KeyboardInput { event, .. } => {
                    Key::some_from(event).and_then(SomeInto::some_into)
                }
                winit::event::WindowEvent::RedrawRequested => Some(WindowRequest::WindowRedraw {
                    id: window_id.into(),
                }),
                We::ScaleFactorChanged { .. } => None,
                We::ModifiersChanged(_) => None,
                We::CursorMoved { .. } => None,
                We::CursorEntered { .. } => None,
                We::CursorLeft { .. } => None,
                We::MouseWheel { .. } => None,
                We::MouseInput { .. } => None,
                We::Ime(_) => None,
                e => {
                    dbg!(e);
                    unimplemented!()
                }
            },
            // We dont care to know about specific implimitaion details of how our events
            // get to use so these can be ignored
            W::DeviceEvent { .. } => None,
            W::UserEvent(e) => Some(e.into()),
            W::Suspended => {
                log::info!("yeilding to schedualer.");
                None
            }
            W::Resumed => {
                log::info!("And we are back.");
                None
            }
            W::AboutToWait => None,
            W::LoopExiting => Some(WindowRequest::Exit),
            W::MemoryWarning => unimplemented!(),
        }
    }
}

impl From<WindowMsg> for WindowRequest {
    #[inline]
    fn from(value: WindowMsg) -> Self {
        match value {
            WindowMsg::ShowImage { image, id } => Self::ShowImage {
                image,
                id: id.into(),
            },
            WindowMsg::OpenWindow { resp } => Self::OpenWindow { resp },
            // Event::Focus {
            //     change: FocusChange::Gained,
            //     window: WindowType::Window(id),
            // } => Some(Self::WindowRedraw { id }),
            //
            // Event::Resize {
            //     window: WindowType::Window(window_id),
            //     size,
            // } => Some(Self::Resize { size, window_id }),
            //
            // Event::Redraw {
            //     window: WindowType::Window(id),
            // } => Some(Self::WindowRedraw { id }),
            //
            // Event::Resize { .. } => None,
            // Event::Focus { .. } => None,
            // Event::Redraw { .. } => None,
            //
            // Event::Input { .. } => {
            //     // parse_the input to an event
            //     todo!()
            // }
            // Event::Quit => Some(Self::Exit),
            // Event::Tick => unimplemented!(),
        }
    }
}
