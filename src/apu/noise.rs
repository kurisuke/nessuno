use super::misc::{Envelope, LengthCounter, Timer};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Noise {
    envelope: Envelope,
    timer: Timer,
    shift_reg: ShiftReg,
    length_counter: LengthCounter,
}

impl Noise {
    pub fn new() -> Noise {
        Noise {
            envelope: Envelope::new(),
            timer: Timer::new(),
            shift_reg: ShiftReg::new(),
            length_counter: LengthCounter::new(),
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x400c => {
                let lc_flag_halt = (data & 0x20) != 0;
                let env_flag_const = (data & 0x10) != 0;
                let env_volume = data & 0x0f;

                self.length_counter.halt = lc_flag_halt;
                self.envelope.flag_loop = lc_flag_halt;
                self.envelope.flag_const = env_flag_const;
                self.envelope.set_volume(env_volume);
            }
            0x400e => {
                let sr_flag_mode = data & 0x80 != 0;
                let timer_period_idx = (data & 0x0f) as usize;

                self.timer.set_period(TIMER_PERIODS[timer_period_idx]);
                self.shift_reg.flag_mode = sr_flag_mode;
            }
            0x400f => {
                let lc_counter = ((data & 0xf8) >> 3) as usize;
                self.length_counter.set_counter(lc_counter);
                self.envelope.flag_start = true;
            }
            _ => {}
        }
    }

    pub fn get_lc_enable(&self) -> bool {
        self.length_counter.get_enable()
    }

    pub fn set_lc_enable(&mut self, enable: bool) {
        self.length_counter.set_enable(enable);
    }

    pub fn clock_apu(&mut self) {
        if self.timer.clock() {
            self.shift_reg.clock();
        }
    }

    pub fn clock_quarter_frame(&mut self) {
        self.envelope.clock_quarter_frame();
    }

    pub fn clock_half_frame(&mut self) {
        self.length_counter.clock_half_frame();
    }

    pub fn sample(&self) -> u8 {
        if self.shift_reg.is_muted() || self.length_counter.is_muted() {
            0
        } else {
            self.envelope.output()
        }
    }
}

#[derive(Deserialize, Serialize)]
struct ShiftReg {
    reg: u16,
    flag_mode: bool,
}

impl ShiftReg {
    fn new() -> ShiftReg {
        ShiftReg {
            reg: 0x0001,
            flag_mode: false,
        }
    }

    fn clock(&mut self) {
        let feedback = if self.flag_mode {
            (self.reg & 0x0001) ^ ((self.reg >> 6) & 0x0001)
        } else {
            (self.reg & 0x0001) ^ ((self.reg >> 1) & 0x0001)
        };
        self.reg >>= 1;
        self.reg |= feedback << 14;
        self.reg &= 0x7fff;
    }

    fn is_muted(&self) -> bool {
        self.reg & 0x0001 != 0
    }
}

const TIMER_PERIODS: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];
