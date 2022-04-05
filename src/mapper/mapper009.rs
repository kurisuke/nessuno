use super::{MapResult, Mapper};

use crate::cartridge::Mirror;

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use typetag::serde;

#[derive(Deserialize, Serialize)]
pub struct Mapper009 {
    num_banks_prg_8k: usize,

    #[serde(with = "BigArray")]
    prg_ram: [u8; 8 * 1024],
    prg_bank_select_8k: usize,
    chr_bank_select_4k_lo_fd: usize,
    chr_bank_select_4k_lo_fe: usize,
    chr_bank_select_4k_hi_fd: usize,
    chr_bank_select_4k_hi_fe: usize,
    latch_lo: bool,
    latch_hi: bool,
    mirror_mode: Mirror,
}

impl Mapper009 {
    pub fn new(num_banks_prg: usize, _num_banks_chr: usize) -> Mapper009 {
        Mapper009 {
            num_banks_prg_8k: num_banks_prg * 2,

            prg_ram: [0; 8 * 1024],
            prg_bank_select_8k: 0,
            chr_bank_select_4k_lo_fd: 0,
            chr_bank_select_4k_lo_fe: 0,
            chr_bank_select_4k_hi_fd: 0,
            chr_bank_select_4k_hi_fe: 0,
            latch_lo: false,
            latch_hi: false,
            mirror_mode: Mirror::Vertical,
        }
    }
}

#[typetag::serde]
impl Mapper for Mapper009 {
    fn cpu_map_read(&mut self, addr: u16) -> MapResult {
        self.cpu_map_read_ro(addr)
    }

    fn cpu_map_read_ro(&self, addr: u16) -> MapResult {
        match addr {
            0x6000..=0x7fff => MapResult::DirectRead(self.prg_ram[(addr & 0x1fff) as usize]),
            0x8000..=0x9fff => {
                // switchable bank
                MapResult::MapAddr(self.prg_bank_select_8k * 0x2000 + (addr & 0x1fff) as usize)
            }
            0xa000..=0xffff => {
                // fixed to last 3 banks
                MapResult::MapAddr((self.num_banks_prg_8k - 3) * 0x2000 + (addr - 0xa000) as usize)
            }
            _ => MapResult::None,
        }
    }

    fn cpu_map_write(&mut self, addr: u16, data: u8) -> MapResult {
        match addr {
            0xa000..=0xafff => {
                self.prg_bank_select_8k = (data & 0x0f) as usize;
            }
            0xb000..=0xbfff => {
                self.chr_bank_select_4k_lo_fd = (data & 0x1f) as usize;
            }
            0xc000..=0xcfff => {
                self.chr_bank_select_4k_lo_fe = (data & 0x1f) as usize;
            }
            0xd000..=0xdfff => {
                self.chr_bank_select_4k_hi_fd = (data & 0x1f) as usize;
            }
            0xe000..=0xefff => {
                self.chr_bank_select_4k_hi_fe = (data & 0x1f) as usize;
            }
            0xf000..=0xffff => {
                if data & 0x01 != 0 {
                    self.mirror_mode = Mirror::Horizontal;
                } else {
                    self.mirror_mode = Mirror::Vertical;
                }
            }
            _ => {}
        }
        MapResult::None
    }

    fn ppu_map_read(&mut self, addr: u16) -> MapResult {
        // set latches
        match addr {
            0x0fd0..=0x0fdf => {
                self.latch_lo = false;
            }
            0x0fe0..=0x0fef => {
                self.latch_lo = true;
            }
            0x1fd0..=0x1fdf => {
                self.latch_hi = false;
            }
            0x1fe0..=0x1fef => {
                self.latch_hi = true;
            }
            _ => {}
        }

        match addr {
            0x0000..=0x0fff => {
                if self.latch_lo {
                    MapResult::MapAddr(
                        self.chr_bank_select_4k_lo_fe * 0x1000 + (addr & 0x0fff) as usize,
                    )
                } else {
                    MapResult::MapAddr(
                        self.chr_bank_select_4k_lo_fd * 0x1000 + (addr & 0x0fff) as usize,
                    )
                }
            }
            0x1000..=0x1fff => {
                if self.latch_hi {
                    MapResult::MapAddr(
                        self.chr_bank_select_4k_hi_fe * 0x1000 + (addr & 0x0fff) as usize,
                    )
                } else {
                    MapResult::MapAddr(
                        self.chr_bank_select_4k_hi_fd * 0x1000 + (addr & 0x0fff) as usize,
                    )
                }
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

    fn load_ram(&mut self, ram: &[u8]) {
        if self.prg_ram.len() == ram.len() {
            self.prg_ram.copy_from_slice(ram);
        }
    }

    fn save_ram(&self) -> Option<Vec<u8>> {
        Some(self.prg_ram.to_vec())
    }
}
