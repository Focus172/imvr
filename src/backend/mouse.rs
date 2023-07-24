use winit::event::MouseButton;

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
