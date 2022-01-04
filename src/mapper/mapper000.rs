use super::Mapper;

use crate::cartridge::Mirror;

pub struct Mapper000 {
    num_banks_prg: u8,
    num_banks_chr: u8,
}

impl Mapper000 {
    pub fn new(num_banks_prg: u8, num_banks_chr: u8) -> Mapper000 {
        Mapper000 {
            num_banks_prg,
            num_banks_chr,
        }
    }
}

impl Mapper for Mapper000 {
    fn cpu_map_read(&self, addr: u16) -> Option<usize> {
        match addr {
            0x8000..=0xffff => {
                if self.num_banks_prg > 1 {
                    Some((addr & 0x7fff) as usize)
                } else {
                    Some((addr & 0x3fff) as usize)
                }
            }
            _ => None,
        }
    }

    fn cpu_map_write(&mut self, addr: u16) -> Option<usize> {
        match addr {
            0x8000..=0xffff => {
                if self.num_banks_prg > 1 {
                    Some((addr & 0x7fff) as usize)
                } else {
                    Some((addr & 0x3fff) as usize)
                }
            }
            _ => None,
        }
    }

    fn ppu_map_read(&self, addr: u16) -> Option<usize> {
        match addr {
            0x0000..=0x1fff => Some(addr as usize),
            _ => None,
        }
    }

    fn ppu_map_write(&mut self, addr: u16) -> Option<usize> {
        match addr {
            0x0000..=0x1fff => {
                if self.num_banks_chr == 0 {
                    // treat as RAM
                    Some(addr as usize)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn mirror(&self) -> Mirror {
        Mirror::Hardware
    }

    fn reset(&mut self) {}
}
