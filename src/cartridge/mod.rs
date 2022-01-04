use crate::mapper::{mapper000::Mapper000, Mapper};
use std::fs::File;
use std::io;
use std::io::Read;
use std::mem;

pub struct Cartridge {
    mem_prg: Vec<u8>,
    mem_chr: Vec<u8>,

    hw_mirror: Mirror,

    mapper_id: u8,
    num_banks_prg: u8,
    num_banks_chr: u8,

    mapper: Box<dyn Mapper>,
}

#[derive(Copy, Clone)]
pub enum Mirror {
    Hardware,
    Vertical,
    Horizontal,
    OneScreenLo,
    OneScreenHi,
}

struct CartridgeHeader {
    name: [u8; 4],
    prg_rom_chunks: u8,
    chr_rom_chunks: u8,
    mapper1: u8,
    mapper2: u8,
    prg_ram_size: u8,
    tv_system1: u8,
    tv_system2: u8,
    unused: [u8; 5],
}

impl CartridgeHeader {
    fn load(reader: &mut impl io::Read) -> Result<Self, io::Error> {
        let mut buf = [0; mem::size_of::<CartridgeHeader>()];
        reader.read_exact(&mut buf)?;

        let header = CartridgeHeader {
            name: [buf[0], buf[1], buf[2], buf[3]],
            prg_rom_chunks: buf[4],
            chr_rom_chunks: buf[5],
            mapper1: buf[6],
            mapper2: buf[7],
            prg_ram_size: buf[8],
            tv_system1: buf[9],
            tv_system2: buf[10],
            unused: [buf[11], buf[12], buf[13], buf[14], buf[15]],
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

        let mapper = match mapper_id {
            0 => Box::new(Mapper000::new(header.prg_rom_chunks, header.chr_rom_chunks)),
            _ => panic!("Unsupported mapper: {:03}", mapper_id),
        };

        let file_type = 1;
        match file_type {
            0 => {
                unreachable!()
            }
            1 => {
                let num_banks_prg = header.prg_rom_chunks;
                let mut mem_prg = vec![0; num_banks_prg as usize * 16384];
                reader.read_exact(&mut mem_prg)?;

                let num_banks_chr = header.chr_rom_chunks;
                let mut mem_chr = vec![0; num_banks_chr as usize * 8192];
                reader.read_exact(&mut mem_chr)?;

                Ok(Cartridge {
                    mem_prg,
                    mem_chr,
                    mapper_id,
                    hw_mirror,
                    num_banks_prg,
                    num_banks_chr,
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
        self.mapper
            .cpu_map_read(addr)
            .map(|mapped_addr| self.mem_prg[mapped_addr])
    }

    pub fn cpu_read_ro(&self, addr: u16) -> Option<u8> {
        self.mapper
            .cpu_map_read(addr)
            .map(|mapped_addr| self.mem_prg[mapped_addr])
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) -> bool {
        if let Some(mapped_addr) = self.mapper.cpu_map_write(addr) {
            self.mem_prg[mapped_addr] = data;
            true
        } else {
            false
        }
    }

    pub fn ppu_read(&mut self, addr: u16) -> Option<u8> {
        self.mapper
            .ppu_map_read(addr)
            .map(|mapped_addr| self.mem_chr[mapped_addr])
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) -> bool {
        if let Some(mapped_addr) = self.mapper.ppu_map_write(addr) {
            self.mem_chr[mapped_addr] = data;
            true
        } else {
            false
        }
    }

    pub fn mirror(&self) -> Mirror {
        let m = self.mapper.mirror();
        match m {
            Mirror::Hardware => self.hw_mirror,
            _ => m,
        }
    }

    pub fn reset(&mut self) {
        self.mapper.reset();
    }
}
