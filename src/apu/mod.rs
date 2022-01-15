mod misc;
mod mixer;
mod pulse;
mod triangle;

use mixer::Mixer;
use pulse::Pulse;
use triangle::Triangle;

pub struct Apu {
    pulse: [Pulse; 2],
    triangle: Triangle,
    mixer: Mixer,
    frame_counter: FrameCounter,
}

impl Apu {
    pub fn new() -> Apu {
        Apu {
            pulse: [Pulse::new(0), Pulse::new(1)],
            triangle: Triangle::new(),
            mixer: Mixer::new(),
            frame_counter: FrameCounter::new(),
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4000..=0x4003 => {
                self.pulse[0].cpu_write(addr, data);
            }
            0x4004..=0x4007 => {
                self.pulse[1].cpu_write(addr, data);
            }
            0x4008 | 0x400a | 0x400b => {
                self.triangle.cpu_write(addr, data);
            }
            0x4015 => {
                self.pulse[0].set_lc_enable((data & 0x01) != 0);
                self.pulse[1].set_lc_enable((data & 0x02) != 0);
                self.triangle.set_lc_enable((data & 0x04) != 0);
            }
            0x4017 => {
                self.frame_counter.mode = if (data & 0x80) != 0 {
                    FrameCounterMode::Step5
                } else {
                    FrameCounterMode::Step4
                };
                self.frame_counter.flag_irq_inhibit = (data & 0x40) != 0;
            }
            _ => {}
        }
    }

    pub fn cpu_read(&self, _addr: u16) -> u8 {
        0x00
    }

    pub fn clock(&mut self) {
        let clock_events = self.frame_counter.clock();
        if clock_events.cpu_cycle {
            self.triangle.clock_cpu();
        }
        if clock_events.apu_cycle {
            self.pulse[0].clock_apu();
            self.pulse[1].clock_apu();
        }
        if clock_events.quarter_frame {
            self.pulse[0].clock_quarter_frame();
            self.pulse[1].clock_quarter_frame();
            self.triangle.clock_quarter_frame();
        }
        if clock_events.half_frame {
            self.pulse[0].clock_half_frame();
            self.pulse[1].clock_half_frame();
            self.triangle.clock_half_frame();
        }

        self.pulse[0].clock_ppu();
        self.pulse[1].clock_ppu();
    }

    pub fn get_output_sample(&self) -> f32 {
        self.mixer.sample(
            self.pulse[0].sample(),
            self.pulse[1].sample(),
            self.triangle.sample(),
            0,
            0,
        )
    }
}

struct ClockEvents {
    cpu_cycle: bool,
    apu_cycle: bool,
    quarter_frame: bool,
    half_frame: bool,
    irq: bool,
}

#[derive(PartialEq)]
enum FrameCounterMode {
    Step4,
    Step5,
}

struct FrameCounter {
    pub flag_irq_inhibit: bool,
    pub mode: FrameCounterMode,
    ppu_clock_counter: usize,
}

impl FrameCounter {
    fn new() -> FrameCounter {
        FrameCounter {
            flag_irq_inhibit: false,
            ppu_clock_counter: 0,
            mode: FrameCounterMode::Step4,
        }
    }

    pub fn clock(&mut self) -> ClockEvents {
        let mut clock_events = ClockEvents {
            apu_cycle: false,
            cpu_cycle: false,
            quarter_frame: false,
            half_frame: false,
            irq: false,
        };
        self.ppu_clock_counter += 1;

        if self.ppu_clock_counter % 3 == 0 {
            clock_events.cpu_cycle = true;
        }

        if self.ppu_clock_counter % 6 == 0 {
            clock_events.apu_cycle = true;
        }

        let cycle_steps: [usize; 4] = match self.mode {
            FrameCounterMode::Step4 => [22371, 44739, 67113, 89484],
            FrameCounterMode::Step5 => [22371, 44739, 67113, 111843],
        };

        if !self.flag_irq_inhibit
            && self.mode == FrameCounterMode::Step4
            && (self.ppu_clock_counter == cycle_steps[3] - 3
                || self.ppu_clock_counter == cycle_steps[3]
                || self.ppu_clock_counter == cycle_steps[3] + 3)
        {
            clock_events.irq = true;
        }

        if cycle_steps.contains(&self.ppu_clock_counter) {
            clock_events.quarter_frame = true;
        }

        if self.ppu_clock_counter == cycle_steps[1] || self.ppu_clock_counter == cycle_steps[3] {
            clock_events.half_frame = true;
        }

        if self.ppu_clock_counter == cycle_steps[3] + 3 {
            self.ppu_clock_counter = 0;
        }

        clock_events
    }
}
