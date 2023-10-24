use winit::event::ElementState;

use crossterm::event::KeyCode as CrossKeyCode;
use crossterm::event::{KeyEvent as CrossKeyEvent, KeyModifiers};

use winit::event::KeyEvent as WinitKeyEvent;
use winit::keyboard::{KeyCode as WinitKeyCode, PhysicalKey};

pub enum Key {
    Char(char),
    Ctrl(char),
    Alt(char),
    Nothing,
}

impl From<WinitKeyEvent> for Key {
    fn from(value: WinitKeyEvent) -> Self {
        match (value.physical_key, value.repeat, value.state) {
            (PhysicalKey::Code(WinitKeyCode::KeyQ), _, ElementState::Pressed) => Key::Char('q'),
            (_, _, _) => unimplemented!(),
        }
    }
}

impl From<CrossKeyEvent> for Key {
    fn from(value: CrossKeyEvent) -> Self {
        let CrossKeyEvent {
            code,
            modifiers,
            kind,
            state,
        } = value;

        match (code, modifiers, kind, state) {
            (CrossKeyCode::Char(c), KeyModifiers::CONTROL, _, _) => Key::Ctrl(c),
            (CrossKeyCode::Char(c), KeyModifiers::ALT, _, _) => Key::Alt(c),
            (CrossKeyCode::Char(c), KeyModifiers::NONE, _, _) => Key::Char(c),
            (_, _, _, _) => Key::Nothing,
        }
    }
}
