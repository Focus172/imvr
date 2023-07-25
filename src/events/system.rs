use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    keyboard::KeyCode,
    window::WindowId,
};

use super::{EventParser, Request};

/// Module for parsing winit events into requests
pub struct WinitEventHandler {
    identity_map: Arc<Mutex<BTreeMap<WindowId, usize>>>,
}

impl<'a> EventParser<Event<'a, ()>> for WinitEventHandler {
    /// Handle an event from the event loop and produce a list of events to
    /// append to the main list
    fn parse(&mut self, event: Event<'a, ()>) -> Option<super::Request> {
        // self.mouse_cache.handle_event(&event);

        // Run window event handlers.
        // let run_context_handlers = match &mut event {
        //     Event::WindowEvent(event) => self.run_window_event_handlers(event, event_loop),
        //     _ => true,
        // };

        // Perform default actions for events.
        match event {
            Event::WindowEvent { window_id, event } => {
                let window_index = self.index_from_window_id(&window_id);
                match event {
                    WindowEvent::Resized(new_size) => Some(Request::Multiple(vec![
                        Request::Resize {
                            size: (new_size.width, new_size.height).into(),
                            window_index,
                        },
                        Request::Redraw { window_index },
                    ])),
                    WindowEvent::KeyboardInput { event, .. } => self.handle_keypress(event),
                    WindowEvent::CloseRequested => Some(Request::CloseWindow { window_index }),
                    // WindowEvent::Focused(_) => todo!(),
                    // WindowEvent::ModifiersChanged(_) => todo!(),
                    _ => None,
                }
            }
            Event::RedrawRequested(window_id) => Some(Request::Redraw {
                window_index: self.index_from_window_id(&window_id),
            }),
            // Event::NewEvents(_) => todo!(),
            // Event::Suspended => todo!(),
            // Event::Resumed => todo!(),
            // Event::MainEventsCleared => todo!(),
            // Event::LoopDestroyed => todo!(),
            _ => None,
        }
    }

    fn close(&mut self) -> ! {
        todo!()
    }
}

impl WinitEventHandler {
    pub fn new(identity_map: Arc<Mutex<BTreeMap<WindowId, usize>>>) -> Self {
        Self { identity_map }
    }

    pub fn handle_keypress(&mut self, key: KeyEvent) -> Option<Request> {
        match (key.physical_key, key.state, key.repeat) {
            (KeyCode::KeyQ, ElementState::Pressed, _) => Some(Request::Exit),
            (KeyCode::KeyL, ElementState::Pressed, _) => {
                log::warn!("This key press is not handled rn");
                // self.request_queue.push_back(Request::NextImage)
                None
            }
            (_, _, _) => None,
        }
    }

    pub fn index_from_window_id(&self, id: &WindowId) -> usize {
        *self.identity_map.lock().unwrap().get(id).unwrap()
    }
}
