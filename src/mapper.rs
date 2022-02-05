pub mod mapper000;
pub mod mapper001;
pub mod mapper002;
pub mod mapper003;
pub mod mapper004;
pub mod mapper007;
pub mod mapper009;

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

    fn mirror(&self) -> Mirror {
        Mirror::Hardware
    }

    fn reset(&mut self) {}

    // Scanline IRQ interface
    fn irq_state(&self) -> bool {
        false
    }

    fn irq_clear(&mut self) {}

    fn on_scanline_end(&mut self) {}

    fn load_ram(&mut self, _ram: &[u8]) {}

    fn save_ram(&self) -> Option<Vec<u8>> {
        None
    }
}
