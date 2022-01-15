pub struct Timer {
    pub period: u16,
    cur: u16,
}

impl Timer {
    pub fn reset(&mut self, period: u16) {
        self.period = period & 0x7ff;
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

pub struct LengthCounter {
    counter: u8,
    halt: bool,
    enable: bool,
}

impl LengthCounter {
    pub fn set_counter(&mut self, table_idx: usize) {
        if self.enable {
            self.counter = LENGTH_TABLE[table_idx & 0x1f];
        }
    }

    pub fn set_enable(&mut self, enable: bool) {
        if enable {
            self.enable = true;
        } else {
            self.enable = false;
            self.counter = 0;
        }
    }

    pub fn set_halt(&mut self, halt: bool) {
        self.halt = halt;
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

pub struct Sequencer {
    seq: u8,
}

impl Sequencer {
    pub fn reset(&mut self, seq_idx: usize) {
        self.seq = SEQUENCES[seq_idx & 0x03];
    }

    pub fn clock(&mut self) {
        self.seq = self.seq.rotate_right(1);
    }

    pub fn is_muted(&self) -> bool {
        self.seq & 0x01 == 0
    }
}

const SEQUENCES: [u8; 4] = [0b01000000, 0b01100000, 0b01111000, 0b10011111];
