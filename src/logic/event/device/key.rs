use crate::logic::req::WindowRequest;

use ext::parse::SomeFrom;

impl SomeFrom<Key> for WindowRequest {
    fn some_from(value: Key) -> Option<Self> {
        match value {
            Key::Char('q') => Some(Self::Exit),
            Key::Char('l') => todo!("Select Next Image"),
            Key::Char(_) => None,
            Key::Ctrl('c') => Some(Self::Exit),
            Key::Ctrl(_) => None,
            Key::Alt(_) => None,
        }
    }
}

#[derive(Debug)]
pub enum Key {
    Char(char),
    Ctrl(char),
    Alt(char),
}

use winit::event::KeyEvent as WinitKeyEvent;

impl SomeFrom<WinitKeyEvent> for Key {
    fn some_from(value: WinitKeyEvent) -> Option<Self> {
        use winit::event::ElementState;
        use winit::keyboard::{KeyCode as WinitKeyCode, PhysicalKey};
        match (value.physical_key, value.repeat, value.state) {
            (PhysicalKey::Code(WinitKeyCode::KeyQ), _, ElementState::Pressed) => {
                Some(Key::Char('q'))
            }
            (_, _, _) => None,
        }
    }
}

use crossterm::event::KeyEvent as CrossKeyEvent;

impl SomeFrom<CrossKeyEvent> for Key {
    fn some_from(value: CrossKeyEvent) -> Option<Self> {
        let CrossKeyEvent {
            code, modifiers, ..
        } = value;

        use crossterm::event::KeyCode as Kc;
        use crossterm::event::KeyModifiers as Km;
        match (code, modifiers) {
            (Kc::Char(c), Km::CONTROL) => Some(Key::Ctrl(c)),
            (Kc::Char(c), Km::ALT) => Some(Key::Alt(c)),
            (Kc::Char(c), Km::NONE) => Some(Key::Char(c)),
            (_, _) => None,
        }
    }
}
