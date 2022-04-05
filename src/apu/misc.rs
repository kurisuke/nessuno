use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Timer {
    pub period: u16,
    cur: u16,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            period: 0x0000,
            cur: 0x0000,
        }
    }

    pub fn set_period(&mut self, period: u16) {
        self.period = period & 0x7ff;
        self.cur = self.period;
    }

    pub fn set_period_hi(&mut self, period_hi: u8) {
        self.period = (self.period & 0x00ff) | ((period_hi as u16 & 0x0007) << 8);
        self.cur = self.period;
    }

    pub fn set_period_lo(&mut self, period_lo: u8) {
        self.period = (self.period & 0x0700) | (period_lo as u16 & 0x00ff);
        self.cur = self.period;
    }

    pub fn clock(&mut self) -> bool {
        if self.cur == 0 {
            self.cur = self.period;
            true
        } else {
            self.cur -= 1;
            false
        }
    }

    pub fn is_muted(&self) -> bool {
        self.cur < 8
    }
}

#[derive(Deserialize, Serialize)]
pub struct LengthCounter {
    counter: u8,
    pub halt: bool,
    enable: bool,
}

impl LengthCounter {
    pub fn new() -> LengthCounter {
        LengthCounter {
            counter: LENGTH_TABLE[0],
            halt: false,
            enable: false,
        }
    }

    pub fn set_counter(&mut self, table_idx: usize) {
        if self.enable {
            self.counter = LENGTH_TABLE[table_idx & 0x1f];
        }
    }

    pub fn get_enable(&self) -> bool {
        self.enable
    }

    pub fn set_enable(&mut self, enable: bool) {
        if enable {
            self.enable = true;
        } else {
            self.enable = false;
            self.counter = 0;
        }
    }

    pub fn clock_half_frame(&mut self) {
        if self.counter > 0 && !self.halt {
            self.counter -= 1;
        }
    }

    pub fn is_muted(&self) -> bool {
        self.counter == 0
    }
}

const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

#[derive(Deserialize, Serialize)]
pub struct Sequencer {
    seq: u8,
    pos: u32,
}

impl Sequencer {
    pub fn new() -> Sequencer {
        Sequencer {
            seq: SEQUENCES[0],
            pos: 0,
        }
    }

    pub fn restart(&mut self) {
        self.seq = self.seq.rotate_left(self.pos);
        self.pos = 0;
    }

    pub fn set_seq(&mut self, seq_idx: usize) {
        self.seq = SEQUENCES[seq_idx & 0x03];
        self.seq = self.seq.rotate_right(self.pos);
    }

    pub fn clock(&mut self) {
        self.seq = self.seq.rotate_right(1);
        self.pos = (self.pos + 1) % 8;
    }

    pub fn is_muted(&self) -> bool {
        self.seq & 0x01 == 0
    }
}

const SEQUENCES: [u8; 4] = [0b01000000, 0b01100000, 0b01111000, 0b10011111];

#[derive(Deserialize, Serialize)]
pub struct Envelope {
    pub flag_start: bool,
    pub flag_loop: bool,
    pub flag_const: bool,
    divider: u8,
    decay_level: u8,
    volume: u8,
}

impl Envelope {
    pub fn new() -> Envelope {
        Envelope {
            flag_start: false,
            flag_loop: false,
            flag_const: false,
            divider: 0x00,
            decay_level: 0x00,
            volume: 0x00,
        }
    }

    pub fn set_volume(&mut self, volume: u8) {
        self.volume = volume & 0x0f;
    }

    pub fn clock_quarter_frame(&mut self) {
        if self.flag_start {
            self.decay_level = 15;
            self.divider = self.volume;
            self.flag_start = false;
        } else if self.divider == 0 {
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

    pub fn output(&self) -> u8 {
        if self.flag_const {
            self.volume
        } else {
            self.decay_level
        }
    }
}
