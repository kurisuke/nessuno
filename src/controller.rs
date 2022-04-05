use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ControllerInput {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Deserialize, Serialize)]
pub struct Controller {
    reg: u8,
    state: u8,
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            reg: 0x00,
            state: 0x00,
        }
    }

    pub fn update(&mut self, input: &[ControllerInput]) {
        let mut reg = 0x00;
        for i in input {
            match *i {
                ControllerInput::A => {
                    reg |= 0x80;
                }
                ControllerInput::B => {
                    reg |= 0x40;
                }
                ControllerInput::Select => {
                    reg |= 0x20;
                }
                ControllerInput::Start => {
                    reg |= 0x10;
                }
                ControllerInput::Up => {
                    reg |= 0x08;
                }
                ControllerInput::Down => {
                    reg |= 0x04;
                }
                ControllerInput::Left => {
                    reg |= 0x02;
                }
                ControllerInput::Right => {
                    reg |= 0x01;
                }
            }
        }
        self.reg = reg;
    }

    pub fn write(&mut self) {
        self.state = self.reg;
    }

    pub fn read(&mut self) -> u8 {
        let data = ((self.state & 0x80) != 0) as u8;
        self.state <<= 1;
        data
    }

    pub fn read_ro(&self) -> u8 {
        ((self.state & 0x80) > 0) as u8
    }
}
