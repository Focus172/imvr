use crate::backend::mouse::MouseButtonState;
use std::collections::BTreeMap;
use winit::event::MouseButton;
use winit::{
    event::{DeviceEvent, DeviceId, ElementState, Event, WindowEvent},
    window::WindowId,
};

#[derive(Default)]
pub struct MouseCache {
    mouse_buttons: BTreeMap<DeviceId, MouseButtonState>,
    mouse_position: BTreeMap<(WindowId, DeviceId), glam::Vec2>,
    mouse_prev_position: BTreeMap<(WindowId, DeviceId), glam::Vec2>,
}

impl MouseCache {
    pub fn get_position(&self, window_id: WindowId, device_id: DeviceId) -> Option<glam::Vec2> {
        self.mouse_position.get(&(window_id, device_id)).copied()
    }

    pub fn get_prev_position(
        &self,
        window_id: WindowId,
        device_id: DeviceId,
    ) -> Option<glam::Vec2> {
        self.mouse_prev_position
            .get(&(window_id, device_id))
            .copied()
    }

    pub fn get_buttons(&self, device_id: DeviceId) -> Option<&MouseButtonState> {
        self.mouse_buttons.get(&device_id)
    }

    pub fn handle_event(&mut self, event: &Event<()>) {
        match event {
            Event::WindowEvent { window_id, event } => self.handle_window_event(*window_id, event),
            Event::DeviceEvent { device_id, event } => self.handle_device_event(*device_id, event),
            _ => (),
        }
    }

    fn handle_window_event(&mut self, window_id: WindowId, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput {
                device_id,
                button,
                state,
                ..
            } => {
                let buttons = self.mouse_buttons.entry(*device_id).or_default();
                buttons.set_pressed(*button, *state == ElementState::Pressed);
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
                ..
            } => {
                let cached_position = self
                    .mouse_position
                    .entry((window_id, *device_id))
                    .or_insert_with(|| [0.0, 0.0].into());
                let cached_prev_position = self
                    .mouse_prev_position
                    .entry((window_id, *device_id))
                    .or_insert_with(|| [0.0, 0.0].into());
                *cached_prev_position = *cached_position;
                *cached_position = glam::DVec2::new(position.x, position.y).as_vec2();
            }
            _ => {}
        }
    }

    fn handle_device_event(&mut self, device_id: DeviceId, event: &DeviceEvent) {
        if let DeviceEvent::Removed = event {
            self.remove_device(device_id)
        }
    }

    fn remove_device(&mut self, device_id: DeviceId) {
        self.mouse_buttons.remove(&device_id);
        let keys: Vec<_> = self
            .mouse_position
            .keys()
            .filter(|(_, x)| *x == device_id)
            .copied()
            .collect();
        for key in &keys {
            self.mouse_position.remove(key);
        }
        let keys: Vec<_> = self
            .mouse_prev_position
            .keys()
            .filter(|(_, x)| *x == device_id)
            .copied()
            .collect();
        for key in &keys {
            self.mouse_prev_position.remove(key);
        }
    }
}

/// The state of all mouse buttons.
#[derive(Debug, Clone, Default)]
pub struct MouseButtonState {
    /// The set of pressed buttons.
    buttons: std::collections::HashSet<MouseButton>,
}

impl MouseButtonState {
    /// Check if a button is pressed.
    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.buttons.get(&button).is_some()
    }

    /// Mark a button as pressed or unpressed.
    pub fn set_pressed(&mut self, button: MouseButton, pressed: bool) {
        if pressed {
            self.buttons.insert(button);
        } else {
            self.buttons.remove(&button);
        }
    }
}
