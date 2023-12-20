use crate::controller::ControllerInput;
use gilrs::{Button, Gilrs};
use std::collections::HashSet;
use winit::keyboard::KeyCode;
use winit_input_helper::WinitInputHelper;

pub struct InputGilrs {
    gilrs: Gilrs,
    pushed_buttons: HashSet<ControllerInput>,
}

impl Default for InputGilrs {
    fn default() -> Self {
        Self::new()
    }
}

impl InputGilrs {
    pub fn new() -> InputGilrs {
        InputGilrs {
            gilrs: Gilrs::new().unwrap(),
            pushed_buttons: HashSet::new(),
        }
    }

    pub fn get(&mut self) -> Option<(Vec<ControllerInput>, Vec<ControllerInput>)> {
        let mut updated = false;
        while let Some(gilrs::Event {
            id: _,
            event,
            time: _,
        }) = self.gilrs.next_event()
        {
            match &event {
                gilrs::EventType::ButtonPressed(button, _) => {
                    if let Some(b) = map_button(button) {
                        self.pushed_buttons.insert(b);
                        updated = true;
                    }
                }
                gilrs::EventType::ButtonReleased(button, _) => {
                    if let Some(b) = map_button(button) {
                        self.pushed_buttons.remove(&b);
                        updated = true;
                    }
                }
                _ => {}
            }
        }
        if updated {
            Some((self.pushed_buttons.iter().cloned().collect(), vec![]))
        } else {
            None
        }
    }
}

fn map_button(button: &Button) -> Option<ControllerInput> {
    match button {
        Button::DPadLeft => Some(ControllerInput::Left),
        Button::DPadRight => Some(ControllerInput::Right),
        Button::DPadUp => Some(ControllerInput::Up),
        Button::DPadDown => Some(ControllerInput::Down),
        Button::Select => Some(ControllerInput::Select),
        Button::Start => Some(ControllerInput::Start),
        Button::East => Some(ControllerInput::A),
        Button::South => Some(ControllerInput::B),
        _ => None,
    }
}

pub struct InputKeyboard {
    pushed_buttons: HashSet<ControllerInput>,
}

impl Default for InputKeyboard {
    fn default() -> Self {
        Self::new()
    }
}

impl InputKeyboard {
    pub fn new() -> InputKeyboard {
        InputKeyboard {
            pushed_buttons: HashSet::new(),
        }
    }

    pub fn get(
        &mut self,
        input: &WinitInputHelper,
    ) -> Option<(Vec<ControllerInput>, Vec<ControllerInput>)> {
        let mut updated = false;

        if input.key_pressed(KeyCode::ArrowLeft) {
            self.pushed_buttons.insert(ControllerInput::Left);
            updated = true;
        }
        if input.key_pressed(KeyCode::ArrowRight) {
            self.pushed_buttons.insert(ControllerInput::Right);
            updated = true;
        }
        if input.key_pressed(KeyCode::ArrowUp) {
            self.pushed_buttons.insert(ControllerInput::Up);
            updated = true;
        }
        if input.key_pressed(KeyCode::ArrowDown) {
            self.pushed_buttons.insert(ControllerInput::Down);
            updated = true;
        }
        if input.key_pressed(KeyCode::Digit1) {
            self.pushed_buttons.insert(ControllerInput::B);
            updated = true;
        }
        if input.key_pressed(KeyCode::Digit2) {
            self.pushed_buttons.insert(ControllerInput::A);
            updated = true;
        }
        if input.key_pressed(KeyCode::Digit3) {
            self.pushed_buttons.insert(ControllerInput::Select);
            updated = true;
        }
        if input.key_pressed(KeyCode::Digit4) {
            self.pushed_buttons.insert(ControllerInput::Start);
            updated = true;
        }

        if input.key_released(KeyCode::ArrowLeft) {
            self.pushed_buttons.remove(&ControllerInput::Left);
            updated = true;
        }
        if input.key_released(KeyCode::ArrowRight) {
            self.pushed_buttons.remove(&ControllerInput::Right);
            updated = true;
        }
        if input.key_released(KeyCode::ArrowUp) {
            self.pushed_buttons.remove(&ControllerInput::Up);
            updated = true;
        }
        if input.key_released(KeyCode::ArrowDown) {
            self.pushed_buttons.remove(&ControllerInput::Down);
            updated = true;
        }
        if input.key_released(KeyCode::Digit1) {
            self.pushed_buttons.remove(&ControllerInput::B);
            updated = true;
        }
        if input.key_released(KeyCode::Digit2) {
            self.pushed_buttons.remove(&ControllerInput::A);
            updated = true;
        }
        if input.key_released(KeyCode::Digit3) {
            self.pushed_buttons.remove(&ControllerInput::Select);
            updated = true;
        }
        if input.key_released(KeyCode::Digit4) {
            self.pushed_buttons.remove(&ControllerInput::Start);
            updated = true;
        }

        if updated {
            Some((self.pushed_buttons.iter().cloned().collect(), vec![]))
        } else {
            None
        }
    }
}
