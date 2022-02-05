use super::{MapResult, Mapper};

use crate::cartridge::Mirror;

pub struct Mapper001 {
    num_banks_prg: usize,
    num_banks_chr: usize,

    prg_ram: [u8; 8 * 1024],

    mirror_mode: Mirror,
    control_reg: u8,
    load_reg: u8,
    prg_bank_select_16_lo: usize,
    prg_bank_select_16_hi: usize,
    prg_bank_select_32: usize,
    chr_bank_select_4_lo: usize,
    chr_bank_select_4_hi: usize,
    chr_bank_select_8: usize,
}

impl Mapper001 {
    pub fn new(num_banks_prg: usize, num_banks_chr: usize) -> Mapper001 {
        Mapper001 {
            num_banks_prg,
            num_banks_chr,

            prg_ram: [0; 8 * 1024],

            mirror_mode: Mirror::Horizontal,
            control_reg: 0x1c,
            load_reg: 0x10,
            prg_bank_select_16_lo: 0,
            prg_bank_select_16_hi: num_banks_prg - 1,
            prg_bank_select_32: 0,
            chr_bank_select_4_lo: 0,
            chr_bank_select_4_hi: 0,
            chr_bank_select_8: 0,
        }
    }
}

impl Mapper for Mapper001 {
    fn cpu_map_read(&mut self, addr: u16) -> MapResult {
        self.cpu_map_read_ro(addr)
    }

    fn cpu_map_read_ro(&self, addr: u16) -> MapResult {
        match addr {
            0x6000..=0x7fff => MapResult::DirectRead(self.prg_ram[(addr & 0x1fff) as usize]),
            0x8000..=0xffff => {
                if self.control_reg & 0x08 != 0 {
                    // 16K mode
                    match addr {
                        0x8000..=0xbfff => MapResult::MapAddr(
                            self.prg_bank_select_16_lo * 0x4000 + (addr & 0x3fff) as usize,
                        ),
                        0xc000..=0xffff => MapResult::MapAddr(
                            self.prg_bank_select_16_hi * 0x4000 + (addr & 0x3fff) as usize,
                        ),
                        _ => unreachable!(),
                    }
                } else {
                    MapResult::MapAddr(self.prg_bank_select_32 * 0x8000 + (addr & 0x7fff) as usize)
                }
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
            0x8000..=0xffff => {
                if data & 0x80 != 0 {
                    // reset serial loading
                    self.load_reg = 0x10;
                    self.control_reg |= 0x0c;
                } else if self.load_reg & 0x01 != 0 {
                    // register is full
                    self.load_reg >>= 1;
                    self.load_reg |= (data & 0x01) << 4;

                    let target = (addr >> 13) & 0x03;
                    match target {
                        0 => {
                            // set Control Register
                            self.control_reg = self.load_reg;
                            self.mirror_mode = match self.control_reg & 0x03 {
                                0 => Mirror::OneScreenLo,
                                1 => Mirror::OneScreenHi,
                                2 => Mirror::Vertical,
                                3 => Mirror::Horizontal,
                                _ => unreachable!(),
                            };
                        }
                        1 => {
                            // set CHR Bank Lo
                            if self.num_banks_chr > 0 {
                                if self.control_reg & 0x10 != 0 {
                                    self.chr_bank_select_4_lo =
                                        self.load_reg as usize % (self.num_banks_chr << 1);
                                } else {
                                    self.chr_bank_select_8 =
                                        (self.load_reg >> 1) as usize % self.num_banks_chr;
                                }
                            }
                        }
                        2 => {
                            // set CHR Bank Hi
                            if self.num_banks_chr > 0 {
                                if self.control_reg & 0x10 != 0 {
                                    self.chr_bank_select_4_hi =
                                        self.load_reg as usize % (self.num_banks_chr << 1);
                                } else {
                                    // do nothing
                                }
                            }
                        }
                        3 => {
                            // configure PRG banks
                            match (self.control_reg >> 2) & 0x03 {
                                0 | 1 => {
                                    self.prg_bank_select_32 = ((self.load_reg & 0x0e) >> 1)
                                        as usize
                                        % (self.num_banks_prg >> 1);
                                }
                                2 => {
                                    self.prg_bank_select_16_lo = 0;
                                    self.prg_bank_select_16_hi =
                                        (self.load_reg & 0x0f) as usize % self.num_banks_prg;
                                }
                                3 => {
                                    self.prg_bank_select_16_lo =
                                        (self.load_reg & 0x0f) as usize % self.num_banks_prg;
                                    self.prg_bank_select_16_hi = self.num_banks_prg - 1;
                                }
                                _ => unreachable!(),
                            }
                        }
                        _ => unreachable!(),
                    }
                    // reset load register
                    self.load_reg = 0x10;
                } else {
                    self.load_reg >>= 1;
                    self.load_reg |= (data & 0x01) << 4;
                }
                MapResult::DirectWrite
            }
            _ => MapResult::None,
        }
    }

    fn ppu_map_read(&mut self, addr: u16) -> MapResult {
        match addr {
            0x0000..=0x1fff => {
                if self.num_banks_chr == 0 {
                    MapResult::MapAddr(addr as usize)
                } else if self.control_reg & 0x10 != 0 {
                    // 4K mode
                    match addr {
                        0x0000..=0x0fff => {
                            MapResult::MapAddr(self.chr_bank_select_4_lo * 0x1000 + addr as usize)
                        }
                        0x1000..=0x1fff => MapResult::MapAddr(
                            self.chr_bank_select_4_hi * 0x1000 + (addr & 0x0fff) as usize,
                        ),
                        _ => unreachable!(),
                    }
                } else {
                    // 8K mode
                    MapResult::MapAddr(self.chr_bank_select_8 * 0x2000 + addr as usize)
                }
            }
            _ => MapResult::None,
        }
    }

    fn ppu_map_write(&mut self, addr: u16, _data: u8) -> MapResult {
        if addr < 0x2000 && self.num_banks_chr == 0 {
            // treat as RAM
            MapResult::MapAddr(addr as usize)
        } else {
            MapResult::None
        }
    }

    fn mirror(&self) -> Mirror {
        self.mirror_mode
    }

    fn reset(&mut self) {
        self.prg_ram = [0; 8 * 1024];

        self.mirror_mode = Mirror::Horizontal;
        self.control_reg = 0x1c;
        self.load_reg = 0x10;
        self.prg_bank_select_16_lo = 0;
        self.prg_bank_select_16_hi = self.num_banks_prg - 1;
        self.prg_bank_select_32 = 0;
        self.chr_bank_select_4_lo = 0;
        self.chr_bank_select_4_hi = 0;
        self.chr_bank_select_8 = 0;
    }

    fn load_ram(&mut self, ram: &[u8]) {
        if self.prg_ram.len() == ram.len() {
            self.prg_ram.copy_from_slice(ram);
        }
    }

    fn save_ram(&self) -> Option<Vec<u8>> {
        Some(self.prg_ram.to_vec())
    }
}
