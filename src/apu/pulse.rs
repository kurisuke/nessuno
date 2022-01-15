use super::misc::{LengthCounter, Sequencer, Timer};

pub struct Pulse {
    envelope: Envelope,
    sweep: Sweep,
    timer: Timer,
    sequencer: Sequencer,
    length_counter: LengthCounter,
}

impl Pulse {
    pub fn new(i: usize) -> Pulse {
        Pulse {
            envelope: Envelope::new(),
            sweep: Sweep::new((i & 0x1) as u16),
            timer: Timer::new(),
            sequencer: Sequencer::new(),
            length_counter: LengthCounter::new(),
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr & 0x0003 {
            0 => {
                // $4000 / $4004
                let seq_idx = ((data & 0xc0) >> 6) as usize;
                let lc_flag_halt = (data & 0x20) != 0;
                let env_flag_const = (data & 0x10) != 0;
                let env_volume = data & 0x0f;

                self.sequencer.set_seq(seq_idx);
                self.length_counter.halt = lc_flag_halt;
                self.envelope.flag_const = env_flag_const;
                self.envelope.set_volume(env_volume);
            }
            1 => {
                // $4001 / $4005
                let sweep_enabled = (data & 0x80) != 0;
                let sweep_divider_reload = (data & 0x70) >> 4;
                let sweep_negate = (data & 0x08) != 0;
                let sweep_shift_count = data & 0x07;

                self.sweep.flag_enabled = sweep_enabled;
                self.sweep.set_divider_reload(sweep_divider_reload);
                self.sweep.flag_negate = sweep_negate;
                self.sweep.set_shift_count(sweep_shift_count);
                self.sweep.flag_reload = true;
            }
            2 => {
                // $4002 / $4006
                self.timer.set_period_lo(data);
            }
            3 => {
                // $4003 / $4007
                let lc_counter = ((data & 0xf8) >> 5) as usize;
                let timer_period_hi = data & 0x07;

                self.length_counter.set_counter(lc_counter);
                self.timer.set_period_hi(timer_period_hi);

                self.sequencer.restart();
                self.envelope.flag_start = true;
            }
            _ => {}
        }
    }

    pub fn set_lc_enable(&mut self, enable: bool) {
        self.length_counter.set_enable(enable);
    }

    pub fn clock_ppu(&mut self) {
        self.sweep.track_sweep(self.timer.period);
    }

    pub fn clock_apu(&mut self) {
        if self.timer.clock() {
            self.sequencer.clock();
        }
    }

    pub fn clock_quarter_frame(&mut self) {
        self.envelope.clock_quarter_frame();
    }

    pub fn clock_half_frame(&mut self) {
        self.length_counter.clock_half_frame();
        if let Some(new_period) = self.sweep.clock_half_frame() {
            self.timer.set_period(new_period);
        }
    }

    pub fn sample(&self) -> u8 {
        if self.sequencer.is_muted()
            || self.sweep.is_muted()
            || self.timer.is_muted()
            || self.length_counter.is_muted()
        {
            0
        } else {
            self.envelope.output()
        }
    }
}

struct Envelope {
    pub flag_start: bool,
    pub flag_loop: bool,
    pub flag_const: bool,
    divider: u8,
    decay_level: u8,
    volume: u8,
}

impl Envelope {
    fn new() -> Envelope {
        Envelope {
            flag_start: false,
            flag_loop: false,
            flag_const: false,
            divider: 0x00,
            decay_level: 0x00,
            volume: 0x00,
        }
    }

    fn set_volume(&mut self, volume: u8) {
        self.volume = volume & 0x0f;
    }

    fn clock_quarter_frame(&mut self) {
        if self.flag_start {
            self.decay_level = 15;
            self.divider = self.volume;
            self.flag_start = false;
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

struct Sweep {
    pub flag_enabled: bool,
    pub flag_negate: bool,
    pub flag_reload: bool,
    target_period: u16,
    cur_period: u16,
    shift_count: usize,
    neg_offset: u16,
    divider: u8,
    divider_reload: u8,
}

impl Sweep {
    fn new(neg_offset: u16) -> Sweep {
        Sweep {
            flag_enabled: false,
            flag_negate: false,
            flag_reload: false,
            target_period: 0x0000,
            cur_period: 0x0000,
            shift_count: 0,
            neg_offset,
            divider: 0x00,
            divider_reload: 0x00,
        }
    }

    fn set_divider_reload(&mut self, divider_reload: u8) {
        self.divider_reload = divider_reload & 0x07;
    }

    fn set_shift_count(&mut self, shift_count: u8) {
        self.shift_count = (shift_count & 0x07) as usize;
    }

    fn track_sweep(&mut self, cur_period: u16) {
        self.cur_period = cur_period;
        let change_amount = cur_period >> self.shift_count;
        if self.flag_negate {
            self.target_period = cur_period - change_amount - self.neg_offset;
        } else {
            self.target_period = cur_period + change_amount;
        }
    }

    fn clock_half_frame(&mut self) -> Option<u16> {
        let new_period = if self.divider == 0 && self.flag_enabled && !self.is_muted() {
            Some(self.target_period)
        } else {
            None
        };

        if self.divider == 0 || self.flag_reload {
            self.divider = self.divider_reload;
            self.flag_reload = false;
        } else {
            self.divider -= 1;
        }

        new_period
    }

    fn is_muted(&self) -> bool {
        self.cur_period < 8 || self.target_period >= 0x7ff
    }
}
