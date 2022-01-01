pub struct DummyBus {
    ram: [u8; 64 * 1024],
}

impl DummyBus {
    pub fn new() -> DummyBus {
        DummyBus {
            ram: [0; 64 * 1024],
        }
    }

    pub fn load_from_str(&mut self, s: &str, addr: u16) {
        for (offset, n) in s.split_ascii_whitespace().enumerate() {
            self.ram[addr as usize + offset] = u8::from_str_radix(n, 16).unwrap();
        }
    }

    pub fn set_reset_vector(&mut self, addr: u16) {
        self.ram[0xfffc] = (addr & 0x00ff) as u8;
        self.ram[0xfffd] = ((addr >> 8) & 0x00ff) as u8;
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
