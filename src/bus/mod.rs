pub mod dummy;

pub trait Bus {
    fn write(&mut self, addr: u16, data: u8);
    fn read(&self, addr: u16) -> u8;
}
