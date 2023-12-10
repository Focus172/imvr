//! A backend agnostic event that is converted into and request

use super::msg::WindowMsg;

pub mod device;

pub type WindowEvent = winit::event::Event<WindowMsg>;

// use ext::glam::UVec2;
// use winit::window::WindowId;

// #[derive(Debug)]
// pub enum Event {
//     FileProvided,
//     Focus {
//         change: FocusChange,
//         window: WindowType,
//     },
//     ContentChanged {
//         window: WindowType,
//     },
//     Resize {
//         window: WindowType,
//         new_size: UVec2,
//     },
//     Input {
//         window: WindowType,
//         input: Key,
//     },
//     Exit,
// }
//
// #[derive(Debug)]
// pub enum FocusChange {
//     Gained,
//     Lost,
// }
//
// impl FocusChange {
//     #[inline]
//     pub fn did_gain(&self) -> bool {
//         matches!(self, Self::Gained)
//     }
// }
//
// impl From<bool> for FocusChange {
//     #[inline]
//     fn from(value: bool) -> Self {
//         match value {
//             true => FocusChange::Gained,
//             false => FocusChange::Lost,
//         }
//     }
// }
//
