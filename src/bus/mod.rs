pub trait CpuBus {
    fn cpu_write(&mut self, addr: u16, data: u8);
    fn cpu_read(&mut self, addr: u16) -> u8;
}
