pub mod mapper000;

pub trait Mapper {
    fn cpu_map_read(&mut self, addr: u16) -> Option<usize>;

    fn cpu_map_write(&mut self, addr: u16) -> Option<usize>;

    fn ppu_map_read(&mut self, addr: u16) -> Option<usize>;

    fn ppu_map_write(&mut self, addr: u16) -> Option<usize>;
}
