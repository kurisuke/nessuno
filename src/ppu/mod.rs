pub struct Ppu {
    tbl_name: [[u8; 1024]; 2],
    tbl_palette: [u8; 32],
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            tbl_name: [[0; 1024]; 2],
            tbl_palette: [0; 32],
        }
    }

    pub fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000 => 0x00,
            0x0001 => 0x00,
            0x0002 => 0x00,
            0x0003 => 0x00,
            0x0004 => 0x00,
            0x0005 => 0x00,
            0x0006 => 0x00,
            0x0007 => 0x00,
            _ => unreachable!(),
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000 => {}
            0x0001 => {}
            0x0002 => {}
            0x0003 => {}
            0x0004 => {}
            0x0005 => {}
            0x0006 => {}
            0x0007 => {}
            _ => unreachable!(),
        }
    }
}
