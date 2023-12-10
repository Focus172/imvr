//! A rquest parseable by the render thread

mod terminal;
mod window;

pub use self::terminal::TerminalRequest;
pub use self::window::WindowRequest;
