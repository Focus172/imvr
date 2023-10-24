// use crossterm::event::Event as CrossEvent;
// use winit::event::Event as WinitEvent;

// use winit::window::WindowId;
//
// use crate::prelude::*;
//
// pub enum Event<T> {
//     Focus {
//         change: FocusChange,
//         window: WindowType,
//     },
//     Redraw {
//         window: WindowType,
//     },
//     Nothing,
//     User(T),
//     Quit,
// }
//
// pub enum FocusChange {
//     Gained,
//     Lost,
// }
//
// pub enum WindowType {
//     Terminal,
//     Window(u64),
// }
//
// impl From<Option<WindowId>> for WindowType {
//     fn from(value: Option<WindowId>) -> Self {
//         match value {
//             Some(id) => WindowType::Window(id.into()),
//             None => WindowType::Terminal,
//         }
//     }
// }
//
// impl<T> From<CrossEvent> for Event<T> {
//     fn from(value: CrossEvent) -> Self {
//         match value {
//             CrossEvent::FocusGained => Event::Focus {
//                 change: FocusChange::Gained,
//                 window: WindowType::Terminal,
//             },
//             CrossEvent::FocusLost => Event::Focus {
//                 change: FocusChange::Lost,
//                 window: WindowType::Terminal,
//             },
//             CrossEvent::Key(_) => todo!(),
//             CrossEvent::Mouse(_) => todo!(),
//             CrossEvent::Paste(_) => todo!(),
//             CrossEvent::Resize(_, _) => todo!(),
//         }
//     }
// }
//
// impl<T> From<WinitEvent<T>> for Event<T> {
//     fn from(value: WinitEvent<T>) -> Self {
//         match value {
//             WinitEvent::NewEvents(_start_cause) => Event::Nothing,
//             WinitEvent::WindowEvent { window_id, event } => match event {
//                 winit::event::WindowEvent::Resized(_) => todo!(),
//                 winit::event::WindowEvent::Moved(_) => todo!(),
//                 winit::event::WindowEvent::CloseRequested => todo!(),
//                 winit::event::WindowEvent::Destroyed => todo!(),
//                 winit::event::WindowEvent::Focused(b) => {
//                     let change = if b {
//                         FocusChange::Gained
//                     } else {
//                         FocusChange::Lost
//                     };
//                     Event::Focus {
//                         change,
//                         window: Some(window_id).into(),
//                     }
//                 }
//                 winit::event::WindowEvent::KeyboardInput {
//                     device_id,
//                     event,
//                     is_synthetic,
//                 } => {
//                     todo!()
//                 }
//                 _ => todo!(),
//             },
//             WinitEvent::DeviceEvent { device_id, event } => todo!(),
//             WinitEvent::UserEvent(u) => Event::User(u),
//             WinitEvent::Suspended => unreachable!("imvr doesn't support mobile yet"),
//             WinitEvent::Resumed => unreachable!("imvr doesn't support mobile yet"),
//             WinitEvent::AboutToWait => Event::Nothing,
//             WinitEvent::RedrawRequested(id) => Event::Redraw {
//                 window: WindowType::Window(id.into()),
//             },
//             WinitEvent::LoopExiting => Event::Quit,
//         }
//     }
// }
