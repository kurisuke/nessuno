use crate::bus::CpuBus;
use crate::cartridge::Cartridge;
use crate::cpu::{Cpu, Disassembly};
use crate::ppu::Ppu;

pub struct System {
    pub cpu: Cpu,
    bus: Bus,

    clock_counter: usize,
}

struct Bus {
    ram_cpu: [u8; 2 * 1024],
    ppu: Ppu,
    cart: Cartridge,
}

impl System {
    pub fn new(cart: Cartridge) -> System {
        System {
            cpu: Cpu::new(),
            bus: Bus {
                ram_cpu: [0; 2 * 1024],
                ppu: Ppu::new(),
                cart,
            },
            clock_counter: 0,
        }
    }

    pub fn step(&mut self) {
        loop {
            self.cpu.clock(&mut self.bus);
            if self.cpu.complete() {
                break;
            }
        }
    }

    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.bus);
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

    pub fn read(&mut self, addr: u16) -> u8 {
        self.bus.cpu_read(addr)
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
                    self.ppu.cpu_write(addr & 0x0007, data);
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
                0x2000..=0x3fff => self.ppu.cpu_read(addr & 0x0007),
                _ => 0,
            }
        }
    }
}
