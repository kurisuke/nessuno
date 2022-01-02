mod palette;

use palette::PALETTE_2C02;
use std::num::Wrapping;

pub struct Ppu {
    render_params: PpuRenderParams,

    tbl_name: [[u8; 1024]; 2],
    tbl_palette: [u8; 32],

    pub frame_complete: bool,

    scanline: usize,
    cycle: usize,

    prng: Prng,
}

pub struct PpuRenderParams {
    pub offset_x: usize,
    pub offset_y: usize,
    pub width_y: usize,
    pub scaling_factor: usize,
    pub bytes_per_pixel: usize,
}

impl Ppu {
    pub fn new(render_params: PpuRenderParams) -> Ppu {
        Ppu {
            render_params,

            tbl_name: [[0; 1024]; 2],
            tbl_palette: [0; 32],

            frame_complete: false,

            cycle: 0,
            scanline: 0,

            prng: Prng::new(),
        }
    }

    pub fn clock(&mut self, frame: &mut [u8]) {
        if let Some(pos) = visible(self.scanline, self.cycle) {
            let color = if self.prng.next().unwrap() & 0x1 > 0 {
                0x3f
            } else {
                0x30
            };
            self.set_pixel(frame, pos, color);
        }

        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline == 261 {
                self.frame_complete = true;
            } else if self.scanline > 261 {
                self.scanline = 0;
            }
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

    fn set_pixel(&self, frame: &mut [u8], pos: (usize, usize), color_idx: usize) {
        match self.render_params.scaling_factor {
            2 => {
                let py = self.render_params.offset_y + pos.0 * 2;
                let px = self.render_params.offset_x + pos.1 * 2;
                let off0 =
                    (py * self.render_params.width_y + px) * self.render_params.bytes_per_pixel;
                let off1 =
                    (py * self.render_params.width_y + px + 1) * self.render_params.bytes_per_pixel;
                let off2 =
                    (py + 1 * self.render_params.width_y + px) * self.render_params.bytes_per_pixel;
                let off3 = (py + 1 * self.render_params.width_y + px + 1)
                    * self.render_params.bytes_per_pixel;

                let color = &PALETTE_2C02[color_idx];
                frame[off0..off0 + self.render_params.bytes_per_pixel].copy_from_slice(color);
                frame[off1..off1 + self.render_params.bytes_per_pixel].copy_from_slice(color);
                frame[off2..off2 + self.render_params.bytes_per_pixel].copy_from_slice(color);
                frame[off3..off3 + self.render_params.bytes_per_pixel].copy_from_slice(color);
            }
            _ => unreachable!(),
        }
    }
}

fn visible(scanline: usize, cycle: usize) -> Option<(usize, usize)> {
    if scanline < 240 && cycle >= 1 && cycle <= 256 {
        Some((scanline, cycle - 1))
    } else {
        None
    }
}

struct Prng {
    seed: u32,
}

impl Prng {
    fn new() -> Self {
        Self { seed: 0 }
    }
}

impl Iterator for Prng {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.seed = (Wrapping(1103515245) * Wrapping(self.seed) + Wrapping(12345)).0 % 2147483648;
        Some(self.seed)
    }
}
