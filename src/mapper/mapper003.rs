use super::{MapResult, Mapper};

pub struct Mapper003 {
    num_banks_prg: usize,
    num_banks_chr: usize,
    chr_bank_select: usize,
}

impl Mapper003 {
    pub fn new(num_banks_prg: usize, num_banks_chr: usize) -> Mapper003 {
        Mapper003 {
            num_banks_prg,
            num_banks_chr,
            chr_bank_select: 0,
        }
    }
}

impl Mapper for Mapper003 {
    fn cpu_map_read(&mut self, addr: u16) -> MapResult {
        self.cpu_map_read_ro(addr)
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
        if let 0x8000..=0xffff = addr {
            self.chr_bank_select = (data & 0x03) as usize % self.num_banks_chr;
        }
        MapResult::None
    }

    fn ppu_map_read(&mut self, addr: u16) -> MapResult {
        match addr {
            0x0000..=0x1fff => MapResult::MapAddr(self.chr_bank_select * 0x2000 + addr as usize),
            _ => MapResult::None,
        }
    }

    fn ppu_map_write(&mut self, _addr: u16, _data: u8) -> MapResult {
        MapResult::None
    }

    fn reset(&mut self) {
        self.chr_bank_select = 0;
    }
}
