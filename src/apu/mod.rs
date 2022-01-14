mod channel;
mod lookup;
mod pulse;

pub struct Apu {}

impl Apu {
    pub fn new() -> Apu {
        Apu {}
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {}

    pub fn cpu_read(&self, addr: u16) -> u8 {
        0x00
    }

    pub fn clock(&mut self) {}

    pub fn get_output_sample(&self) -> f32 {
        0.0f32
    }
}
