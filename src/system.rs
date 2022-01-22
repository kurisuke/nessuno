use crate::apu::Apu;
use crate::bus::CpuBus;
use crate::cartridge::Cartridge;
use crate::controller::{Controller, ControllerInput};
use crate::cpu::{Cpu, Disassembly};
use crate::ppu::{PatternTable, Ppu, SetPixel};

const TIME_PER_CLOCK: f64 = 1f64 / 5369318f64; // PPU Clock freq

pub struct System {
    pub cpu: Cpu,
    bus: Bus,

    clock_counter: usize,
    time_per_sample: f64,
    time_audio: f64,
}

struct Bus {
    ram_cpu: [u8; 2 * 1024],
    ppu: Ppu,
    apu: Apu,
    cart: Cartridge,
    controller: [Controller; 2],

    dma_page: u8,
    dma_addr: u8,
    dma_data: u8,
    dma_transfer: bool,
    dma_start_wait: bool,
}

pub struct SystemClockResult {
    pub set_pixel: Option<SetPixel>,
    pub frame_complete: bool,
    pub cpu_complete: bool,
    pub audio_sample: Option<f32>,
}

impl System {
    pub fn new(cart: Cartridge, sample_rate: u32) -> System {
        System {
            cpu: Cpu::new(),
            bus: Bus {
                ram_cpu: [0; 2 * 1024],
                ppu: Ppu::new(),
                apu: Apu::new(),
                cart,
                controller: [Controller::new(), Controller::new()],

                dma_page: 0x00,
                dma_addr: 0x00,
                dma_data: 0x00,
                dma_transfer: false,
                dma_start_wait: true,
            },
            clock_counter: 0,
            time_per_sample: 1f64 / (sample_rate as f64),
            time_audio: 0f64,
        }
    }

    pub fn clock(&mut self) -> SystemClockResult {
        let mut res = SystemClockResult {
            set_pixel: None,
            frame_complete: false,
            cpu_complete: false,
            audio_sample: None,
        };

        let ppu_res = self.bus.ppu.clock(&mut self.bus.cart);
        self.bus.apu.clock();
        res.set_pixel = ppu_res.set_pixel;
        res.frame_complete = ppu_res.frame_complete;

        if self.clock_counter % 3 == 0 {
            if self.bus.dma_transfer {
                if self.bus.dma_start_wait {
                    // wait for start: after first odd cycle
                    if self.clock_counter % 2 == 1 {
                        self.bus.dma_start_wait = false;
                    }
                } else if self.clock_counter % 2 == 0 {
                    // even cycle: read from cpu
                    self.bus.dma_data = self
                        .bus
                        .cpu_read(((self.bus.dma_page as u16) << 8) | (self.bus.dma_addr as u16));
                } else {
                    // odd cycle: write to ppu
                    self.bus.ppu.write_oam(self.bus.dma_addr, self.bus.dma_data);
                    if self.bus.dma_addr == 255 {
                        self.bus.dma_addr = 0;
                        self.bus.dma_transfer = false;
                        self.bus.dma_start_wait = true;
                    } else {
                        self.bus.dma_addr += 1;
                    }
                }
            } else {
                self.cpu.clock(&mut self.bus);
            }
        }

        // audio sync
        self.time_audio += TIME_PER_CLOCK;
        if self.time_audio >= self.time_per_sample {
            self.time_audio -= self.time_per_sample;
            res.audio_sample = Some(self.bus.apu.get_output_sample());
        }

        if ppu_res.nmi {
            self.cpu.nmi(&mut self.bus);
        }

        if self.bus.cart.irq_state() {
            self.bus.cart.irq_clear();
            self.cpu.irq(&mut self.bus);
        }

        res.cpu_complete = self.cpu.complete();
        self.clock_counter += 1;

        res
    }

    pub fn frame(&mut self, wait_cpu_complete: bool) {
        let mut cpu_complete = loop {
            let clock_res = self.clock();
            if clock_res.frame_complete {
                break clock_res.cpu_complete;
            }
        };

        if wait_cpu_complete {
            while !cpu_complete {
                cpu_complete = self.clock().cpu_complete;
            }
        }
    }

