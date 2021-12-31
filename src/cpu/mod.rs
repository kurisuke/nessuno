mod instr;

use crate::bus::Bus;
use instr::{AddrMode, Instr, Op, INSTR_LOOKUP};
use std::num::Wrapping;

pub struct Cpu {
    bus: Box<dyn Bus>,

    a: u8,      // accumulator register
    x: u8,      // X register
    y: u8,      // Y register
    stkp: u8,   // stack pointer (points to location on bus)
    pc: u16,    // program counter
    status: u8, // status register

    cycles: u8,
    fetched: u8,
    addr_abs: u16,
    addr_rel: u16,
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

            cycles: 0,
            fetched: 0x00,
            addr_abs: 0x0000,
            addr_rel: 0x0000,
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

    pub fn clock(&mut self) {
        if self.cycles == 0 {
            let opcode = self.read(self.pc);
            self.pc += 1;

            let instr = &INSTR_LOOKUP[opcode as usize];
            self.cycles = instr.cycles;

            let add_cycle_addr = self.addr_mode(&instr.addr_mode);
            let add_cycle_op = self.op(opcode, instr);
            self.cycles += (add_cycle_addr & add_cycle_op) as u8;
        }

        self.cycles -= 1;
    }

    fn addr_mode(&mut self, addr_mode: &AddrMode) -> bool {
        match addr_mode {
            &AddrMode::Imp => {
                self.fetched = self.a;
                false
            }
            &AddrMode::Imm => {
                self.addr_abs = self.pc;
                self.pc += 1;
                false
            }
            &AddrMode::Zp0 => {
                self.addr_abs = self.read(self.pc) as u16;
                self.pc += 1;
                false
            }
            &AddrMode::Zpx => {
                self.addr_abs = self.read(self.pc) as u16 + self.x as u16;
                self.addr_abs &= 0x00ff;
                self.pc += 1;
                false
            }
            &AddrMode::Zpy => {
                self.addr_abs = self.read(self.pc) as u16 + self.y as u16;
                self.addr_abs &= 0x00ff;
                self.pc += 1;
                false
            }
            &AddrMode::Rel => {
                self.addr_rel = self.read(self.pc) as u16;
                self.pc += 1;
                if self.addr_rel & 0x80 > 0 {
                    self.addr_rel |= 0xff00;
                }
                false
            }
            &AddrMode::Abs => {
                let lo = self.read(self.pc) as u16;
                self.pc += 1;
                let hi = self.read(self.pc) as u16;
                self.pc += 1;

                self.addr_abs = (hi << 8) | lo;

                false
            }
            &AddrMode::Abx => {
                let lo = self.read(self.pc) as u16;
                self.pc += 1;
                let hi = self.read(self.pc) as u16;
                self.pc += 1;

                self.addr_abs = (hi << 8) | lo;
                self.addr_abs += self.x as u16;

                // if high byte has changed (page changed), need additional clock cycle
                if (self.addr_abs & 0xff00) != (hi << 8) {
                    true
                } else {
                    false
                }
            }
            &AddrMode::Aby => {
                let lo = self.read(self.pc) as u16;
                self.pc += 1;
                let hi = self.read(self.pc) as u16;
                self.pc += 1;

                self.addr_abs = (hi << 8) | lo;
                self.addr_abs += self.y as u16;

                // if high byte has changed (page changed), need additional clock cycle
                if (self.addr_abs & 0xff00) != (hi << 8) {
                    true
                } else {
                    false
                }
            }
            &AddrMode::Ind => {
                let ptr_lo = self.read(self.pc) as u16;
                self.pc += 1;
                let ptr_hi = self.read(self.pc) as u16;
                self.pc += 1;

                let ptr = (ptr_hi << 8) | ptr_lo;

                if ptr_lo == 0x00ff {
                    // page boundary hw bug
                    self.addr_abs = ((self.read(ptr & 0xff00) as u16) << 8) | self.read(ptr) as u16;
                } else {
                    self.addr_abs = ((self.read(ptr + 1) as u16) << 8) | self.read(ptr) as u16;
                }
                false
            }
            &AddrMode::Izx => {
                let t = self.read(self.pc) as u16;
                self.pc += 1;

                let lo = self.read((t + self.x as u16) & 0x00ff) as u16;
                let hi = self.read((t + self.x as u16 + 1) & 0x00ff) as u16;

                self.addr_abs = (hi << 8) | lo;

                false
            }
            &AddrMode::Izy => {
                let t = self.read(self.pc) as u16;
                self.pc += 1;

                let lo = self.read(t & 0x00ff) as u16;
                let hi = self.read((t + 1) & 0x00ff) as u16;

                self.addr_abs = (hi << 8) | lo;
                self.addr_abs += self.y as u16;

                // if high byte has changed (page changed), need additional clock cycle
                if (self.addr_abs & 0xff00) != (hi << 8) {
                    true
                } else {
                    false
                }
            }
        }
    }

