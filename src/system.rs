use crate::apu::Apu;
use crate::bus::CpuBus;
use crate::cartridge::Cartridge;
use crate::controller::{Controller, ControllerInput};
use crate::cpu::{Cpu, Disassembly};
use crate::ppu::{PatternTable, Ppu, SetPixel};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

/// time per PPU clock
const TIME_PER_CLOCK: f64 = 1f64 / 5369318f64;

#[derive(Deserialize, Serialize)]
/// representation of NES system hardware, including all components
pub struct System {
    /// 2A03 CPU (MOS 6502 compatible)
    pub cpu: Cpu,
    /// other components on CPU memory bus
    bus: Bus,

    /// clock counter (in PPU cycles)
    clock_counter: usize,
    /// rate at which APU emulation produces new samples (depends on audio driver output frequency)
    time_per_sample: f64,
    /// elapsed time since last audio sample producation
    time_audio: f64,
}

#[derive(Deserialize, Serialize)]
/// representation of all devices on CPU memory bus
struct Bus {
    #[serde(with = "BigArray")]
    /// on-board RAM (2 kB)
    ram_cpu: [u8; 2 * 1024],
    /// Picture Processing Unit (2C02)
    ppu: Ppu,
    /// Audio Processing Unit (on 2A03)
    apu: Apu,
    /// on-cartridge memory (RAM / ROM / io ports)
    cart: Cartridge,
    /// controller port
    controller: [Controller; 2],

    /// flag if DMA transfer active
    dma_transfer: bool,
    /// flag if waiting for cycle sync before DMA start
    dma_start_wait: bool,
    /// memory page for DMA transfer (hi byte of address)
    dma_page: u8,
    /// current DMA read offset (lo byte of address)
    dma_addr: u8,
    /// current DMA transfer data (cache)
    dma_data: u8,
}

/// result state after system clock
pub struct SystemClockResult {
    /// pixel information for screen update
    pub set_pixel: Option<SetPixel>,
    /// flag if completed video frame (scanline -1, position 0)
    pub frame_complete: bool,
    /// flag if completed CPU instruction
    pub cpu_complete: bool,
    /// audio sample (optional, produced based on audio driver output frequency)
    pub audio_sample: Option<f32>,
}

impl System {
    /// create NES system representation
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - audio driver output frequency in Hz. Will
    ///   determine APU sample production rate.
    ///
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

    /// advance system clock one PPU cycle
    ///
    /// Returns various result data for the presentation layer.
    ///
    pub fn clock(&mut self) -> SystemClockResult {
        let mut res = SystemClockResult {
            set_pixel: None,
            frame_complete: false,
            cpu_complete: false,
            audio_sample: None,
        };

        // run PPU cycle, forward results
        let ppu_res = self.bus.ppu.clock(&mut self.bus.cart);
        res.set_pixel = ppu_res.set_pixel;
        res.frame_complete = ppu_res.frame_complete;

        // APU cycle
        //
        // Note: although most APU components are updated at 1/2 CPU
        // clock rate, some functions (e.g. sweeps) are updated
        // quasi-analogous. Thus we run the APU "clock" at every cycle.
        self.bus.apu.clock();

        // CPU cycle if applicable. CPU runs at 1/3 of PPU rate
        if self.clock_counter % 3 == 0 {
            if self.bus.dma_transfer {
                // CPU interrupted by DMA, advance DMA
                self.handle_dma_transfer();
            } else {
                // regular CPU cycle
                self.cpu.clock(&mut self.bus);
            }
        }

        // Produce audio sample from APU, if required time elapsed
        self.time_audio += TIME_PER_CLOCK;
        if self.time_audio >= self.time_per_sample {
            self.time_audio -= self.time_per_sample;
            res.audio_sample = Some(self.bus.apu.get_output_sample());
        }

        // NMI triggered by PPU?
        if ppu_res.nmi {
            self.cpu.nmi(&mut self.bus);
        }

        // IRQ triggered by cartridge?
        if self.bus.cart.irq_state() {
            self.bus.cart.irq_clear();
            self.cpu.irq(&mut self.bus);
        }

        // notify if we completed a cpu instruction (for step)
        res.cpu_complete = self.cpu.complete();
        self.clock_counter += 1;

        res
    }

    /// reset all system components
    ///
    pub fn reset(&mut self) {
        self.bus.cart.reset();
        self.cpu.reset(&mut self.bus);
        self.bus.ppu.reset();
        self.clock_counter = 0;
    }

    /// manually trigger CPU maskable interrupct (IRQ)
    ///
    pub fn cpu_irq(&mut self) {
        self.cpu.irq(&mut self.bus);
    }

