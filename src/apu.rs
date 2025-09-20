mod misc;
mod mixer;
mod noise;
mod pulse;
mod triangle;

use mixer::Mixer;
use noise::Noise;
use pulse::Pulse;
use triangle::Triangle;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Apu {
    pulse: [Pulse; 2],
    triangle: Triangle,
    noise: Noise,
    mixer: Mixer,
    frame_counter: FrameCounter,
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}

impl Apu {
    pub fn new() -> Apu {
        Apu {
            pulse: [Pulse::new(0), Pulse::new(1)],
            triangle: Triangle::new(),
            noise: Noise::new(),
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
            0x4008..=0x400b => {
                self.triangle.cpu_write(addr, data);
            }
            0x400c..=0x400f => {
                self.noise.cpu_write(addr, data);
            }
            0x4015 => {
                self.pulse[0].set_lc_enable((data & 0x01) != 0);
                self.pulse[1].set_lc_enable((data & 0x02) != 0);
                self.triangle.set_lc_enable((data & 0x04) != 0);
                self.noise.set_lc_enable((data & 0x08) != 0);
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

    pub fn cpu_read(&self, addr: u16) -> u8 {
        match addr {
            0x4015 => {
                let mut status = 0;
                status |= self.pulse[0].get_lc_enable() as u8;
                status |= (self.pulse[1].get_lc_enable() as u8) << 1;
                status |= (self.triangle.get_lc_enable() as u8) << 2;
                status |= (self.noise.get_lc_enable() as u8) << 3;
                status
            }
            _ => 0x00,
        }
    }

    pub fn clock(&mut self) {
        let clock_events = self.frame_counter.clock();
        if clock_events.cpu_cycle {
            self.triangle.clock_cpu();
        }
        if clock_events.apu_cycle {
            self.pulse[0].clock_apu();
            self.pulse[1].clock_apu();
            self.noise.clock_apu();
        }
        if clock_events.quarter_frame {
            self.pulse[0].clock_quarter_frame();
            self.pulse[1].clock_quarter_frame();
            self.triangle.clock_quarter_frame();
            self.noise.clock_quarter_frame();
        }
        if clock_events.half_frame {
            self.pulse[0].clock_half_frame();
            self.pulse[1].clock_half_frame();
            self.triangle.clock_half_frame();
            self.noise.clock_half_frame();
        }

        self.pulse[0].clock_ppu();
        self.pulse[1].clock_ppu();
    }

    pub fn get_output_sample(&self) -> f32 {
        self.mixer.sample(
            self.pulse[0].sample(),
            self.pulse[1].sample(),
            self.triangle.sample(),
            self.noise.sample(),
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

#[derive(Deserialize, PartialEq, Serialize)]
enum FrameCounterMode {
    Step4,
    Step5,
}

#[derive(Deserialize, Serialize)]
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

        if self.ppu_clock_counter.is_multiple_of(3) {
            clock_events.cpu_cycle = true;
        }

        if self.ppu_clock_counter.is_multiple_of(6) {
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
