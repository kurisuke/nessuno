use crate::bus::CpuBus;
use crate::cartridge::Cartridge;
use crate::controller::{Controller, ControllerInput};
use crate::cpu::{Cpu, Disassembly};
use crate::ppu::{PatternTable, PixelRgba, Ppu, PpuRenderParams};

pub struct System {
    pub cpu: Cpu,
    bus: Bus,

    clock_counter: usize,
}

struct Bus {
    ram_cpu: [u8; 2 * 1024],
    ppu: Ppu,
    cart: Cartridge,
    controller: [Controller; 2],

    dma_page: u8,
    dma_addr: u8,
    dma_data: u8,
    dma_transfer: bool,
    dma_dummy: bool,
}

impl System {
    pub fn new(cart: Cartridge, render_params: PpuRenderParams) -> System {
        System {
            cpu: Cpu::new(),
            bus: Bus {
                ram_cpu: [0; 2 * 1024],
                ppu: Ppu::new(render_params),
                cart,
                controller: [Controller::new(), Controller::new()],

                dma_page: 0x00,
                dma_addr: 0x00,
                dma_data: 0x00,
                dma_transfer: false,
                dma_dummy: true,
            },
            clock_counter: 0,
        }
    }

    pub fn clock(&mut self, frame: &mut [u8]) {
        let nmi = self.bus.ppu.clock(&mut self.bus.cart, frame);
        if self.clock_counter % 3 == 0 {
            if self.bus.dma_transfer {
                if self.bus.dma_dummy {
                    if self.clock_counter % 2 == 1 {
                        self.bus.dma_dummy = false;
                    }
                } else {
                    if self.clock_counter % 2 == 0 {
                        // even cycle: read from cpu
                        self.bus.dma_data = self.bus.cpu_read(
                            ((self.bus.dma_page as u16) << 8) | (self.bus.dma_addr as u16),
                        );
                    } else {
                        // odd cycle: write to ppu
                        self.bus.ppu.write_oam(self.bus.dma_addr, self.bus.dma_data);
                        if self.bus.dma_addr == 255 {
                            self.bus.dma_addr = 0;
                            self.bus.dma_transfer = false;
                            self.bus.dma_dummy = true;
                        } else {
                            self.bus.dma_addr += 1;
                        }
                    }
                }
            } else {
                self.cpu.clock(&mut self.bus);
            }
        }

        if nmi {
            self.cpu.nmi(&mut self.bus);
        }

        self.clock_counter += 1;
    }

    pub fn frame(&mut self, frame: &mut [u8], wait_cpu_complete: bool) {
        loop {
            self.clock(frame);
            if self.bus.ppu.frame_complete {
                break;
            }
        }

        if wait_cpu_complete {
            while !self.cpu.complete() {
                self.clock(frame);
            }
        }

        self.bus.ppu.frame_complete = false;
    }

    pub fn step(&mut self, frame: &mut [u8]) {
        // Run cycles until the current CPU instruction has executed
        loop {
            self.clock(frame);
            if self.cpu.complete() {
                break;
            }
        }

        // Run additional system clock cycles (e.g. PPU) until the next CPU instruction starts
        while self.cpu.complete() {
            self.clock(frame);
        }
    }

    pub fn step_count(&mut self, frame: &mut [u8], count: usize) {
        for _ in 0..count {
            self.step(frame);
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

    pub fn cpu_disassemble(&mut self, addr_start: u16, addr_stop: u16) -> Disassembly {
        self.cpu.disassemble(&mut self.bus, addr_start, addr_stop)
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.bus.cpu_read_ro(addr)
    }

    pub fn ppu_get_pattern_table(&mut self, table_idx: usize, palette: usize) -> PatternTable {
        self.bus
            .ppu
            .get_pattern_table(&mut self.bus.cart, table_idx, palette)
    }

    pub fn ppu_get_color_from_palette(&mut self, palette: usize, pixel_value: u8) -> &PixelRgba {
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
                0x4014 => {
                    self.dma_page = data;
                    self.dma_addr = 0x00;
                    self.dma_transfer = true;
                }
                0x4016..=0x4017 => {
                    self.controller[(addr & 0x0001) as usize].write();
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
                0x4016..=0x4017 => self.controller[(addr & 0x0001) as usize].read_ro(),
                _ => 0,
            }
        }
    }
}