    /// manually trigger CPU non-maskable interrupct (NMI)
    ///
    pub fn cpu_nmi(&mut self) {
        self.cpu.nmi(&mut self.bus);
    }

    /// produce disassembly for address range (inclusive)
    ///
    /// # Arguments
    ///
    /// - `addr_start`, `addr_stop` - inclusive memory address range
    ///
    pub fn cpu_disassemble(&self, addr_start: u16, addr_stop: u16) -> Disassembly {
        self.cpu.disassemble(&self.bus, addr_start, addr_stop)
    }

    /// get PPU pattern table contents
    ///
    /// # Arguments
    ///
    /// - `table_idx` - index of the table (0 or 1)
    /// - `palette` - RAM palette number (0..7)
    ///
    pub fn ppu_get_pattern_table(&mut self, table_idx: usize, palette: usize) -> PatternTable {
        self.bus
            .ppu
            .get_pattern_table(&mut self.bus.cart, table_idx, palette)
    }

    /// get color from RAM palette (as hardware palette color)
    ///
    /// # Arguments
    ///
    /// - `palette` - RAM palette number (0..7)
    /// - `pixel_value` - offset in palette (0..3)
    ///
    pub fn ppu_get_color_from_palette(&mut self, palette: usize, pixel_value: u8) -> usize {
        self.bus
            .ppu
            .get_color_from_palette(&mut self.bus.cart, palette, pixel_value)
    }

    /// update controller ports with new input data
    ///
    pub fn controller_update(&mut self, input1: &[ControllerInput], input2: &[ControllerInput]) {
        self.bus.controller[0].update(input1);
        self.bus.controller[1].update(input2);
    }

    /// get debug info about PPU OAM (Object Attribute Memory)
    ///
    pub fn ppu_debug_oam(&self, entry: usize) -> String {
        self.bus.ppu.debug_oam(entry)
    }

    fn handle_dma_transfer(&mut self) {
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

            // end transfer and reset addr pointer after full no. of cycles
            if self.bus.dma_addr == 255 {
                self.bus.dma_addr = 0;
                self.bus.dma_transfer = false;
                self.bus.dma_start_wait = true;
            } else {
                self.bus.dma_addr += 1;
            }
        }
    }
}

impl CpuBus for Bus {
    fn cpu_write(&mut self, addr: u16, data: u8) {
        // first check if write is handled by cartridge
        if !self.cart.cpu_write(addr, data) {
            match addr {
                // CPU Ram
                0x0000..=0x1fff => {
                    self.ram_cpu[(addr & 0x07ff) as usize] = data;
                }
                // PPU Registers
                0x2000..=0x3fff => {
                    self.ppu.cpu_write(&mut self.cart, addr & 0x0007, data);
                }
                // APU Registers
                0x4000..=0x4013 | 0x4015 | 0x4017 => {
                    self.apu.cpu_write(addr, data);
                }
                // Direct Memory Access (DMA)
                0x4014 => {
                    self.dma_page = data;
                    self.dma_addr = 0x00;
                    self.dma_transfer = true;
                } // Controller Ports
                0x4016 => {
                    self.controller[0].write();
                    self.controller[1].write();
                }
                _ => {}
            }
        }
    }

    fn cpu_read(&mut self, addr: u16) -> u8 {
        // first check if read is handled by cartridge
        if let Some(data) = self.cart.cpu_read(addr) {
            data
        } else {
            match addr {
                // CPU Ram
                0x0000..=0x1fff => self.ram_cpu[(addr & 0x07ff) as usize],
                // PPU Registers
                0x2000..=0x3fff => self.ppu.cpu_read(&mut self.cart, addr & 0x0007),
                // APU Status
                0x4015 => self.apu.cpu_read(addr),
                // Controller Ports
                0x4016..=0x4017 => self.controller[(addr & 0x0001) as usize].read(),
                _ => 0,
            }
        }
    }

    fn cpu_read_ro(&self, addr: u16) -> u8 {
        // first check if read is handled by cartridge
        if let Some(data) = self.cart.cpu_read_ro(addr) {
            data
        } else {
            match addr {
                // CPU Ram
                0x0000..=0x1fff => self.ram_cpu[(addr & 0x07ff) as usize],
                // PPU Registers
                0x2000..=0x3fff => self.ppu.cpu_read_ro(addr & 0x0007),
                // APU Status
                0x4015 => self.apu.cpu_read(addr),
                // Controller Ports
                0x4016..=0x4017 => self.controller[(addr & 0x0001) as usize].read_ro(),
                _ => 0,
            }
        }
    }
}
