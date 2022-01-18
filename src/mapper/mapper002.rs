use super::{MapResult, Mapper};

pub struct Mapper002 {
    num_banks_prg: usize,
    num_banks_chr: usize,

    prg_bank_select_lo: usize,
    prg_bank_select_hi: usize,
}

impl Mapper002 {
    pub fn new(num_banks_prg: usize, num_banks_chr: usize) -> Mapper002 {
        Mapper002 {
            num_banks_prg,
            num_banks_chr,

            prg_bank_select_lo: 0,
            prg_bank_select_hi: num_banks_prg - 1,
        }
    }
}

impl Mapper for Mapper002 {
    fn cpu_map_read(&mut self, addr: u16) -> MapResult {
        match addr {
            0x8000..=0xbfff => MapResult::MapAddr(
                self.prg_bank_select_lo as usize * 0x4000 + (addr & 0x3fff) as usize,
            ),
            0xc000..=0xffff => MapResult::MapAddr(
                self.prg_bank_select_hi as usize * 0x4000 + (addr & 0x3fff) as usize,
            ),
            _ => MapResult::None,
        }
    }

    fn cpu_map_read_ro(&self, addr: u16) -> MapResult {
        match addr {
            0x8000..=0xbfff => MapResult::MapAddr(
                self.prg_bank_select_lo as usize * 0x4000 + (addr & 0x3fff) as usize,
            ),
            0xc000..=0xffff => MapResult::MapAddr(
                self.prg_bank_select_hi as usize * 0x4000 + (addr & 0x3fff) as usize,
            ),
            _ => MapResult::None,
        }
    }

    fn cpu_map_write(&mut self, addr: u16, data: u8) -> MapResult {
        match addr {
            0x8000..=0xffff => {
                self.prg_bank_select_lo = (data & 0x0f) as usize % self.num_banks_prg;
            }
            _ => {}
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

    fn reset(&mut self) {
        self.prg_bank_select_lo = 0;
        self.prg_bank_select_hi = self.num_banks_prg - 1;
    }
}
