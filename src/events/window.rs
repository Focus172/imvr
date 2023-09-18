use std::collections::VecDeque;

use winit::{
    event::{ElementState, Event as WEvent, KeyEvent, WindowEvent},
    keyboard::KeyCode,
};

use crate::prelude::*;

/// Module for parsing winit events into requests
pub struct WindowEventHandler {
    queued: VecDeque<Request>,
}

impl WindowEventHandler {
    pub fn new() -> Self {
        Self {
            queued: VecDeque::new(),
        }
    }

    pub fn add(&mut self, event: WEvent<()>) {
        if let Some(e) = parse(event) {
            self.queued.push_back(e);
        }
    }

    pub fn next(&mut self) -> Option<Request> {
        self.queued.pop_front()
    }
}

fn parse(event: WEvent<()>) -> Option<Request> {
    // self.mouse_cache.handle_event(&event);

    // Run window event handlers.
    // let run_context_handlers = match &mut event {
    //     Event::WindowEvent(event) => self.run_window_event_handlers(event, event_loop),
    //     _ => true,
    // };

    // Perform default actions for events.
    match event {
        WEvent::WindowEvent { window_id, event } => {
            match event {
                WindowEvent::Resized(new_size) => Some(Request::Multiple(vec![
                    Request::Resize {
                        size: (new_size.width, new_size.height).into(),
                        window_id: window_id.into(),
                    },
                    Request::Redraw {
                        window_id: window_id.into(),
                    },
                ])),
                WindowEvent::KeyboardInput { event, .. } => handle_keypress(event),
                WindowEvent::CloseRequested => Some(Request::CloseWindow {
                    window_id: window_id.into(),
                }),
                // WindowEvent::Focused(_) => todo!(),
                // WindowEvent::ModifiersChanged(_) => todo!(),
                _ => None,
            }
        }
        WEvent::RedrawRequested(window_id) => Some(Request::Redraw {
            window_id: window_id.into(),
        }),
        // Event::NewEvents(_) => todo!(),
        // Event::Suspended => todo!(),
        // Event::Resumed => todo!(),
        // Event::MainEventsCleared => todo!(),
        // Event::LoopDestroyed => todo!(),
        _ => None,
    }
}

pub fn handle_keypress(key: KeyEvent) -> Option<Request> {
    match (key.physical_key, key.state, key.repeat) {
        (KeyCode::KeyQ, ElementState::Pressed, _) => Some(Request::Exit { code: Some(0) }),
        (KeyCode::KeyL, ElementState::Pressed, _) => {
            log::warn!("This key press is not handled rn");
            // self.request_queue.push_back(Request::NextImage)
            None
        }
        (_, _, _) => None,
    }
}