    fn op(&mut self, opcode: u8, instr: &Instr) -> bool {
        match &instr.op {
            &Op::Adc => {
                // Add with Carry In
                self.fetch(&instr.addr_mode);
                let tmp = self.a as u16 + self.fetched as u16 + self.get_flag(Flag::C) as u16;
                self.set_flag(Flag::C, tmp > 0x00ff);
                self.set_flag(Flag::Z, tmp & 0x00ff == 0);
                self.set_flag(Flag::N, tmp & 0x0080 != 0);
                self.set_flag(
                    Flag::V,
                    !(self.a as u16 ^ self.fetched as u16) & (self.a as u16 ^ tmp) & 0x0080 != 0,
                );
                self.a = (tmp & 0x00ff) as u8;
                true
            }
            &Op::And => {
                // Bitwise Logical AND
                self.fetch(&instr.addr_mode);
                self.a &= self.fetched;
                self.set_flag(Flag::Z, self.a == 0x00);
                self.set_flag(Flag::N, self.a & 0x80 != 0);
                true
            }
            &Op::Asl => {
                // Arithmetic shift left
                self.fetch(&instr.addr_mode);
                let tmp = (self.fetched as u16) << 1;
                self.set_flag(Flag::C, (tmp & 0xff00) > 0);
                self.set_flag(Flag::Z, (tmp & 0x00ff) == 0);
                self.set_flag(Flag::N, tmp & 0x0080 != 0);

                if instr.addr_mode == AddrMode::Imp {
                    self.a = (tmp & 0x00ff) as u8;
                } else {
                    self.write(self.addr_abs, (tmp & 0x00ff) as u8);
                }

                false
            }
            &Op::Bcc => {
                // Branch if Carry Clear
                if !self.get_flag(Flag::C) {
                    self.branch();
                }
                false
            }
            &Op::Bcs => {
                // Branch if Carry Set
                if self.get_flag(Flag::C) {
                    self.branch();
                }
                false
            }
            &Op::Beq => {
                // Branch if Equal
                if self.get_flag(Flag::Z) {
                    self.branch();
                }
                false
            }
            &Op::Bit => {
                // Bit testing ??
                self.fetch(&instr.addr_mode);
                let tmp = self.a & self.fetched;
                self.set_flag(Flag::Z, tmp == 0x00);
                self.set_flag(Flag::N, self.fetched & (1 << 7) != 0);
                self.set_flag(Flag::V, self.fetched & (1 << 6) != 0);
                false
            }
            &Op::Bmi => {
                // Branch if Negative
                if self.get_flag(Flag::N) {
                    self.branch();
                }
                false
            }
            &Op::Bne => {
                // Branch if Not Equal
                if !self.get_flag(Flag::Z) {
                    self.branch();
                }
                false
            }
            &Op::Bpl => {
                // Branch if Positive
                if !self.get_flag(Flag::N) {
                    self.branch();
                }
                false
            }
            &Op::Brk => {
                // Break (Program sourced interrupt)
                self.pc += 1;

                self.set_flag(Flag::I, true);
                self.write(0x0100 + self.stkp as u16, ((self.pc >> 8) & 0x00ff) as u8);
                self.stkp -= 1;
                self.write(0x0100 + self.stkp as u16, (self.pc & 0x00ff) as u8);
                self.stkp -= 1;

                self.set_flag(Flag::B, true);
                self.write(0x0100 + self.stkp as u16, self.status);
                self.stkp -= 1;
                self.set_flag(Flag::B, false);

                let lo = self.read(0xfffe) as u16;
                let hi = self.read(0xffff) as u16;
                self.pc = (hi << 8) | lo;

                false
            }
            &Op::Bvc => {
                // Branch if Overflow Clear
                if !self.get_flag(Flag::V) {
                    self.branch();
                }
                false
            }
            &Op::Bvs => {
                // Branch if Overflow Set
                if self.get_flag(Flag::V) {
                    self.branch();
                }
                false
            }
            &Op::Clc => {
                // Clear Carry Flag
                self.set_flag(Flag::C, false);
                false
            }
            &Op::Cld => {
                // Clear Decimal Flag
                self.set_flag(Flag::D, false);
                false
            }
            &Op::Cli => {
                // Disable Interrupts / Clear Interrupt Flag
                self.set_flag(Flag::I, false);
                false
            }
            &Op::Clv => {
                // Clear Overflow Flag
                self.set_flag(Flag::V, false);
                false
            }
            &Op::Cmp => {
                // Compare Accumulator
                self.fetch(&instr.addr_mode);
                let tmp = (Wrapping(self.a as u16) - Wrapping(self.fetched as u16)).0;
                self.set_flag(Flag::C, self.a >= self.fetched);
                self.set_flag(Flag::Z, tmp & 0x00ff == 0);
                self.set_flag(Flag::N, tmp & 0x0080 != 0);
                true
            }
            &Op::Cpx => {
                // Compare X register
                self.fetch(&instr.addr_mode);
                let tmp = (Wrapping(self.x as u16) - Wrapping(self.fetched as u16)).0;
                self.set_flag(Flag::C, self.x >= self.fetched);
                self.set_flag(Flag::Z, tmp & 0x00ff == 0);
                self.set_flag(Flag::N, tmp & 0x0080 != 0);
                false
            }
            &Op::Cpy => {
                // Compare Y register
                self.fetch(&instr.addr_mode);
                let tmp = (Wrapping(self.y as u16) - Wrapping(self.fetched as u16)).0;
                self.set_flag(Flag::C, self.y >= self.fetched);
                self.set_flag(Flag::Z, tmp & 0x00ff == 0);
                self.set_flag(Flag::N, tmp & 0x0080 != 0);
                false
            }
            &Op::Dec => {
                // Decrement Value at Memory Location
                self.fetch(&instr.addr_mode);
                let tmp = (Wrapping(self.fetched) - Wrapping(1)).0;
                self.write(self.addr_abs, (tmp & 0x00ff) as u8);
                self.set_flag(Flag::Z, tmp & 0x00ff == 0);
                self.set_flag(Flag::N, tmp & 0x0080 != 0);
                false
            }
            &Op::Dex => {
                // Decrement X Register
                self.x = (Wrapping(self.x) - Wrapping(1)).0;
                self.set_flag(Flag::Z, self.x & 0x00ff == 0);
                self.set_flag(Flag::N, self.x & 0x0080 != 0);
                false
            }
            &Op::Dey => {
                // Decrement Y Register
                self.x = (Wrapping(self.y) - Wrapping(1)).0;
                self.set_flag(Flag::Z, self.y & 0x00ff == 0);
                self.set_flag(Flag::N, self.y & 0x0080 != 0);
                false
            }
            &Op::Eor => {
                // Bitwise Logical AND
                self.fetch(&instr.addr_mode);
                self.a ^= self.fetched;
                self.set_flag(Flag::Z, self.a == 0x00);
                self.set_flag(Flag::N, self.a & 0x80 != 0);
                true
            }
            &Op::Inc => {
                // Increment Value at Memory Location
                self.fetch(&instr.addr_mode);
                let tmp = (Wrapping(self.fetched) + Wrapping(1)).0;
                self.write(self.addr_abs, (tmp & 0x00ff) as u8);
                self.set_flag(Flag::Z, tmp & 0x00ff == 0);
                self.set_flag(Flag::N, tmp & 0x0080 != 0);
                false
            }
            &Op::Inx => {
                // Increment X Register
                self.x = (Wrapping(self.x) + Wrapping(1)).0;
                self.set_flag(Flag::Z, self.x & 0x00ff == 0);
                self.set_flag(Flag::N, self.x & 0x0080 != 0);
                false
            }
            &Op::Iny => {
                // Increment Y Register
                self.x = (Wrapping(self.y) + Wrapping(1)).0;
                self.set_flag(Flag::Z, self.y & 0x00ff == 0);
                self.set_flag(Flag::N, self.y & 0x0080 != 0);
                false
            }
            &Op::Jmp => {
                // Jump To Location
                self.pc = self.addr_abs;
                false
            }
            &Op::Jsr => {
                // Jump To Sub-Routine
                self.pc -= 1;

                self.write(0x0100 + self.stkp as u16, ((self.pc >> 8) & 0x00ff) as u8);
                self.stkp -= 1;
                self.write(0x0100 + self.stkp as u16, (self.pc & 0x00ff) as u8);
                self.stkp -= 1;

                self.pc = self.addr_abs;
                false
            }
            &Op::Lda => {
                // Load Accumulator
                self.fetch(&instr.addr_mode);
                self.a = self.fetched;
                self.set_flag(Flag::Z, self.a & 0x00ff == 0);
                self.set_flag(Flag::N, self.a & 0x0080 != 0);
                true
            }
            &Op::Ldx => {
                // Load X Register
                self.fetch(&instr.addr_mode);
                self.x = self.fetched;
                self.set_flag(Flag::Z, self.x & 0x00ff == 0);
                self.set_flag(Flag::N, self.x & 0x0080 != 0);
                true
            }
            &Op::Ldy => {
                // Load Y Register
                self.fetch(&instr.addr_mode);
                self.y = self.fetched;
                self.set_flag(Flag::Z, self.y & 0x00ff == 0);
                self.set_flag(Flag::N, self.y & 0x0080 != 0);
                true
            }
            &Op::Lsr => {
                // Logical Shift Right
                self.fetch(&instr.addr_mode);
                self.set_flag(Flag::C, self.fetched & 0x01 > 0);

                let tmp = self.fetched >> 1;
                self.set_flag(Flag::Z, tmp == 0);
                self.set_flag(Flag::N, tmp != 0);

                if instr.addr_mode == AddrMode::Imp {
                    self.a = tmp;
                } else {
                    self.write(self.addr_abs, tmp);
                }
                false
            }
            &Op::Nop => {
                // No Operation
                // NOP variants differ by cycle length
                match opcode {
                    0x1c | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => true,
                    _ => false,
                }
            }
            &Op::Ora => {
                // Bitwise Logical OR
                self.fetch(&instr.addr_mode);
                self.a |= self.fetched;
                self.set_flag(Flag::Z, self.a == 0x00);
                self.set_flag(Flag::N, self.a & 0x80 != 0);
                true
            }
            &Op::Pha => {
                // Push Accumulator to Stack
                self.write(0x0100 + self.stkp as u16, self.a);
                self.stkp -= 1;
                false
            }
            &Op::Php => {
                // Push Status Register to Stack
                self.write(
                    0x0100 + self.stkp as u16,
                    self.status | Flag::B.mask() | Flag::U.mask(),
                );
                self.set_flag(Flag::B, false);
                self.set_flag(Flag::U, false);
                false
            }
            &Op::Pla => {
                // Pop Accumulator off Stack
                self.stkp += 1;
                self.a = self.read(0x0100 + self.stkp as u16);
                self.set_flag(Flag::Z, self.a == 0x00);
                self.set_flag(Flag::N, self.a & 0x80 != 0);
                false
            }
            &Op::Plp => {
                // Pop Status Register off Stack
                self.stkp += 1;
                self.status = self.read(0x0100 + self.stkp as u16);
                self.set_flag(Flag::U, true);
                false
            }
            &Op::Rol => {
                // Rotate Left
                self.fetch(&instr.addr_mode);
                let tmp = ((self.fetched as u16) << 1) | (self.get_flag(Flag::C) as u16);
                self.set_flag(Flag::C, tmp & 0xff00 != 0);
                self.set_flag(Flag::Z, tmp & 0x00ff == 0);
                self.set_flag(Flag::N, tmp & 0x0080 != 0);
                if instr.addr_mode == AddrMode::Imp {
                    self.a = (tmp & 0x00ff) as u8;
                } else {
                    self.write(self.addr_abs, (tmp & 0x00ff) as u8);
                }
                false
            }
            &Op::Ror => {
                // Rotate Right
                self.fetch(&instr.addr_mode);
                let tmp = ((self.fetched as u16) >> 1) | ((self.get_flag(Flag::C) as u16) << 7);
                self.set_flag(Flag::C, tmp & 0xff00 != 0);
                self.set_flag(Flag::Z, tmp & 0x00ff == 0);
                self.set_flag(Flag::N, tmp & 0x0080 != 0);
                if instr.addr_mode == AddrMode::Imp {
                    self.a = (tmp & 0x00ff) as u8;
                } else {
                    self.write(self.addr_abs, (tmp & 0x00ff) as u8);
                }
                false
            }
            &Op::Rti => {
                // Return from Interrupt
                self.stkp += 1;
                self.status = self.read(0x0100 + self.stkp as u16);
                self.status &= !Flag::B.mask();
                self.status &= !Flag::U.mask();

                self.stkp += 1;
                self.pc = self.read(0x100 + self.stkp as u16) as u16;
                self.stkp += 1;
                self.pc |= (self.read(0x100 + self.stkp as u16) as u16) << 8;
                false
            }
            &Op::Rts => {
                // Return from Subroutine
                self.stkp += 1;
                self.pc = self.read(0x100 + self.stkp as u16) as u16;
                self.stkp += 1;
                self.pc |= (self.read(0x100 + self.stkp as u16) as u16) << 8;
                false
            }
            &Op::Sbc => {
                // Substract With Carry
                self.fetch(&instr.addr_mode);
                let inv = (self.fetched as u16) ^ 0x00ff;

                let tmp = (Wrapping(self.a as u16)
                    + Wrapping(inv)
                    + Wrapping(self.get_flag(Flag::C) as u16))
                .0;
                self.set_flag(Flag::C, tmp > 0x00ff);
                self.set_flag(Flag::Z, tmp & 0x00ff == 0);
                self.set_flag(Flag::N, tmp & 0x0080 != 0);
                self.set_flag(
                    Flag::V,
                    (self.a as u16 ^ inv) & (self.a as u16 ^ tmp) & 0x0080 != 0,
                );
                self.a = (tmp & 0x00ff) as u8;
                true
            }
            &Op::Sec => {
                // Set Carry Flag
                self.set_flag(Flag::C, true);
                false
            }
            &Op::Sed => {
                // Set Decimal Flag
                self.set_flag(Flag::D, true);
                false
            }
            &Op::Sei => {
                // Set Interrupt Flag / Enable Interrupts
                self.set_flag(Flag::I, true);
                false
            }
            &Op::Sta => {
                // Store Accumulator at Address
                self.write(self.addr_abs, self.a);
                false
            }
            &Op::Stx => {
                // Store X Register at Address
                self.write(self.addr_abs, self.x);
                false
            }
            &Op::Sty => {
                // Store Y Register at Address
                self.write(self.addr_abs, self.y);
                false
            }
            &Op::Tax => {
                // Transfer Accumulator to X Register
                self.x = self.a;
                self.set_flag(Flag::Z, self.x == 0);
                self.set_flag(Flag::N, self.x & 0x80 != 0);
                false
            }
            &Op::Tay => {
                // Transfer Accumulator to Y Register
                self.y = self.a;
                self.set_flag(Flag::Z, self.y == 0);
                self.set_flag(Flag::N, self.y & 0x80 != 0);
                false
            }
            &Op::Tsx => {
                // Transfer Stack Pointer to X Register
                self.x = self.stkp;
                self.set_flag(Flag::Z, self.x == 0);
                self.set_flag(Flag::N, self.x & 0x80 != 0);
                false
            }
            &Op::Txa => {
                // Transfer X Register to Accumulator
                self.a = self.x;
                self.set_flag(Flag::Z, self.a == 0);
                self.set_flag(Flag::N, self.a & 0x80 != 0);
                false
            }
            &Op::Txs => {
                // Transfer X Register to Stack Pointer
                self.stkp = self.x;
                false
            }
            &Op::Tya => {
                // Transfer Y Register to Accumulator
                self.a = self.y;
                self.set_flag(Flag::Z, self.a == 0);
                self.set_flag(Flag::N, self.a & 0x80 != 0);
                false
            }
            &Op::Xxx => false, // ignore
        }
    }

