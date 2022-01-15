use super::misc::{LengthCounter, Timer};

pub struct Triangle {
    timer: Timer,
    linear_counter: LinearCounter,
    length_counter: LengthCounter,
    sequencer: Sequencer,
}

impl Triangle {
    pub fn new() -> Triangle {
        Triangle {
            timer: Timer::new(),
            linear_counter: LinearCounter::new(),
            length_counter: LengthCounter::new(),
            sequencer: Sequencer::new(),
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4008 => {
                let flag_control = (data & 0x80) != 0;
                let linear_reload = data & 0x7f;

                self.linear_counter.flag_control = flag_control;
                self.length_counter.halt = flag_control;
                self.linear_counter.reload = linear_reload;
            }
            0x400a => {
                self.timer.set_period_lo(data);
            }
            0x400b => {
                let lc_table_idx = ((data & 0xf8) >> 5) as usize;
                let timer_period_hi = data & 0x07;

                self.length_counter.set_counter(lc_table_idx);
                self.linear_counter.flag_reload = true;
                self.timer.set_period_hi(timer_period_hi);
            }
            _ => {}
        }
    }

    pub fn set_lc_enable(&mut self, enable: bool) {
        self.length_counter.set_enable(enable);
    }

    pub fn clock_cpu(&mut self) {
        if self.timer.clock() && !self.linear_counter.is_muted() && !self.length_counter.is_muted()
        {
            self.sequencer.clock();
        }
    }

    pub fn clock_quarter_frame(&mut self) {
        self.linear_counter.clock_quarter_frame();
    }

    pub fn clock_half_frame(&mut self) {
        self.length_counter.clock_half_frame();
    }

    pub fn sample(&self) -> u8 {
        self.sequencer.output()
    }
}

struct LinearCounter {
    pub flag_reload: bool,
    pub flag_control: bool,
    pub reload: u8,
    counter: u8,
}

impl LinearCounter {
    fn new() -> LinearCounter {
        LinearCounter {
            flag_reload: false,
            flag_control: false,
            reload: 0x00,
            counter: 0x00,
        }
    }

    fn clock_quarter_frame(&mut self) {
        if self.flag_reload {
            self.counter = self.reload;
        } else if self.counter > 0 {
            self.counter -= 1;
        }

        if self.flag_control {
            self.flag_reload = false;
        }
    }

    pub fn is_muted(&self) -> bool {
        self.counter == 0
    }
}

struct Sequencer {
    pos: usize,
}

impl Sequencer {
    fn new() -> Sequencer {
        Sequencer { pos: 0 }
    }

    fn clock(&mut self) {
        self.pos = (self.pos + 1) % 32;
    }

    fn output(&self) -> u8 {
        SEQ_TRIANGLE[self.pos]
    }
}

const SEQ_TRIANGLE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15,
];
