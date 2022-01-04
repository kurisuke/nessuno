use super::{MapResult, Mapper};

use crate::cartridge::Mirror;

pub struct Mapper004 {
    num_banks_prg: usize,

    prg_ram: [u8; 8 * 1024],
    mirror_mode: Mirror,

    bank_reg: [u8; 8],
    prg_bank_offset: [usize; 4],
    chr_bank_offset: [usize; 8],
    target_reg_idx: usize,
    prg_bank_mode: bool,
    chr_inversion: bool,
}

impl Mapper004 {
    pub fn new(num_banks_prg: usize, _num_banks_chr: usize) -> Mapper004 {
        Mapper004 {
            num_banks_prg,

            prg_ram: [0; 8 * 1024],
            mirror_mode: Mirror::Horizontal,

            bank_reg: [0; 8],
            prg_bank_offset: [
                0,
                0x2000,
                (num_banks_prg * 2 - 2) * 0x2000,
                (num_banks_prg * 2 - 1) * 0x2000,
            ],
            chr_bank_offset: [0; 8],
            target_reg_idx: 0,
            prg_bank_mode: false,
            chr_inversion: false,
        }
    }
}

impl Mapper for Mapper004 {
    fn cpu_map_read(&mut self, addr: u16) -> MapResult {
        self.cpu_map_read_ro(addr)
    }

    fn cpu_map_read_ro(&self, addr: u16) -> MapResult {
        match addr {
            0x6000..=0x7fff => MapResult::DirectRead(self.prg_ram[(addr & 0x1fff) as usize]),
            0x8000..=0xffff => {
                let idx = ((addr - 0x8000) >> 13) as usize;
                MapResult::MapAddr(self.prg_bank_offset[idx] + (addr & 0x1fff) as usize)
            }
            _ => MapResult::None,
        }
    }

    fn cpu_map_write(&mut self, addr: u16, data: u8) -> MapResult {
        match addr {
            0x6000..=0x7fff => {
                self.prg_ram[(addr & 0x1fff) as usize] = data;
                MapResult::DirectWrite
            }
            0x8000..=0x9fff => {
                // Bank Select
                if addr & 0x0001 == 0 {
                    // configure target / mode
                    self.target_reg_idx = (data & 0x07) as usize;
                    self.prg_bank_mode = data & 0x40 != 0;
                    self.chr_inversion = data & 0x80 != 0;
                } else {
                    // update mapping
                    self.bank_reg[self.target_reg_idx] = data;

                    if self.chr_inversion {
                        self.chr_bank_offset[0] = self.bank_reg[2] as usize * 0x0400;
                        self.chr_bank_offset[1] = self.bank_reg[3] as usize * 0x0400;
                        self.chr_bank_offset[2] = self.bank_reg[4] as usize * 0x0400;
                        self.chr_bank_offset[3] = self.bank_reg[5] as usize * 0x0400;
                        self.chr_bank_offset[4] = (self.bank_reg[0] & 0xfe) as usize * 0x0400;
                        self.chr_bank_offset[5] = ((self.bank_reg[0] & 0xfe) + 1) as usize * 0x0400;
                        self.chr_bank_offset[6] = (self.bank_reg[1] & 0xfe) as usize * 0x0400;
                        self.chr_bank_offset[7] = ((self.bank_reg[1] & 0xfe) + 1) as usize * 0x0400;
                    } else {
                        self.chr_bank_offset[0] = (self.bank_reg[0] & 0xfe) as usize * 0x0400;
                        self.chr_bank_offset[1] = ((self.bank_reg[0] & 0xfe) + 1) as usize * 0x0400;
                        self.chr_bank_offset[2] = (self.bank_reg[1] & 0xfe) as usize * 0x0400;
                        self.chr_bank_offset[3] = ((self.bank_reg[1] & 0xfe) + 1) as usize * 0x0400;
                        self.chr_bank_offset[4] = self.bank_reg[2] as usize * 0x0400;
                        self.chr_bank_offset[5] = self.bank_reg[3] as usize * 0x0400;
                        self.chr_bank_offset[6] = self.bank_reg[4] as usize * 0x0400;
                        self.chr_bank_offset[7] = self.bank_reg[5] as usize * 0x0400;
                    }

                    if self.prg_bank_mode {
                        self.prg_bank_offset[2] = (self.bank_reg[6] & 0x3f) as usize * 0x2000;
                        self.prg_bank_offset[0] = (self.num_banks_prg * 2 - 2) * 0x2000;
                    } else {
                        self.prg_bank_offset[0] = (self.bank_reg[6] & 0x3f) as usize * 0x2000;
                        self.prg_bank_offset[2] = (self.num_banks_prg * 2 - 2) * 0x2000;
                    }

                    self.prg_bank_offset[1] = (self.bank_reg[7] & 0x3f) as usize * 0x2000;
                    self.prg_bank_offset[3] = (self.num_banks_prg * 2 - 1) * 0x2000;
                }
                MapResult::DirectWrite
            }
            0xa000..=0xbfff => {
                if addr & 0x0001 == 0 {
                    // Mirroring
                    self.mirror_mode = if data & 0x01 != 0 {
                        Mirror::Horizontal
                    } else {
                        Mirror::Vertical
                    };
                } else {
                    // PRG Ram Protect, TODO
                }
                MapResult::DirectWrite
            }
            0xc000..=0xdfff => {
                // IRQ Latch / Configure, TODO
                MapResult::DirectWrite
            }
            0xe000..=0xffff => {
                // IRQ Enable / Disable, TODO
                MapResult::DirectWrite
            }
            _ => MapResult::None,
        }
    }

    fn ppu_map_read(&mut self, addr: u16) -> MapResult {
        match addr {
            0x0000..=0x1fff => {
                let idx = (addr >> 10) as usize;
                MapResult::MapAddr(self.chr_bank_offset[idx] + (addr & 0x03ff) as usize)
            }
            _ => MapResult::None,
        }
    }

    fn ppu_map_write(&mut self, _addr: u16, _data: u8) -> MapResult {
        MapResult::None
    }

    fn mirror(&self) -> Mirror {
        self.mirror_mode
    }

    fn reset(&mut self) {
        self.prg_ram = [0; 8 * 1024];
        self.mirror_mode = Mirror::Horizontal;
        self.prg_bank_mode = false;
        self.chr_inversion = false;

        self.bank_reg = [0; 8];
        self.prg_bank_offset = [
            0,
            0x2000,
            (self.num_banks_prg * 2 - 2) * 0x2000,
            (self.num_banks_prg * 2 - 1) * 0x2000,
        ];
        self.chr_bank_offset = [0; 8];
        self.target_reg_idx = 0;
    }
}
