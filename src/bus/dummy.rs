pub struct DummyBus {
    ram: [u8; 64 * 1024],
}

impl DummyBus {
    pub fn new() -> DummyBus {
        DummyBus {
            ram: [0; 64 * 1024],
        }
    }
}

impl super::Bus for DummyBus {
    fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }
    fn read(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }
}
