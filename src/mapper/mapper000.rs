use super::{MapResult, Mapper};

use crate::cartridge::Mirror;

pub struct Mapper000 {
    num_banks_prg: usize,
    num_banks_chr: usize,
}

impl Mapper000 {
    pub fn new(num_banks_prg: usize, num_banks_chr: usize) -> Mapper000 {
        Mapper000 {
            num_banks_prg,
            num_banks_chr,
        }
    }
}

impl Mapper for Mapper000 {
    fn cpu_map_read(&mut self, addr: u16) -> MapResult {
        match addr {
            0x8000..=0xffff => {
                if self.num_banks_prg > 1 {
                    MapResult::MapAddr((addr & 0x7fff) as usize)
                } else {
                    MapResult::MapAddr((addr & 0x3fff) as usize)
                }
            }
            _ => MapResult::None,
        }
    }

    fn cpu_map_read_ro(&self, addr: u16) -> MapResult {
        match addr {
            0x8000..=0xffff => {
                if self.num_banks_prg > 1 {
                    MapResult::MapAddr((addr & 0x7fff) as usize)
                } else {
                    MapResult::MapAddr((addr & 0x3fff) as usize)
                }
            }
            _ => MapResult::None,
        }
    }

    fn cpu_map_write(&mut self, addr: u16, data: u8) -> MapResult {
        match addr {
            0x8000..=0xffff => {
                if self.num_banks_prg > 1 {
                    MapResult::MapAddr((addr & 0x7fff) as usize)
                } else {
                    MapResult::MapAddr((addr & 0x3fff) as usize)
                }
            }
            _ => MapResult::None,
        }
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
        Mirror::Hardware
    }

    fn reset(&mut self) {}
}
