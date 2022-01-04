pub mod mapper000;

use crate::cartridge::Mirror;

pub trait Mapper {
    fn cpu_map_read(&self, addr: u16) -> Option<usize>;

    fn cpu_map_write(&mut self, addr: u16) -> Option<usize>;

    fn ppu_map_read(&self, addr: u16) -> Option<usize>;

    fn ppu_map_write(&mut self, addr: u16) -> Option<usize>;

    fn mirror(&self) -> Mirror;

    fn reset(&mut self);
}
