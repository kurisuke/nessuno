enum AddrMode {
    Imp,
    Imm,
    Zp0,
    Zpx,
    Zpy,
    Rel,
    Abs,
    Abx,
    Aby,
    Ind,
    Izx,
    Izy,
}

enum Op {
    Adc,
    And,
    Asl,
    Bcc,
    Bcs,
    Beq,
    Bit,
    Bmi,
    Bne,
    Bpl,
    Brk,
    Bvc,
    Bvs,
    Clc,
    Cld,
    Cli,
    Clv,
    Cmp,
    Cpx,
    Cpy,
    Dec,
    Dex,
    Dey,
    Eor,
    Inc,
    Inx,
    Iny,
    Jmp,
    Jsr,
    Lda,
    Ldx,
    Ldy,
    Lsr,
    Nop,
    Ora,
    Pha,
    Php,
    Pla,
    Plp,
    Rol,
    Ror,
    Rti,
    Rts,
    Sbc,
    Sec,
    Sed,
    Sei,
    Sta,
    Stx,
    Sty,
    Tax,
    Tay,
    Tsx,
    Txa,
    Txs,
    Tya,
    Xxx,
}

struct Instr<'a> {
    name: &'a str,
    op: Op,
    addr_mode: AddrMode,
    cycles: u8,
}

impl Instr<'_> {
    const fn new(name: &str, op: Op, addr_mode: AddrMode, cycles: u8) -> Instr {
        Instr {
            name,
            op,
            addr_mode,
            cycles,
        }
    }
}

