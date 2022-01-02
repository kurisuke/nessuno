mod palette;

use crate::cartridge::{self, Cartridge};
use palette::PALETTE_2C02;
use tiny_rng::{Rand, Rng};

pub struct Ppu {
    render_params: PpuRenderParams,

    tbl_pattern: [[u8; 4 * 1024]; 2],
    tbl_name: [[u8; 1024]; 2],
    tbl_palette: [u8; 32],

    pub frame_complete: bool,

    scanline: usize,
    cycle: usize,

    rng: Rng,
}

pub type PatternTable = [[u8; 4 * 128]; 128];
pub type PixelRgba = [u8; 4];

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

            tbl_pattern: [[0; 4 * 1024]; 2],
            tbl_name: [[0; 1024]; 2],
            tbl_palette: [0; 32],

            frame_complete: false,

            cycle: 0,
            scanline: 0,

            rng: Rng::from_seed(123456789),
        }
    }

    pub fn clock(&mut self, frame: &mut [u8]) {
        if let Some(pos) = visible(self.scanline, self.cycle) {
            let color = self.rng.rand_usize() & 0x3f;
            self.set_pixel(frame, pos, color);
        }

        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline == 261 && self.cycle == 0 {
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

    pub fn cpu_read_ro(&self, addr: u16) -> u8 {
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

    pub fn get_pattern_table(
        &self,
        cart: &mut Cartridge,
        table_idx: usize,
        palette: usize,
    ) -> PatternTable {
        let mut table = [[0; 4 * 128]; 128];
        for tile_y in 0..16 {
            for tile_x in 0..16 {
                let offset = tile_y * 256 + tile_x * 16;

                for row in 0..8 {
                    let mut tile_lsb =
                        self.ppu_read(cart, (table_idx * 0x1000 + offset + row) as u16);
                    let mut tile_msb =
                        self.ppu_read(cart, (table_idx * 0x1000 + offset + row + 8) as u16);

                    for col in 0..8 {
                        let pixel_value = (tile_lsb & 0x01) + (tile_msb & 0x01);
                        tile_lsb >>= 1;
                        tile_msb >>= 1;

                        let pos_x = (tile_x * 8 + (7 - col)) * 4;
                        let pos_y = tile_y * 8 + row;
                        table[pos_y][pos_x..pos_x + 4].copy_from_slice(
                            self.get_color_from_palette(cart, palette, pixel_value),
                        );
                    }
                }
            }
        }
        table
    }

    pub fn get_color_from_palette(
        &self,
        cart: &mut Cartridge,
        palette: usize,
        pixel_value: u8,
    ) -> &PixelRgba {
        let offset = 0x3f00 + ((palette as u16) << 2) + pixel_value as u16;
        let color_idx = self.ppu_read(cart, offset);
        &PALETTE_2C02[color_idx as usize]
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
                let off2 = ((py + 1) * self.render_params.width_y + px)
                    * self.render_params.bytes_per_pixel;
                let off3 = ((py + 1) * self.render_params.width_y + px + 1)
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

    fn ppu_read(&self, cart: &mut Cartridge, mut addr: u16) -> u8 {
        addr &= 0x3fff;

        if let Some(data) = cart.ppu_read(addr) {
            data
        } else {
            match addr {
                0x0000..=0x1fff => {
                    self.tbl_pattern[((addr & 0x1000) >> 12) as usize][(addr & 0x0fff) as usize]
                }
                0x2000..=0x3eff => 0,
                0x3f00..=0x3fff => {
                    addr &= 0x001f;
                    addr = match addr {
                        0x0010 | 0x0014 | 0x0018 | 0x001c => addr - 0x0010,
                        _ => addr,
                    };
                    self.tbl_palette[addr as usize]
                }
                _ => 0,
            }
        }
    }

    fn ppu_write(&mut self, cart: &mut Cartridge, mut addr: u16, data: u8) {
        addr &= 0x3fff;

        if !cart.ppu_write(addr, data) {
            match addr {
                0x0000..=0x1fff => {
                    self.tbl_pattern[((addr >> 0x1000) >> 12) as usize][(addr & 0x0fff) as usize] =
                        data;
                }
                0x2000..=0x3eff => {}
                0x3f00..=0x3fff => {
                    addr &= 0x001f;
                    addr = match addr {
                        0x0010 | 0x0014 | 0x0018 | 0x001c => addr - 0x0010,
                        _ => addr,
                    };
                    self.tbl_palette[addr as usize] = data;
                }
                _ => {}
            }
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
