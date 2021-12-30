mod instr;

use crate::bus::Bus;

pub struct Cpu {
    bus: Box<dyn Bus>,

    a: u8,      // accumulator register
    x: u8,      // X register
    y: u8,      // Y register
    stkp: u8,   // stack pointer (points to location on bus)
    pc: u16,    // program counter
    status: u8, // status register
}

impl Cpu {
    pub fn new(bus: Box<dyn Bus>) -> Cpu {
        Cpu {
            bus,
            a: 0x00,
            x: 0x00,
            y: 0x00,
            stkp: 0x00,
            pc: 0x0000,
            status: 0x00,
        }
    }

    fn read(&self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.bus.write(addr, data);
    }

    fn get_flag(&self, f: Flag) -> bool {
        self.status & f.mask() != 0
    }

    fn set_flag(&mut self, f: Flag, v: bool) {
        if v {
            // set flag
            self.status |= f.mask();
        } else {
            // clear flag
            self.status &= !f.mask();
        }
    }

    fn clock(&mut self) {}

    fn reset(&mut self) {}

    fn irq(&mut self) {}

    fn nmi(&mut self) {}

    fn fetch(&mut self) -> u8 {
        0
    }
}

enum Flag {
    C, // Carry bit
    Z, // Zero
    I, // Disable interrupts
    D, // Decimal mode (not implemented, as not used on NES)
    B, // Break
    U, // Unused
    V, // Overflow
    N, // Negative
}

impl Flag {
    fn mask(&self) -> u8 {
        match self {
            Flag::C => 1 << 0,
            Flag::Z => 1 << 1,
            Flag::I => 1 << 2,
            Flag::D => 1 << 3,
            Flag::B => 1 << 4,
            Flag::U => 1 << 5,
            Flag::V => 1 << 6,
            Flag::N => 1 << 7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::dummy::DummyBus;

    #[test]
    fn test_flags() {
        let mut cpu = Cpu::new(Box::new(DummyBus::new()));
        cpu.set_flag(Flag::I, true);
        cpu.set_flag(Flag::D, true);
        cpu.set_flag(Flag::V, true);
        assert_eq!(cpu.get_flag(Flag::I), true);
        assert_eq!(cpu.get_flag(Flag::D), true);
        assert_eq!(cpu.get_flag(Flag::V), true);
        assert_eq!(cpu.get_flag(Flag::N), false);
        assert_eq!(cpu.get_flag(Flag::Z), false);

        cpu.set_flag(Flag::I, false);
        cpu.set_flag(Flag::D, true);
        cpu.set_flag(Flag::V, false);
        assert_eq!(cpu.get_flag(Flag::I), false);
        assert_eq!(cpu.get_flag(Flag::D), true);
        assert_eq!(cpu.get_flag(Flag::V), false);
        assert_eq!(cpu.get_flag(Flag::N), false);
        assert_eq!(cpu.get_flag(Flag::Z), false);
    }
}
