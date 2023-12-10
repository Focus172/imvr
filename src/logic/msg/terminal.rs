// use crate::prelude::*;
//
// use crossterm::event::Event as CrossEvent;

// use super::{FocusChange, WindowType};

use super::Msg;

#[derive(Debug)]
pub enum TerminalMsg {
    None,
}

impl Msg {
    pub fn as_terminal(&mut self) -> Option<TerminalMsg> {
        Some(TerminalMsg::None)
    }
}

// impl From<CrossEvent> for Event {
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
//             CrossEvent::Key(k) => Event::Input {
//                 window: WindowType::Terminal,
//                 input: k.some_into().unwrap(),
//             },
//             CrossEvent::Mouse(_) => todo!(),
//             CrossEvent::Paste(_) => todo!(),
//             CrossEvent::Resize(_, _) => todo!(),
//         }
//     }
// }