    pub fn step(&mut self) {
        // Run cycles until the current CPU instruction has executed
        loop {
            let clock_res = self.clock();
            if clock_res.cpu_complete {
                break;
            }
        }

        // Run additional system clock cycles (e.g. PPU) until the next CPU instruction starts
        loop {
            let clock_res = self.clock();
            if !clock_res.cpu_complete {
                break;
            }
        }
    }

    pub fn reset(&mut self) {
        self.bus.cart.reset();
        self.cpu.reset(&mut self.bus);
        self.bus.ppu.reset();
        self.clock_counter = 0;
    }

    pub fn cpu_irq(&mut self) {
        self.cpu.irq(&mut self.bus);
    }

    pub fn cpu_nmi(&mut self) {
        self.cpu.nmi(&mut self.bus);
    }

    pub fn cpu_disassemble(&self, addr_start: u16, addr_stop: u16) -> Disassembly {
        self.cpu.disassemble(&self.bus, addr_start, addr_stop)
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.bus.cpu_read_ro(addr)
    }

    pub fn ppu_get_pattern_table(&mut self, table_idx: usize, palette: usize) -> PatternTable {
        self.bus
            .ppu
            .get_pattern_table(&mut self.bus.cart, table_idx, palette)
    }

    pub fn ppu_get_color_from_palette(&mut self, palette: usize, pixel_value: u8) -> usize {
        self.bus
            .ppu
            .get_color_from_palette(&mut self.bus.cart, palette, pixel_value)
    }

    pub fn controller_update(&mut self, input1: &[ControllerInput], input2: &[ControllerInput]) {
        self.bus.controller[0].update(input1);
        self.bus.controller[1].update(input2);
    }

    pub fn ppu_debug_oam(&self, entry: usize) -> String {
        self.bus.ppu.debug_oam(entry)
    }
}

impl CpuBus for Bus {
    fn cpu_write(&mut self, addr: u16, data: u8) {
        if !self.cart.cpu_write(addr, data) {
            match addr {
                // CPU Ram
                0x0000..=0x1fff => {
                    self.ram_cpu[(addr & 0x07ff) as usize] = data;
                }
                0x2000..=0x3fff => {
                    self.ppu.cpu_write(&mut self.cart, addr & 0x0007, data);
                }
                0x4000..=0x4013 | 0x4015 | 0x4017 => {
                    self.apu.cpu_write(addr, data);
                }
                0x4014 => {
                    self.dma_page = data;
                    self.dma_addr = 0x00;
                    self.dma_transfer = true;
                }
                0x4016 => {
                    self.controller[0].write();
                    self.controller[1].write();
                }
                _ => {}
            }
        }
    }

    fn cpu_read(&mut self, addr: u16) -> u8 {
        if let Some(data) = self.cart.cpu_read(addr) {
            data
        } else {
            match addr {
                // CPU Ram
                0x0000..=0x1fff => self.ram_cpu[(addr & 0x07ff) as usize],
                0x2000..=0x3fff => self.ppu.cpu_read(&mut self.cart, addr & 0x0007),
                0x4015 => self.apu.cpu_read(addr),
                0x4016..=0x4017 => self.controller[(addr & 0x0001) as usize].read(),
                _ => 0,
            }
        }
    }

    fn cpu_read_ro(&self, addr: u16) -> u8 {
        if let Some(data) = self.cart.cpu_read_ro(addr) {
            data
        } else {
            match addr {
                // CPU Ram
                0x0000..=0x1fff => self.ram_cpu[(addr & 0x07ff) as usize],
                0x2000..=0x3fff => self.ppu.cpu_read_ro(addr & 0x0007),
                0x4015 => self.apu.cpu_read(addr),
                0x4016..=0x4017 => self.controller[(addr & 0x0001) as usize].read_ro(),
                _ => 0,
            }
        }
    }
}
