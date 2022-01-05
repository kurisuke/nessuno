pub mod palette;

use crate::cartridge::{Cartridge, Mirror};

pub struct Ppu {
    tbl_pattern: [[u8; 4 * 1024]; 2],
    tbl_name: [[u8; 1024]; 2],
    tbl_palette: [u8; 32],

    scanline: isize,
    cycle: usize,

    control: ControlReg,
    mask: MaskReg,
    status: StatusReg,

    vram_addr: LoopyReg,
    tram_addr: LoopyReg,
    fine_x: u8,

    address_latch: bool,
    ppu_data_buffer: u8,

    bg_next_tile_id: u8,
    bg_next_tile_attrib: u8,
    bg_next_tile_lsb: u8,
    bg_next_tile_msb: u8,
    bg_shifter_pattern_lo: u16,
    bg_shifter_pattern_hi: u16,
    bg_shifter_attrib_lo: u16,
    bg_shifter_attrib_hi: u16,

    oam: [OamEntry; 64],
    oam_addr: u8,

    sprite_scanline: [OamEntry; 8],
    sprite_count: usize,
    sprite_shifter_pattern_lo: [u8; 8],
    sprite_shifter_pattern_hi: [u8; 8],

    sprite_zero_hit_possible: bool,
    sprite_zero_rendered: bool,
}

pub struct PpuClockResult {
    pub nmi: bool,
    pub set_pixel: Option<SetPixel>,
    pub frame_complete: bool,
}

pub struct SetPixel {
    pub pos: (usize, usize),
    pub color: usize,
}

