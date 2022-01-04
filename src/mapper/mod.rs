pub mod mapper000;
pub mod mapper001;
pub mod mapper002;

use crate::cartridge::Mirror;

pub enum MapResult {
    None,
    MapAddr(usize),
    DirectRead(u8),
    DirectWrite,
}

pub trait Mapper {
    fn cpu_map_read(&mut self, addr: u16) -> MapResult;

    fn cpu_map_read_ro(&self, addr: u16) -> MapResult;

    fn cpu_map_write(&mut self, addr: u16, data: u8) -> MapResult;

    fn ppu_map_read(&mut self, addr: u16) -> MapResult;

    fn ppu_map_write(&mut self, addr: u16, data: u8) -> MapResult;

    fn mirror(&self) -> Mirror;

    fn reset(&mut self);
}
