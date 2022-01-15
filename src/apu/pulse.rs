use super::misc::{Timer,Sequencer,LengthCounter};

struct Pulse {
}

struct Envelope {
    flag_start: bool,
    flag_loop: bool,
    flag_const: bool,
    divider: u8,
    decay_level: u8,
    volume: u8,
}

impl Envelope {
    fn set_start(&mut self, flag_start: bool) {
        self.flag_start = flag_start;
    }

    fn set_loop(&mut self, flag_loop: bool) {
        self.flag_loop = flag_loop;
    }

    fn set_const(&mut self, flag_const: bool) {
        self.flag_const = flag_const;
    }

    fn set_volume(&mut self, volume: u8) {
        self.volume = volume & 0x0f;
    }

    fn clock_quarter_frame(&mut self) {
        if self.flag_start {
            self.decay_level = 15;
            self.divider = self.volume;
        } else {
            if self.divider == 0 {
                self.divider = self.volume;

                if self.decay_level == 0 {
                    if self.flag_loop {
                        self.decay_level = 15;
                    }
                } else {
                    self.decay_level -= 1;
                }                
            } else {
                self.divider -= 1;
            }
        }
    }

    fn output(&self) -> u8 {
        if self.flag_const {
            self.volume
        } else {
            self.decay_level
        }
    }
}