const INSTR_LOOKUP: [Instr; 256] = [
    // opcodes 0x00..=0x0f
    Instr::new("BRK", Op::Brk, AddrMode::Imm, 7),
    Instr::new("ORA", Op::Ora, AddrMode::Izx, 6),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 8),
    Instr::new("???", Op::Nop, AddrMode::Imp, 3),
    Instr::new("ORA", Op::Ora, AddrMode::Zp0, 3),
    Instr::new("ASL", Op::Asl, AddrMode::Zp0, 5),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 5),
    Instr::new("PHP", Op::Php, AddrMode::Imp, 3),
    Instr::new("ORA", Op::Ora, AddrMode::Imm, 2),
    Instr::new("ASL", Op::Asl, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("???", Op::Nop, AddrMode::Imp, 4),
    Instr::new("ORA", Op::Ora, AddrMode::Abs, 4),
    Instr::new("ASL", Op::Asl, AddrMode::Abs, 6),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 6),
    // opcodes 0x10..=0x1f
    Instr::new("BPL", Op::Bpl, AddrMode::Rel, 2),
    Instr::new("ORA", Op::Ora, AddrMode::Izy, 5),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 8),
    Instr::new("???", Op::Nop, AddrMode::Imp, 4),
    Instr::new("ORA", Op::Ora, AddrMode::Zpx, 4),
    Instr::new("ASL", Op::Asl, AddrMode::Zpx, 6),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 6),
    Instr::new("CLC", Op::Clc, AddrMode::Imp, 2),
    Instr::new("ORA", Op::Ora, AddrMode::Aby, 4),
    Instr::new("???", Op::Nop, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 7),
    Instr::new("???", Op::Nop, AddrMode::Imp, 4),
    Instr::new("ORA", Op::Ora, AddrMode::Abx, 4),
    Instr::new("ASL", Op::Asl, AddrMode::Abx, 7),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 7),
    // opcodes 0x20..=0x2f
    Instr::new("JSR", Op::Jsr, AddrMode::Abs, 6),
    Instr::new("AND", Op::And, AddrMode::Izx, 6),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 8),
    Instr::new("BIT", Op::Bit, AddrMode::Zp0, 3),
    Instr::new("AND", Op::And, AddrMode::Zp0, 3),
    Instr::new("ROL", Op::Rol, AddrMode::Zp0, 5),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 5),
    Instr::new("PLP", Op::Plp, AddrMode::Imp, 4),
    Instr::new("AND", Op::And, AddrMode::Imm, 2),
    Instr::new("ROL", Op::Rol, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("BIT", Op::Bit, AddrMode::Abs, 4),
    Instr::new("AND", Op::And, AddrMode::Abs, 4),
    Instr::new("ROL", Op::Rol, AddrMode::Abs, 6),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 6),
    // opcodes 0x30..=0x3f
    Instr::new("BMI", Op::Bmi, AddrMode::Rel, 2),
    Instr::new("AND", Op::And, AddrMode::Izy, 5),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 8),
    Instr::new("???", Op::Nop, AddrMode::Imp, 4),
    Instr::new("AND", Op::And, AddrMode::Zpx, 4),
    Instr::new("ROL", Op::Rol, AddrMode::Zpx, 6),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 6),
    Instr::new("SEC", Op::Sec, AddrMode::Imp, 2),
    Instr::new("AND", Op::And, AddrMode::Aby, 4),
    Instr::new("???", Op::Nop, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 7),
    Instr::new("???", Op::Nop, AddrMode::Imp, 4),
    Instr::new("AND", Op::And, AddrMode::Abx, 4),
    Instr::new("ROL", Op::Rol, AddrMode::Abx, 7),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 7),
    // opcodes 0x40..=0x4f
    Instr::new("RTI", Op::Rti, AddrMode::Imp, 6),
    Instr::new("EOR", Op::Eor, AddrMode::Izx, 6),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 8),
    Instr::new("???", Op::Nop, AddrMode::Imp, 3),
    Instr::new("EOR", Op::Eor, AddrMode::Zp0, 3),
    Instr::new("LSR", Op::Lsr, AddrMode::Zp0, 5),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 5),
    Instr::new("PHA", Op::Pha, AddrMode::Imp, 3),
    Instr::new("EOR", Op::Eor, AddrMode::Imm, 2),
    Instr::new("LSR", Op::Lsr, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("JMP", Op::Jmp, AddrMode::Abs, 3),
    Instr::new("EOR", Op::Eor, AddrMode::Abs, 4),
    Instr::new("LSR", Op::Lsr, AddrMode::Abs, 6),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 6),
    // opcodes 0x50..=0x5f
    Instr::new("BVC", Op::Bvc, AddrMode::Rel, 2),
    Instr::new("EOR", Op::Eor, AddrMode::Izy, 5),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 8),
    Instr::new("???", Op::Nop, AddrMode::Imp, 4),
    Instr::new("EOR", Op::Eor, AddrMode::Zpx, 4),
    Instr::new("LSR", Op::Lsr, AddrMode::Zpx, 6),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 6),
    Instr::new("CLI", Op::Cli, AddrMode::Imp, 2),
    Instr::new("EOR", Op::Eor, AddrMode::Aby, 4),
    Instr::new("???", Op::Nop, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 7),
    Instr::new("???", Op::Nop, AddrMode::Imp, 4),
    Instr::new("EOR", Op::Eor, AddrMode::Abx, 4),
    Instr::new("LSR", Op::Lsr, AddrMode::Abx, 7),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 7),
    // opcodes 0x60..=0x6f
    Instr::new("RTS", Op::Rts, AddrMode::Imp, 6),
    Instr::new("ADC", Op::Adc, AddrMode::Izx, 6),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 8),
    Instr::new("???", Op::Nop, AddrMode::Imp, 3),
    Instr::new("ADC", Op::Adc, AddrMode::Zp0, 3),
    Instr::new("ROR", Op::Ror, AddrMode::Zp0, 5),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 5),
    Instr::new("PLA", Op::Pla, AddrMode::Imp, 4),
    Instr::new("ADC", Op::Adc, AddrMode::Imm, 2),
    Instr::new("ROR", Op::Ror, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("JMP", Op::Jmp, AddrMode::Ind, 5),
    Instr::new("ADC", Op::Adc, AddrMode::Abs, 4),
    Instr::new("ROR", Op::Ror, AddrMode::Abs, 6),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 6),
    // opcodes 0x70..=0x7f
    Instr::new("BVS", Op::Bvs, AddrMode::Rel, 2),
    Instr::new("ADC", Op::Adc, AddrMode::Izy, 5),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 8),
    Instr::new("???", Op::Nop, AddrMode::Imp, 4),
    Instr::new("ADC", Op::Adc, AddrMode::Zpx, 4),
    Instr::new("ROR", Op::Ror, AddrMode::Zpx, 6),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 6),
    Instr::new("SEI", Op::Sei, AddrMode::Imp, 2),
    Instr::new("ADC", Op::Adc, AddrMode::Aby, 4),
    Instr::new("???", Op::Nop, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 7),
    Instr::new("???", Op::Nop, AddrMode::Imp, 4),
    Instr::new("ADC", Op::Adc, AddrMode::Abx, 4),
    Instr::new("ROR", Op::Ror, AddrMode::Abx, 7),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 7),
    // opcodes 0x80..=0x8f
    Instr::new("???", Op::Nop, AddrMode::Imp, 2),
    Instr::new("STA", Op::Sta, AddrMode::Izx, 6),
    Instr::new("???", Op::Nop, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 6),
    Instr::new("STY", Op::Sty, AddrMode::Zp0, 3),
    Instr::new("STA", Op::Sta, AddrMode::Zp0, 3),
    Instr::new("STX", Op::Stx, AddrMode::Zp0, 3),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 3),
    Instr::new("DEY", Op::Dey, AddrMode::Imp, 2),
    Instr::new("???", Op::Nop, AddrMode::Imp, 2),
    Instr::new("TXA", Op::Txa, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("STY", Op::Sty, AddrMode::Abs, 4),
    Instr::new("STA", Op::Sta, AddrMode::Abs, 4),
    Instr::new("STX", Op::Stx, AddrMode::Abs, 4),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 4),
    // opcodes 0x90..=0x9f
    Instr::new("BCC", Op::Bcc, AddrMode::Rel, 2),
    Instr::new("STA", Op::Sta, AddrMode::Izy, 6),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 6),
    Instr::new("STY", Op::Sty, AddrMode::Zpx, 4),
    Instr::new("STA", Op::Sta, AddrMode::Zpx, 4),
    Instr::new("STX", Op::Stx, AddrMode::Zpy, 4),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 4),
    Instr::new("TYA", Op::Tya, AddrMode::Imp, 2),
    Instr::new("STA", Op::Sta, AddrMode::Aby, 5),
    Instr::new("TXS", Op::Txs, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 5),
    Instr::new("???", Op::Nop, AddrMode::Imp, 5),
    Instr::new("STA", Op::Sta, AddrMode::Abx, 5),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 5),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 5),
    // opcodes 0xa0..=0xaf
    Instr::new("LDY", Op::Ldy, AddrMode::Imm, 2),
    Instr::new("LDA", Op::Lda, AddrMode::Izx, 6),
    Instr::new("LDX", Op::Ldx, AddrMode::Imm, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 6),
    Instr::new("LDY", Op::Ldy, AddrMode::Zp0, 3),
    Instr::new("LDA", Op::Lda, AddrMode::Zp0, 3),
    Instr::new("LDX", Op::Ldx, AddrMode::Zp0, 3),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 3),
    Instr::new("TAY", Op::Tay, AddrMode::Imp, 2),
    Instr::new("LDA", Op::Lda, AddrMode::Imm, 2),
    Instr::new("TAX", Op::Tax, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("LDY", Op::Ldy, AddrMode::Abs, 4),
    Instr::new("LDA", Op::Lda, AddrMode::Abs, 4),
    Instr::new("LDX", Op::Ldx, AddrMode::Abs, 4),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 4),
    // opcodes 0xb0..=0xbf
    Instr::new("BCS", Op::Bcs, AddrMode::Rel, 2),
    Instr::new("LDA", Op::Lda, AddrMode::Izy, 5),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 5),
    Instr::new("LDY", Op::Ldy, AddrMode::Zpx, 4),
    Instr::new("LDA", Op::Lda, AddrMode::Zpx, 4),
    Instr::new("LDX", Op::Ldx, AddrMode::Zpy, 4),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 4),
    Instr::new("CLV", Op::Clv, AddrMode::Imp, 2),
    Instr::new("LDA", Op::Lda, AddrMode::Aby, 4),
    Instr::new("TSX", Op::Tsx, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 4),
    Instr::new("LDY", Op::Ldy, AddrMode::Abx, 4),
    Instr::new("LDA", Op::Lda, AddrMode::Abx, 4),
    Instr::new("LDX", Op::Ldx, AddrMode::Aby, 4),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 4),
    // opcodes 0xc0..=0xcf
    Instr::new("CPY", Op::Cpy, AddrMode::Imm, 2),
    Instr::new("CMP", Op::Cmp, AddrMode::Izx, 6),
    Instr::new("???", Op::Nop, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 8),
    Instr::new("CPY", Op::Cpy, AddrMode::Zp0, 3),
    Instr::new("CMP", Op::Cmp, AddrMode::Zp0, 3),
    Instr::new("DEC", Op::Dec, AddrMode::Zp0, 5),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 5),
    Instr::new("INY", Op::Iny, AddrMode::Imp, 2),
    Instr::new("CMP", Op::Cmp, AddrMode::Imm, 2),
    Instr::new("DEX", Op::Dex, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("CPY", Op::Cpy, AddrMode::Abs, 4),
    Instr::new("CMP", Op::Cmp, AddrMode::Abs, 4),
    Instr::new("DEC", Op::Dec, AddrMode::Abs, 6),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 6),
    // opcodes 0xd0..=0xdf
    Instr::new("BNE", Op::Bne, AddrMode::Rel, 2),
    Instr::new("CMP", Op::Cmp, AddrMode::Izy, 5),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 8),
    Instr::new("???", Op::Nop, AddrMode::Imp, 4),
    Instr::new("CMP", Op::Cmp, AddrMode::Zpx, 4),
    Instr::new("DEC", Op::Dec, AddrMode::Zpx, 6),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 6),
    Instr::new("CLD", Op::Cld, AddrMode::Imp, 2),
    Instr::new("CMP", Op::Cmp, AddrMode::Aby, 4),
    Instr::new("NOP", Op::Nop, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 7),
    Instr::new("???", Op::Nop, AddrMode::Imp, 4),
    Instr::new("CMP", Op::Cmp, AddrMode::Abx, 4),
    Instr::new("DEC", Op::Dec, AddrMode::Abx, 7),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 7),
    // opcodes 0xe0..=0xef
    Instr::new("CPX", Op::Cpx, AddrMode::Imm, 2),
    Instr::new("SBC", Op::Sbc, AddrMode::Izx, 6),
    Instr::new("???", Op::Nop, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 8),
    Instr::new("CPX", Op::Cpx, AddrMode::Zp0, 3),
    Instr::new("SBC", Op::Sbc, AddrMode::Zp0, 3),
    Instr::new("INC", Op::Inc, AddrMode::Zp0, 5),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 5),
    Instr::new("INX", Op::Inx, AddrMode::Imp, 2),
    Instr::new("SBC", Op::Sbc, AddrMode::Imm, 2),
    Instr::new("NOP", Op::Nop, AddrMode::Imp, 2),
    Instr::new("???", Op::Sbc, AddrMode::Imp, 2),
    Instr::new("CPX", Op::Cpx, AddrMode::Abs, 4),
    Instr::new("SBC", Op::Sbc, AddrMode::Abs, 4),
    Instr::new("INC", Op::Inc, AddrMode::Abs, 6),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 6),
    // opcodes 0xf0..=0xff
    Instr::new("BEQ", Op::Beq, AddrMode::Rel, 2),
    Instr::new("SBC", Op::Sbc, AddrMode::Izy, 5),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 8),
    Instr::new("???", Op::Nop, AddrMode::Imp, 4),
    Instr::new("SBC", Op::Sbc, AddrMode::Zpx, 4),
    Instr::new("INC", Op::Inc, AddrMode::Zpx, 6),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 6),
    Instr::new("SED", Op::Sed, AddrMode::Imp, 2),
    Instr::new("SBC", Op::Sbc, AddrMode::Aby, 4),
    Instr::new("NOP", Op::Nop, AddrMode::Imp, 2),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 7),
    Instr::new("???", Op::Nop, AddrMode::Imp, 4),
    Instr::new("SBC", Op::Sbc, AddrMode::Abx, 4),
    Instr::new("INC", Op::Inc, AddrMode::Abx, 7),
    Instr::new("???", Op::Xxx, AddrMode::Imp, 7),
];