pub type PatternTable = [[usize; 128]; 128];

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            tbl_pattern: [[0; 4 * 1024]; 2],
            tbl_name: [[0; 1024]; 2],
            tbl_palette: [0; 32],

            cycle: 0,
            scanline: 0,

            control: ControlReg { reg: 0x00 },
            mask: MaskReg { reg: 0x00 },
            status: StatusReg { reg: 0x00 },

            vram_addr: LoopyReg { reg: 0x0000 },
            tram_addr: LoopyReg { reg: 0x0000 },
            fine_x: 0x00,

            address_latch: false,
            ppu_data_buffer: 0x00,

            bg_next_tile_id: 0x00,
            bg_next_tile_attrib: 0x00,
            bg_next_tile_lsb: 0x00,
            bg_next_tile_msb: 0x00,
            bg_shifter_pattern_lo: 0x0000,
            bg_shifter_pattern_hi: 0x0000,
            bg_shifter_attrib_lo: 0x0000,
            bg_shifter_attrib_hi: 0x0000,

            oam: [OamEntry {
                bytes: [0, 0, 0, 0],
            }; 64],
            oam_addr: 0x00,

            sprite_scanline: [OamEntry {
                bytes: [0, 0, 0, 0],
            }; 8],
            sprite_count: 0,
            sprite_shifter_pattern_lo: [0x00; 8],
            sprite_shifter_pattern_hi: [0x00; 8],

            sprite_zero_hit_possible: false,
            sprite_zero_rendered: false,
        }
    }

    pub fn reset(&mut self) {
        self.fine_x = 0x00;
        self.address_latch = false;
        self.ppu_data_buffer = 0x00;
        self.scanline = 0;
        self.cycle = 0;
        self.bg_next_tile_id = 0x00;
        self.bg_next_tile_attrib = 0x00;
        self.bg_next_tile_lsb = 0x00;
        self.bg_next_tile_msb = 0x00;
        self.bg_shifter_pattern_lo = 0x0000;
        self.bg_shifter_pattern_hi = 0x0000;
        self.bg_shifter_attrib_lo = 0x0000;
        self.bg_shifter_attrib_hi = 0x0000;
        self.status.reg = 0x00;
        self.mask.reg = 0x00;
        self.control.reg = 0x00;
        self.vram_addr.reg = 0x0000;
        self.tram_addr.reg = 0x0000;
        self.oam = [OamEntry {
            bytes: [0, 0, 0, 0],
        }; 64];
        self.oam_addr = 0x00;
        self.sprite_scanline = [OamEntry {
            bytes: [0, 0, 0, 0],
        }; 8];
        self.sprite_count = 0;
        self.sprite_shifter_pattern_lo = [0x00; 8];
        self.sprite_shifter_pattern_hi = [0x00; 8];
        self.sprite_zero_hit_possible = false;
        self.sprite_zero_rendered = false;
    }

    pub fn clock(&mut self, cart: &mut Cartridge) -> PpuClockResult {
        let mut res = PpuClockResult {
            nmi: false,
            set_pixel: None,
            frame_complete: false,
        };

        match self.scanline {
            -1..=239 => {
                if self.scanline == 0 && self.cycle == 0 {
                    // "Odd frame" cycle skip
                    self.cycle = 1;
                }

                if self.scanline == -1 && self.cycle == 1 {
                    // start of new cycle, clear flags
                    self.status.set_flag(StatusRegFlag::VerticalBlank, false);
                    self.status.set_flag(StatusRegFlag::SpriteZeroHit, false);
                    self.status.set_flag(StatusRegFlag::SpriteOverflow, false);

                    for i in 0..8 {
                        self.sprite_shifter_pattern_lo[i] = 0;
                        self.sprite_shifter_pattern_hi[i] = 0;
                    }
                }

                if (self.cycle >= 2 && self.cycle < 258) || (self.cycle >= 321 && self.cycle < 338)
                {
                    self.update_shifters();

                    match (self.cycle - 1) % 8 {
                        0 => {
                            // Fetch next bg tile id
                            self.load_bg_shifters();
                            self.bg_next_tile_id =
                                self.ppu_read(cart, 0x2000 | (self.vram_addr.reg & 0x0fff));
                        }
                        2 => {
                            // Fetch next bg tile attrib
                            let read_addr = 0x23c0
                                | (self.vram_addr.nametable_y() << 11)
                                | (self.vram_addr.nametable_x() << 10)
                                | ((self.vram_addr.coarse_y() >> 2) << 3)
                                | (self.vram_addr.coarse_x() >> 2);
                            self.bg_next_tile_attrib = self.ppu_read(cart, read_addr);

                            if (self.vram_addr.coarse_y() & 0x02) != 0 {
                                self.bg_next_tile_attrib >>= 4;
                            }
                            if (self.vram_addr.coarse_x() & 0x02) != 0 {
                                self.bg_next_tile_attrib >>= 2;
                            }
                            self.bg_next_tile_attrib &= 0x03;
                        }
                        4 => {
                            // Fetch next bg tile LSB bit plane
                            let read_addr =
                                ((self.control.get_flag(ControlRegFlag::PatternBg) as u16) << 12)
                                    + ((self.bg_next_tile_id as u16) << 4)
                                    + self.vram_addr.fine_y();
                            self.bg_next_tile_lsb = self.ppu_read(cart, read_addr);
                        }
                        6 => {
                            // Fetch next bg tile MSB bit plane
                            let read_addr =
                                ((self.control.get_flag(ControlRegFlag::PatternBg) as u16) << 12)
                                    + ((self.bg_next_tile_id as u16) << 4)
                                    + self.vram_addr.fine_y()
                                    + 8;
                            self.bg_next_tile_msb = self.ppu_read(cart, read_addr);
                        }
                        7 => {
                            self.increment_scroll_x();
                        }
                        _ => {} // do nothing
                    }
                }

                if self.cycle == 256 {
                    self.increment_scroll_y();
                }

                if self.cycle == 257 {
                    self.load_bg_shifters();
                    self.transfer_address_x();
                }

                if self.cycle == 338 || self.cycle == 340 {
                    self.bg_next_tile_id =
                        self.ppu_read(cart, 0x2000 | (self.vram_addr.reg & 0x0fff));
                }

                if self.scanline == -1 && self.cycle >= 280 && self.cycle < 305 {
                    self.transfer_address_y();
                }

                // FOREGROUND RENDERING
                if self.cycle == 257 && self.scanline >= 0 {
                    for s in self.sprite_scanline.iter_mut() {
                        s.clear(0xff);
                    }

                    self.sprite_count = 0;
                    let sprite_size = if self.control.get_flag(ControlRegFlag::SpriteSize) {
                        16
                    } else {
                        8
                    };
                    self.sprite_zero_hit_possible = false;
                    for (i, oam_entry) in self.oam.iter().enumerate() {
                        let diff = self.scanline - oam_entry.y() as isize;
                        if diff >= 0 && diff < sprite_size {
                            if self.sprite_count < 8 {
                                if i == 0 {
                                    self.sprite_zero_hit_possible = true;
                                }
                                self.sprite_scanline[self.sprite_count] = *oam_entry;
                                self.sprite_count += 1
                            }
                            if self.sprite_count >= 8 {
                                break;
                            }
                        }
                    }
                    self.status
                        .set_flag(StatusRegFlag::SpriteOverflow, self.sprite_count >= 8);
                }

                if self.cycle == 340 {
                    for (i, s) in self
                        .sprite_scanline
                        .iter()
                        .take(self.sprite_count)
                        .enumerate()
                    {
                        let scanline_diff = self.scanline - s.y() as isize;

                        let sprite_pattern_addr_lo =
                            if !self.control.get_flag(ControlRegFlag::SpriteSize) {
                                // 8x8 sprites
                                if s.attrib() & 0x80 == 0 {
                                    // not vertically flipped, i.e. normal
                                    ((self.control.get_flag(ControlRegFlag::PatternSprite) as u16)
                                        << 12)
                                        | ((s.id() as u16) << 4)
                                        | (scanline_diff & 0x07) as u16
                                } else {
                                    // vertically flipped
                                    ((self.control.get_flag(ControlRegFlag::PatternSprite) as u16)
                                        << 12)
                                        | ((s.id() as u16) << 4)
                                        | ((7 - scanline_diff) & 0x07) as u16
                                }
                            } else {
                                // 8x16 sprites
                                if s.attrib() & 0x80 == 0 {
                                    // not vertically flipped, i.e. normal
                                    if scanline_diff < 8 {
                                        // top half tile
                                        (((s.id() & 0x01) as u16) << 12)
                                            | (((s.id() & 0xfe) as u16) << 4)
                                            | (scanline_diff & 0x07) as u16
                                    } else {
                                        (((s.id() & 0x01) as u16) << 12)
                                            | (((s.id() & 0xfe) as u16 + 1) << 4)
                                            | (scanline_diff & 0x07) as u16
                                    }
                                } else {
                                    // vertically flipped
                                    if scanline_diff < 8 {
                                        // top half tile
                                        (((s.id() & 0x01) as u16) << 12)
                                            | (((s.id() & 0xfe) as u16 + 1) << 4)
                                            | ((7 - scanline_diff) & 0x07) as u16
                                    } else {
                                        (((s.id() & 0x01) as u16) << 12)
                                            | (((s.id() & 0xfe) as u16) << 4)
                                            | ((7 - scanline_diff) & 0x07) as u16
                                    }
                                }
                            };
                        let sprite_pattern_addr_hi = sprite_pattern_addr_lo + 8;
                        let sprite_pattern_bits_lo = self.ppu_read(cart, sprite_pattern_addr_lo);
                        let sprite_pattern_bits_hi = self.ppu_read(cart, sprite_pattern_addr_hi);

                        let (sprite_pattern_bits_lo, sprite_pattern_bits_hi) =
                            if s.attrib() & 0x40 > 0 {
                                // horizontally flipped
                                (
                                    sprite_pattern_bits_lo.reverse_bits(),
                                    sprite_pattern_bits_hi.reverse_bits(),
                                )
                            } else {
                                // not flipped
                                (sprite_pattern_bits_lo, sprite_pattern_bits_hi)
                            };

                        self.sprite_shifter_pattern_lo[i] = sprite_pattern_bits_lo;
                        self.sprite_shifter_pattern_hi[i] = sprite_pattern_bits_hi;
                    }
                }

                if self.cycle == 260
                    && (self.mask.get_flag(MaskRegFlag::RenderBg)
                        || self.mask.get_flag(MaskRegFlag::RenderSprites))
                {
                    cart.on_scanline_end();
                }
            }
            240 => {
                // Post render scanline - do nothing
            }
            241..=260 => {
                if self.scanline == 241 && self.cycle == 1 {
                    // end of frame, start of vblank period
                    // if configured, emit CPU NMI
                    self.status.set_flag(StatusRegFlag::VerticalBlank, true);
                    if self.control.get_flag(ControlRegFlag::EnableNmi) {
                        res.nmi = true;
                    }
                }
            }
            _ => {
                unreachable!();
            }
        }

        if let Some(pos) = visible(self.scanline, self.cycle) {
            // BACKGROUND
            let (bg_palette, bg_palette_idx) = if self.mask.get_flag(MaskRegFlag::RenderBg) {
                // fine x scrolling
                let bit_mux = 0x8000 >> self.fine_x;

                // get palette idx
                let p0_pixel = ((self.bg_shifter_pattern_lo & bit_mux) != 0) as u8;
                let p1_pixel = ((self.bg_shifter_pattern_hi & bit_mux) != 0) as u8;
                let bg_palette_idx = (p1_pixel << 1) | p0_pixel;

                // get palette
                let bg_pal0 = ((self.bg_shifter_attrib_lo & bit_mux) != 0) as u8;
                let bg_pal1 = ((self.bg_shifter_attrib_hi & bit_mux) != 0) as u8;
                let bg_palette = (bg_pal1 << 1) | bg_pal0;
                (bg_palette, bg_palette_idx)
            } else {
                (0x00, 0x00)
            };

            // FOREGROUND
            let (fg_palette, fg_palette_idx, fg_priority) = if self
                .mask
                .get_flag(MaskRegFlag::RenderSprites)
            {
                self.sprite_zero_rendered = false;

                let mut fg_palette = 0x00;
                let mut fg_palette_idx = 0x00;
                let mut fg_priority = false;
                for (i, s) in self
                    .sprite_scanline
                    .iter_mut()
                    .take(self.sprite_count)
                    .enumerate()
                {
                    if s.x() == 0 {
                        let fg_pixel_lo = ((self.sprite_shifter_pattern_lo[i] & 0x80) != 0) as u8;
                        let fg_pixel_hi = ((self.sprite_shifter_pattern_hi[i] & 0x80) != 0) as u8;
                        fg_palette_idx = (fg_pixel_hi << 1) | fg_pixel_lo;

                        fg_palette = (s.attrib() & 0x03) + 0x04;
                        fg_priority = (s.attrib() & 0x20) == 0;

                        if fg_palette_idx != 0 {
                            if i == 0 {
                                self.sprite_zero_rendered = true;
                            }
                            break;
                        }
                    }
                }
                (fg_palette, fg_palette_idx, fg_priority)
            } else {
                (0x00, 0x00, false)
            };

            let (palette, palette_idx) = if bg_palette_idx == 0 && fg_palette_idx == 0 {
                // both transparent, draw background
                (0x00, 0x00)
            } else if bg_palette_idx == 0 && fg_palette_idx > 0 {
                // bg transparent, fg visible -> fg wins
                (fg_palette, fg_palette_idx)
            } else if bg_palette_idx > 0 && fg_palette_idx == 0 {
                // bg visible, fg transparent -> bg wins
                (bg_palette, bg_palette_idx)
            } else if bg_palette_idx > 0 && fg_palette_idx > 0 {
                // sprite zero hit detection
                if self.sprite_zero_hit_possible
                    && self.sprite_zero_rendered
                    && self.mask.get_flag(MaskRegFlag::RenderBg)
                    && self.mask.get_flag(MaskRegFlag::RenderSprites)
                {
                    if !(self.mask.get_flag(MaskRegFlag::RenderBgLeft)
                        || self.mask.get_flag(MaskRegFlag::RenderSpritesLeft))
                    {
                        if self.cycle >= 9 && self.cycle < 258 {
                            self.status.set_flag(StatusRegFlag::SpriteZeroHit, true);
                        }
                    } else {
                        if self.cycle >= 1 && self.cycle < 258 {
                            self.status.set_flag(StatusRegFlag::SpriteZeroHit, true);
                        }
                    }
                }

                // both visible -> eval fg priority flag
                if fg_priority {
                    (fg_palette, fg_palette_idx)
                } else {
                    (bg_palette, bg_palette_idx)
                }
            } else {
                unreachable!()
            };

            let color = self.get_color_from_palette(cart, palette as usize, palette_idx);
            res.set_pixel = Some(SetPixel { pos, color });
        }

        self.cycle += 1;
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline >= 261 {
                self.scanline = -1;
                res.frame_complete = true;
            }
        }

        res
    }

    fn increment_scroll_x(&mut self) {
        if self.mask.get_flag(MaskRegFlag::RenderBg)
            || self.mask.get_flag(MaskRegFlag::RenderSprites)
        {
            if self.vram_addr.coarse_x() == 31 {
                // wrap & switch to other table
                self.vram_addr.set_coarse_x(0);
                self.vram_addr.flip_nametable_x();
            } else {
                self.vram_addr.set_coarse_x(self.vram_addr.coarse_x() + 1);
            }
        }
    }

    fn increment_scroll_y(&mut self) {
        if self.mask.get_flag(MaskRegFlag::RenderBg)
            || self.mask.get_flag(MaskRegFlag::RenderSprites)
        {
            if self.vram_addr.fine_y() < 7 {
                self.vram_addr.set_fine_y(self.vram_addr.fine_y() + 1);
            } else {
                self.vram_addr.set_fine_y(0);

                match self.vram_addr.coarse_y() {
                    29 => {
                        // wrap & switch to other table
                        self.vram_addr.set_coarse_y(0);
                        self.vram_addr.flip_nametable_y();
                    }
                    31 => {
                        // ptr in attribute memory
                        // wrap without nametable switc
                        self.vram_addr.set_coarse_y(0);
                    }
                    _ => {
                        self.vram_addr.set_coarse_y(self.vram_addr.coarse_y() + 1);
                    }
                }
            }
        }
    }

    fn transfer_address_x(&mut self) {
        if self.mask.get_flag(MaskRegFlag::RenderBg)
            || self.mask.get_flag(MaskRegFlag::RenderSprites)
        {
            self.vram_addr.set_nametable_x(self.tram_addr.nametable_x());
            self.vram_addr.set_coarse_x(self.tram_addr.coarse_x());
        }
    }

    fn transfer_address_y(&mut self) {
        if self.mask.get_flag(MaskRegFlag::RenderBg)
            || self.mask.get_flag(MaskRegFlag::RenderSprites)
        {
            self.vram_addr.set_fine_y(self.tram_addr.fine_y());
            self.vram_addr.set_nametable_y(self.tram_addr.nametable_y());
            self.vram_addr.set_coarse_y(self.tram_addr.coarse_y());
        }
    }

    fn load_bg_shifters(&mut self) {
        self.bg_shifter_pattern_lo =
            (self.bg_shifter_pattern_lo & 0xff00) | (self.bg_next_tile_lsb as u16);
        self.bg_shifter_pattern_hi =
            (self.bg_shifter_pattern_hi & 0xff00) | (self.bg_next_tile_msb as u16);

        let tmp = if (self.bg_next_tile_attrib & 0b01) != 0 {
            0xff
        } else {
            0x00
        };
        self.bg_shifter_attrib_lo = (self.bg_shifter_attrib_lo & 0xff00) | tmp;
        let tmp = if (self.bg_next_tile_attrib & 0b10) != 0 {
            0xff
        } else {
            0x00
        };
        self.bg_shifter_attrib_hi = (self.bg_shifter_attrib_hi & 0xff00) | tmp;
    }

    fn update_shifters(&mut self) {
        if self.mask.get_flag(MaskRegFlag::RenderBg) {
            self.bg_shifter_pattern_lo <<= 1;
            self.bg_shifter_pattern_hi <<= 1;

            self.bg_shifter_attrib_lo <<= 1;
            self.bg_shifter_attrib_hi <<= 1;
        }

        if self.mask.get_flag(MaskRegFlag::RenderSprites) && self.cycle >= 1 && self.cycle < 258 {
            for (i, s) in self
                .sprite_scanline
                .iter_mut()
                .take(self.sprite_count)
                .enumerate()
            {
                if s.x() > 0 {
                    s.dec_x();
                } else {
                    self.sprite_shifter_pattern_lo[i] <<= 1;
                    self.sprite_shifter_pattern_hi[i] <<= 1;
                }
            }
        }
    }

    pub fn cpu_read(&mut self, cart: &mut Cartridge, addr: u16) -> u8 {
        // Some PPU registers change the internal state (mut)
        // in response to a read
        match addr {
            0x0000 => 0x00, // Control - not readable
            0x0001 => 0x00, // Mask - not readable
            0x0002 => {
                // Status

                // only top 3 bits of the status reg are returned
                // the other values are undefined, most likely last data buffer values
                let data = (self.status.reg & 0xe0) | (self.ppu_data_buffer & 0x1f);

                // clear vertical blank flag
                self.status.set_flag(StatusRegFlag::VerticalBlank, false);

                // reset adress latch flag
                self.address_latch = false;
                data
            }
            0x0003 => 0x00, // OAM Address
            0x0004 => {
                // OAM Data
                let entry = self.oam_addr / 4;
                let offset = self.oam_addr % 4;
                self.oam[entry as usize].bytes[offset as usize]
            }
            0x0005 => 0x00, // Scroll - not readable
            0x0006 => 0x00, // PPU Address - not readable
            0x0007 => {
                // PPU Data

                // reads from the name table are delayed one cycle
                // get last read result from buffer
                let mut data = self.ppu_data_buffer;
                // store current read result in buffer
                self.ppu_data_buffer = self.ppu_read(cart, self.vram_addr.reg);

                // special case: palette memory is returned without cycle delay
                if self.vram_addr.reg > 0x3f00 {
                    data = self.ppu_data_buffer;
                }
                self.vram_addr.reg += if self.control.get_flag(ControlRegFlag::IncrementMode) {
                    32
                } else {
                    1
                };
                data
            }
            _ => unreachable!(),
        }
    }

    pub fn cpu_read_ro(&self, addr: u16) -> u8 {
        match addr {
            0x0000 => {
                // Control
                self.control.reg
            }
            0x0001 => {
                // Mask
                self.mask.reg
            }
            0x0002 => {
                // Status
                self.status.reg
            }
            0x0003 => 0x00, // OAM Address
            0x0004 => 0x00, // OAM Data
            0x0005 => 0x00, // Scroll
            0x0006 => 0x00, // PPU Address
            0x0007 => 0x00, // PPU Data
            _ => unreachable!(),
        }
    }

    pub fn cpu_write(&mut self, cart: &mut Cartridge, addr: u16, data: u8) {
        match addr {
            0x0000 => {
                // Control
                self.control.reg = data;
                self.tram_addr
                    .set_nametable_x(self.control.get_flag(ControlRegFlag::NametableX) as u16);
                self.tram_addr
                    .set_nametable_y(self.control.get_flag(ControlRegFlag::NametableY) as u16);
            }
            0x0001 => {
                // Mask
                self.mask.reg = data;
            }
            0x0002 => {
                // Status
                // not writable
            }
            0x0003 => {
                // OAM Address
                self.oam_addr = data;
            }
            0x0004 => {
                // OAM Data
                let entry = self.oam_addr / 4;
                let offset = self.oam_addr % 4;
                self.oam[entry as usize].bytes[offset as usize] = data;
            }
            0x0005 => {
                // Scroll
                if !self.address_latch {
                    // X offset
                    self.fine_x = data & 0x07;
                    self.tram_addr.set_coarse_x((data >> 3) as u16);
                } else {
                    // Y offset
                    self.tram_addr.set_fine_y((data & 0x07) as u16);
                    self.tram_addr.set_coarse_y((data >> 3) as u16);
                }
                self.address_latch = !self.address_latch;
            }
            0x0006 => {
                // PPU Address
                if !self.address_latch {
                    // write high byte
                    self.tram_addr.reg =
                        (((data & 0x3f) as u16) << 8) | (self.tram_addr.reg & 0x00ff);
                } else {
                    // write low byte & update vram address
                    self.tram_addr.reg = (self.tram_addr.reg & 0xff00) | (data as u16);
                    self.vram_addr.reg = self.tram_addr.reg;
                }
                self.address_latch = !self.address_latch;
            }
            0x0007 => {
                // PPU data
                self.ppu_write(cart, self.vram_addr.reg, data);
                self.vram_addr.reg += if self.control.get_flag(ControlRegFlag::IncrementMode) {
                    32
                } else {
                    1
                };
            }
            _ => unreachable!(),
        }
    }

    pub fn write_oam(&mut self, addr: u8, data: u8) {
        let entry = addr / 4;
        let offset = addr % 4;
        self.oam[entry as usize].bytes[offset as usize] = data;
    }

    pub fn debug_oam(&self, entry: usize) -> String {
        format!("{:02x}: {}", entry, self.oam[entry].debug_string())
    }

    pub fn get_pattern_table(
        &self,
        cart: &mut Cartridge,
        table_idx: usize,
        palette: usize,
    ) -> PatternTable {
        let mut table = [[0; 128]; 128];
        for tile_y in 0..16 {
            for tile_x in 0..16 {
                let offset = tile_y * 256 + tile_x * 16;

                for row in 0..8 {
                    let mut tile_lsb =
                        self.ppu_read(cart, (table_idx * 0x1000 + offset + row) as u16);
                    let mut tile_msb =
                        self.ppu_read(cart, (table_idx * 0x1000 + offset + row + 8) as u16);

                    for col in 0..8 {
                        let pixel_value = ((tile_lsb & 0x01) << 1) | (tile_msb & 0x01);
                        tile_lsb >>= 1;
                        tile_msb >>= 1;

                        let pos_x = tile_x * 8 + (7 - col);
                        let pos_y = tile_y * 8 + row;
                        table[pos_y][pos_x] =
                            self.get_color_from_palette(cart, palette, pixel_value);
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
    ) -> usize {
        let offset = 0x3f00 + ((palette as u16) << 2) + pixel_value as u16;
        let color_idx = self.ppu_read(cart, offset);
        color_idx as usize
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
                0x2000..=0x3eff => {
                    addr &= 0x0fff;

                    match cart.mirror() {
                        Mirror::Vertical => match addr {
                            0x0000..=0x03ff | 0x0800..=0x0bff => {
                                self.tbl_name[0][(addr & 0x03ff) as usize]
                            }
                            0x0400..=0x07ff | 0x0c00..=0x0fff => {
                                self.tbl_name[1][(addr & 0x03ff) as usize]
                            }
                            _ => 0x00,
                        },
                        Mirror::Horizontal => match addr {
                            0x0000..=0x07ff => self.tbl_name[0][(addr & 0x03ff) as usize],
                            0x0800..=0x0fff => self.tbl_name[1][(addr & 0x03ff) as usize],
                            _ => 0x00,
                        },
                        Mirror::OneScreenLo => self.tbl_name[0][(addr & 0x03ff) as usize],
                        Mirror::OneScreenHi => self.tbl_name[1][(addr & 0x03ff) as usize],
                        _ => 0x00,
                    }
                }
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
                    self.tbl_pattern[((addr & 0x1000) >> 12) as usize][(addr & 0x0fff) as usize] =
                        data;
                }
                0x2000..=0x3eff => {
                    addr &= 0x0fff;

                    match cart.mirror() {
                        Mirror::Vertical => match addr {
                            0x0000..=0x03ff | 0x0800..=0x0bff => {
                                self.tbl_name[0][(addr & 0x03ff) as usize] = data;
                            }
                            0x0400..=0x07ff | 0x0c00..=0x0fff => {
                                self.tbl_name[1][(addr & 0x03ff) as usize] = data;
                            }
                            _ => {}
                        },
                        Mirror::Horizontal => match addr {
                            0x0000..=0x07ff => {
                                self.tbl_name[0][(addr & 0x03ff) as usize] = data;
                            }
                            0x0800..=0x0fff => {
                                self.tbl_name[1][(addr & 0x03ff) as usize] = data;
                            }
                            _ => {}
                        },
                        Mirror::OneScreenLo => {
                            self.tbl_name[0][(addr & 0x03ff) as usize] = data;
                        }
                        Mirror::OneScreenHi => {
                            self.tbl_name[1][(addr & 0x03ff) as usize] = data;
                        }
                        _ => {}
                    }
                }
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

/// PPU Registers

struct StatusReg {
    pub reg: u8,
}

enum StatusRegFlag {
    SpriteOverflow,
    SpriteZeroHit,
    VerticalBlank,
}

impl StatusReg {
    pub fn get_flag(&self, f: StatusRegFlag) -> bool {
        self.reg & f.mask() != 0
    }

    fn set_flag(&mut self, f: StatusRegFlag, v: bool) {
        if v {
            // set flag
            self.reg |= f.mask();
        } else {
            // clear flag
            self.reg &= !f.mask();
        }
    }
}

impl StatusRegFlag {
    fn mask(&self) -> u8 {
        match self {
            StatusRegFlag::SpriteOverflow => 1 << 5,
            StatusRegFlag::SpriteZeroHit => 1 << 6,
            StatusRegFlag::VerticalBlank => 1 << 7,
        }
    }
}

struct MaskReg {
    pub reg: u8,
}

enum MaskRegFlag {
    Grayscale,
    RenderBgLeft,
    RenderSpritesLeft,
    RenderBg,
    RenderSprites,
    EnhanceRed,
    EnhanceGreen,
    EnhanceBlue,
}

impl MaskReg {
    pub fn get_flag(&self, f: MaskRegFlag) -> bool {
        self.reg & f.mask() != 0
    }

    fn set_flag(&mut self, f: MaskRegFlag, v: bool) {
        if v {
            // set flag
            self.reg |= f.mask();
        } else {
            // clear flag
            self.reg &= !f.mask();
        }
    }
}

impl MaskRegFlag {
    fn mask(&self) -> u8 {
        match self {
            MaskRegFlag::Grayscale => 1 << 0,
            MaskRegFlag::RenderBgLeft => 1 << 1,
            MaskRegFlag::RenderSpritesLeft => 1 << 2,
            MaskRegFlag::RenderBg => 1 << 3,
            MaskRegFlag::RenderSprites => 1 << 4,
            MaskRegFlag::EnhanceRed => 1 << 5,
            MaskRegFlag::EnhanceGreen => 1 << 6,
            MaskRegFlag::EnhanceBlue => 1 << 7,
        }
    }
}

struct ControlReg {
    pub reg: u8,
}

enum ControlRegFlag {
    NametableX,
    NametableY,
    IncrementMode,
    PatternSprite,
    PatternBg,
    SpriteSize,
    SlaveMode,
    EnableNmi,
}

impl ControlReg {
    pub fn get_flag(&self, f: ControlRegFlag) -> bool {
        self.reg & f.mask() != 0
    }

    fn set_flag(&mut self, f: ControlRegFlag, v: bool) {
        if v {
            // set flag
            self.reg |= f.mask();
        } else {
            // clear flag
            self.reg &= !f.mask();
        }
    }
}

impl ControlRegFlag {
    fn mask(&self) -> u8 {
        match self {
            ControlRegFlag::NametableX => 1 << 0,
            ControlRegFlag::NametableY => 1 << 1,
            ControlRegFlag::IncrementMode => 1 << 2,
            ControlRegFlag::PatternSprite => 1 << 3,
            ControlRegFlag::PatternBg => 1 << 4,
            ControlRegFlag::SpriteSize => 1 << 5,
            ControlRegFlag::SlaveMode => 1 << 6,
            ControlRegFlag::EnableNmi => 1 << 7,
        }
    }
}

struct LoopyReg {
    reg: u16,
}

impl LoopyReg {
    fn coarse_x(&self) -> u16 {
        self.reg & 0x001f
    }

    fn set_coarse_x(&mut self, v: u16) {
        self.reg &= 0xffe0;
        self.reg |= v & 0x001f;
    }

    fn coarse_y(&self) -> u16 {
        (self.reg & 0x03e0) >> 5
    }

    fn set_coarse_y(&mut self, v: u16) {
        self.reg &= 0xfc1f;
        self.reg |= (v << 5) & 0x03e0;
    }

    fn nametable_x(&self) -> u16 {
        (self.reg & 0x0400) >> 10
    }

    fn set_nametable_x(&mut self, v: u16) {
        self.reg &= 0xfbff;
        self.reg |= (v << 10) & 0x0400;
    }

    fn flip_nametable_x(&mut self) {
        let v = self.nametable_x() != 0;
        self.set_nametable_x(!v as u16);
    }

    fn nametable_y(&self) -> u16 {
        (self.reg & 0x0800) >> 11
    }

    fn set_nametable_y(&mut self, v: u16) {
        self.reg &= 0xf7ff;
        self.reg |= (v << 11) & 0x0800;
    }

    fn flip_nametable_y(&mut self) {
        let v = self.nametable_y() != 0;
        self.set_nametable_y(!v as u16);
    }

    fn fine_y(&self) -> u16 {
        (self.reg & 0x7000) >> 12
    }

    fn set_fine_y(&mut self, v: u16) {
        self.reg &= 0x8fff;
        self.reg |= (v << 12) & 0x7000;
    }
}

#[derive(Copy, Clone)]
struct OamEntry {
    pub bytes: [u8; 4],
}

impl OamEntry {
    pub fn y(&self) -> u8 {
        self.bytes[0]
    }

    pub fn id(&self) -> u8 {
        self.bytes[1]
    }

    pub fn attrib(&self) -> u8 {
        self.bytes[2]
    }

    pub fn x(&self) -> u8 {
        self.bytes[3]
    }

    pub fn dec_x(&mut self) {
        self.bytes[3] -= 1;
    }

    pub fn clear(&mut self, v: u8) {
        self.bytes[0] = v;
        self.bytes[1] = v;
        self.bytes[2] = v;
        self.bytes[3] = v;
    }

    pub fn debug_string(&self) -> String {
        format!(
            "({:03}, {:03}) ID: {:02x} AT: {:02x}",
            self.bytes[3], self.bytes[0], self.bytes[1], self.bytes[2]
        )
    }
}

fn visible(scanline: isize, cycle: usize) -> Option<(usize, usize)> {
    if scanline >= 0 && scanline < 240 && cycle >= 1 && cycle <= 256 {
        Some((scanline as usize, cycle - 1))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loopy_reg() {
        let mut reg = LoopyReg { reg: 0 };
        for cx in 0..32 {
            for cy in 0..32 {
                for ntx in 0..2 {
                    for nty in 0..2 {
                        for fy in 0..8 {
                            reg.set_coarse_x(cx);
                            reg.set_coarse_y(cy);
                            reg.set_nametable_x(ntx);
                            reg.set_nametable_y(nty);
                            reg.set_fine_y(fy);

                            assert_eq!(reg.coarse_x(), cx);
                            assert_eq!(reg.coarse_y(), cy);
                            assert_eq!(reg.nametable_x(), ntx);
                            assert_eq!(reg.nametable_y(), nty);
                            assert_eq!(reg.fine_y(), fy);

                            reg.flip_nametable_x();
                            assert_eq!(reg.nametable_x(), (ntx == 0) as u16);
                            reg.flip_nametable_x();
                            assert_eq!(reg.nametable_x(), (ntx != 0) as u16);

                            reg.flip_nametable_y();
                            assert_eq!(reg.nametable_y(), (nty == 0) as u16);
                            reg.flip_nametable_y();
                            assert_eq!(reg.nametable_y(), (nty != 0) as u16);
                        }
                    }
                }
            }
        }
    }
}
