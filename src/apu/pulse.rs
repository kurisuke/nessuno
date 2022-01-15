use super::misc::{LengthCounter, Sequencer, Timer};

struct Pulse {
    envelope: Envelope,
    sweep: Sweep,
    timer: Timer,
    sequencer: Sequencer,
    length_counter: LengthCounter,
    cpu_clock_counter: usize,
}

impl Pulse {
    fn clock_ppu(&mut self) {
        self.sweep.track_sweep(self.timer.period);
    }

    fn clock_cpu(&mut self) {
        self.cpu_clock_counter += 1;
        if self.cpu_clock_counter % 2 == 0 {
            // clock timer & sequencer
            if self.timer.clock() {
                self.sequencer.clock();
            }
        }
    }

    fn clock_quarter_frame(&mut self) {
        self.envelope.clock_quarter_frame();
    }

    fn clock_half_frame(&mut self) {
        self.length_counter.clock_half_frame();
        if let Some(new_period) = self.sweep.clock_half_frame() {
            self.timer.reset(new_period);
        }
    }

    fn sample(&self) -> u8 {
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
