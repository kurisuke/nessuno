use super::{MapResult, Mapper};
use crate::cartridge::Mirror;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Mapper007 {
    num_banks_prg: usize,
    num_banks_chr: usize,
    prg_bank_select: usize,
    mirror_mode: Mirror,
}

impl Mapper007 {
    pub fn new(num_banks_prg: usize, num_banks_chr: usize) -> Mapper007 {
        Mapper007 {
            num_banks_prg,
            num_banks_chr,
            prg_bank_select: 0,
            mirror_mode: Mirror::OneScreenLo,
        }
    }
}

#[typetag::serde]
impl Mapper for Mapper007 {
    fn cpu_map_read(&mut self, addr: u16) -> MapResult {
        self.cpu_map_read_ro(addr)
    }

    fn cpu_map_read_ro(&self, addr: u16) -> MapResult {
        match addr {
            0x8000..=0xffff => {
                MapResult::MapAddr(self.prg_bank_select * 0x8000 + (addr & 0x7fff) as usize)
            }
            _ => MapResult::None,
        }
    }

    fn cpu_map_write(&mut self, addr: u16, data: u8) -> MapResult {
        if let 0x8000..=0xffff = addr {
            self.prg_bank_select = (data & 0x07) as usize % (self.num_banks_prg >> 1);
            self.mirror_mode = if (data & 0x10) == 0 {
                Mirror::OneScreenLo
            } else {
                Mirror::OneScreenHi
            };
        }
        MapResult::None
    }

    fn ppu_map_read(&mut self, addr: u16) -> MapResult {
        match addr {
            0x0000..=0x1fff => MapResult::MapAddr(addr as usize),
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
        self.prg_bank_select = 0;
        self.mirror_mode = Mirror::OneScreenLo;
    }
}