    fn branch(&mut self) {
        self.cycles += 1;
        self.addr_abs = (Wrapping(self.pc) + Wrapping(self.addr_rel)).0;

        if (self.addr_abs & 0xff00) != (self.pc & 0xff00) {
            self.cycles += 1;
        }

        self.pc = self.addr_abs;
    }

    pub fn reset(&mut self) {
        self.a = 0x00;
        self.x = 0x00;
        self.y = 0x00;
        self.stkp = 0xfd;
        self.status = 0x00 | Flag::U.mask();

        self.addr_abs = 0xfffc;
        let lo = self.read(self.addr_abs) as u16;
        let hi = self.read(self.addr_abs + 1) as u16;
        self.pc = (hi << 8) | lo;

        self.addr_rel = 0x0000;
        self.addr_abs = 0x0000;
        self.fetched = 0x00;

        self.cycles = 8;
    }

    pub fn irq(&mut self) {
        if !self.get_flag(Flag::I) {
            self.write(0x0100 + self.stkp as u16, ((self.pc >> 8) & 0x00ff) as u8);
            self.stkp -= 1;
            self.write(0x0100 + self.stkp as u16, (self.pc & 0x00ff) as u8);
            self.stkp -= 1;

            self.set_flag(Flag::B, false);
            self.set_flag(Flag::U, true);
            self.set_flag(Flag::I, true);
            self.write(0x0100 + self.stkp as u16, self.status);
            self.stkp -= 1;

            self.addr_abs = 0xfffe;
            let lo = self.read(self.addr_abs) as u16;
            let hi = self.read(self.addr_abs + 1) as u16;
            self.pc = (hi << 8) | lo;

            self.cycles = 7;
        }
    }

    pub fn nmi(&mut self) {
        self.write(0x0100 + self.stkp as u16, ((self.pc >> 8) & 0x00ff) as u8);
        self.stkp -= 1;
        self.write(0x0100 + self.stkp as u16, (self.pc & 0x00ff) as u8);
        self.stkp -= 1;

        self.set_flag(Flag::B, false);
        self.set_flag(Flag::U, true);
        self.set_flag(Flag::I, true);
        self.write(0x0100 + self.stkp as u16, self.status);
        self.stkp -= 1;

        self.addr_abs = 0xfffa;
        let lo = self.read(self.addr_abs) as u16;
        let hi = self.read(self.addr_abs + 1) as u16;
        self.pc = (hi << 8) | lo;

        self.cycles = 8;
    }

    fn fetch(&mut self, addr_mode: &AddrMode) -> u8 {
        if addr_mode != &AddrMode::Imp {
            self.fetched = self.read(self.addr_abs);
        }
        self.fetched
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
        assert_eq!(!Flag::B.mask(), 0b11101111);
        assert_eq!(!Flag::U.mask(), 0b11011111);

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
