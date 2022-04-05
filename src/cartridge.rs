use crate::mapper::{
    mapper000::Mapper000, mapper001::Mapper001, mapper002::Mapper002, mapper003::Mapper003,
    mapper004::Mapper004, mapper007::Mapper007, mapper009::Mapper009, MapResult, Mapper,
};
use std::fs::File;
use std::io;
use std::io::Read;
use std::mem;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Cartridge {
    mem_prg: Vec<u8>,
    mem_chr: Vec<u8>,

    hw_mirror: Mirror,
    mapper: Box<dyn Mapper>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Mirror {
    Hardware,
    Vertical,
    Horizontal,
    OneScreenLo,
    OneScreenHi,
}

struct CartridgeHeader {
    _name: [u8; 4],
    prg_rom_chunks: u8,
    chr_rom_chunks: u8,
    mapper1: u8,
    mapper2: u8,
    _prg_ram_size: u8,
    _tv_system1: u8,
    _tv_system2: u8,
    _unused: [u8; 5],
}

impl CartridgeHeader {
    fn load(reader: &mut impl io::Read) -> Result<Self, io::Error> {
        let mut buf = [0; mem::size_of::<CartridgeHeader>()];
        reader.read_exact(&mut buf)?;

        let header = CartridgeHeader {
            _name: [buf[0], buf[1], buf[2], buf[3]],
            prg_rom_chunks: buf[4],
            chr_rom_chunks: buf[5],
            mapper1: buf[6],
            mapper2: buf[7],
            _prg_ram_size: buf[8],
            _tv_system1: buf[9],
            _tv_system2: buf[10],
            _unused: [buf[11], buf[12], buf[13], buf[14], buf[15]],
        };

        Ok(header)
    }
}

impl Cartridge {
    pub fn new(filename: &str) -> Result<Cartridge, io::Error> {
        let f = File::open(filename)?;
        let mut reader = io::BufReader::new(f);
        let header = CartridgeHeader::load(&mut reader)?;

        if header.mapper1 & 0x04 != 0 {
            let _junk = reader.seek_relative(512)?;
        }
        let mapper_id = ((header.mapper2 >> 4) << 4) | (header.mapper1 >> 4);
        let hw_mirror = if header.mapper1 & 0x01 != 0 {
            Mirror::Vertical
        } else {
            Mirror::Horizontal
        };

        let num_banks_prg = header.prg_rom_chunks as usize;
        let num_banks_chr = header.chr_rom_chunks as usize;

        let mapper: Box<dyn Mapper> = match mapper_id {
            0 => Box::new(Mapper000::new(num_banks_prg, num_banks_chr)),
            1 => Box::new(Mapper001::new(num_banks_prg, num_banks_chr)),
            2 => Box::new(Mapper002::new(num_banks_prg, num_banks_chr)),
            3 => Box::new(Mapper003::new(num_banks_prg, num_banks_chr)),
            4 => Box::new(Mapper004::new(num_banks_prg, num_banks_chr)),
            7 => Box::new(Mapper007::new(num_banks_prg, num_banks_chr)),
            9 => Box::new(Mapper009::new(num_banks_prg, num_banks_chr)),
            _ => panic!("Unsupported mapper: {:03}", mapper_id),
        };
        println!(
            "Mapper: {:03}, #prg: {}, #chr: {}",
            mapper_id, num_banks_prg, num_banks_chr
        );

        let file_type = 1;
        match file_type {
            0 => {
                unreachable!()
            }
            1 => {
                let mut mem_prg = vec![0; num_banks_prg as usize * 0x4000];
                reader.read_exact(&mut mem_prg)?;

                let mem_chr = match num_banks_chr {
                    0 => vec![0; 8192],
                    _ => {
                        let mut m = vec![0; num_banks_chr as usize * 0x2000];
                        reader.read_exact(&mut m)?;
                        m
                    }
                };

                Ok(Cartridge {
                    mem_prg,
                    mem_chr,
                    hw_mirror,
                    mapper,
                })
            }
            2 => {
                unreachable!()
            }
            _ => {
                unreachable!()
            }
        }
    }

    pub fn cpu_read(&mut self, addr: u16) -> Option<u8> {
        match self.mapper.cpu_map_read(addr) {
            MapResult::MapAddr(mapped_addr) => Some(self.mem_prg[mapped_addr]),
            MapResult::DirectRead(v) => Some(v),
            _ => None,
        }
    }

    pub fn cpu_read_ro(&self, addr: u16) -> Option<u8> {
        match self.mapper.cpu_map_read_ro(addr) {
            MapResult::MapAddr(mapped_addr) => Some(self.mem_prg[mapped_addr]),
            MapResult::DirectRead(v) => Some(v),
            _ => None,
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) -> bool {
        match self.mapper.cpu_map_write(addr, data) {
            MapResult::MapAddr(mapped_addr) => {
                self.mem_prg[mapped_addr] = data;
                true
            }
            MapResult::DirectWrite => true,
            _ => false,
        }
    }

    pub fn ppu_read(&mut self, addr: u16) -> Option<u8> {
        match self.mapper.ppu_map_read(addr) {
            MapResult::MapAddr(mapped_addr) => Some(self.mem_chr[mapped_addr]),
            MapResult::DirectRead(v) => Some(v),
            _ => None,
        }
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) -> bool {
        match self.mapper.ppu_map_write(addr, data) {
            MapResult::MapAddr(mapped_addr) => {
                self.mem_chr[mapped_addr] = data;
                true
            }
            MapResult::DirectWrite => true,
            _ => false,
        }
    }

    pub fn mirror(&self) -> Mirror {
        let m = self.mapper.mirror();
        match m {
            Mirror::Hardware => self.hw_mirror,
            _ => m,
        }
    }

    pub fn on_scanline_end(&mut self) {
        self.mapper.on_scanline_end();
    }

    pub fn irq_state(&self) -> bool {
        self.mapper.irq_state()
    }

    pub fn irq_clear(&mut self) {
        self.mapper.irq_clear()
    }

    pub fn reset(&mut self) {
        self.mapper.reset();
    }
}
