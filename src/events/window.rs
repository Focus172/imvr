use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use winit::{
    event::{ElementState, Event as WEvent, KeyEvent, WindowEvent},
    keyboard::KeyCode,
};

use crate::prelude::*;

/// Module for parsing winit events into requests
pub struct WindowEventHandler {
    channel_in: Receiver<Request>,
    channel_out: Sender<WEvent<'static, ()>>, // identity_map: Arc<Mutex<BTreeMap<WindowId, usize>>>,
    _handle: thread::JoinHandle<()>,
}

impl WindowEventHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

        let handle = thread::spawn(move || parse(rx2, tx));
        Self {
            channel_in: rx,
            channel_out: tx2,
            _handle: handle,
        }
    }
    /// Handle an event from the event loop and produce a list of events to
    /// append to the main list

    fn close(&mut self) -> ! {
        todo!()
    }
}

fn parse(channel_in: Receiver<WEvent<'static, ()>>, channel_out: Sender<Request>) -> ! {
    // self.mouse_cache.handle_event(&event);

    // Run window event handlers.
    // let run_context_handlers = match &mut event {
    //     Event::WindowEvent(event) => self.run_window_event_handlers(event, event_loop),
    //     _ => true,
    // };

    // Perform default actions for events.
    for event in channel_in.iter() {
        let req = match event {
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
        };

        // this unwrap should cause this to exit when it is done
        if let Some(r) = req {
            channel_out.send(r).unwrap()
        }
    }

    panic!("Ran out of evenrts");
}

pub fn handle_keypress(key: KeyEvent) -> Option<Request> {
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
