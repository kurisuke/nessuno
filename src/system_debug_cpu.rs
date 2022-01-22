use crate::bus::CpuBus;
use crate::cpu::{Cpu, Disassembly};

pub struct SystemDebugCpu {
    memory: MemoryDebugCpu,
    pub cpu: Cpu,
}

impl Default for SystemDebugCpu {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemDebugCpu {
    pub fn new() -> SystemDebugCpu {
        SystemDebugCpu {
            memory: MemoryDebugCpu::new(),
            cpu: Cpu::new(),
        }
    }

    pub fn load_from_str(&mut self, s: &str, addr: u16) {
        for (offset, n) in s.split_ascii_whitespace().enumerate() {
            self.memory.ram[addr as usize + offset] = u8::from_str_radix(n, 16).unwrap();
        }
    }

    pub fn set_reset_vector(&mut self, addr: u16) {
        self.memory.ram[0xfffc] = (addr & 0x00ff) as u8;
        self.memory.ram[0xfffd] = ((addr >> 8) & 0x00ff) as u8;
    }

    pub fn cpu_step(&mut self) {
        loop {
            self.cpu.clock(&mut self.memory);
            if self.cpu.complete() {
                break;
            }
        }
    }

    pub fn cpu_reset(&mut self) {
        self.cpu.reset(&mut self.memory);
    }

    pub fn cpu_irq(&mut self) {
        self.cpu.irq(&mut self.memory);
    }

    pub fn cpu_nmi(&mut self) {
        self.cpu.nmi(&mut self.memory);
    }

    pub fn cpu_disassemble(&self, addr_start: u16, addr_stop: u16) -> Disassembly {
        self.cpu.disassemble(&self.memory, addr_start, addr_stop)
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.memory.ram[addr as usize]
    }
}

struct MemoryDebugCpu {
    ram: [u8; 64 * 1024],
}

impl MemoryDebugCpu {
    fn new() -> MemoryDebugCpu {
        MemoryDebugCpu {
            ram: [0; 64 * 1024],
        }
    }
}

impl CpuBus for MemoryDebugCpu {
    fn cpu_write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }

    fn cpu_read(&mut self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn cpu_read_ro(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }
}
