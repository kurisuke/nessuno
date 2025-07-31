use bdf_parser::{Font, Property};

pub struct TextWriter {
    font: Font,
    font_params: FontParams,
    screen_params: TextScreenParams,
}

pub struct TextScreenParams {
    pub width: usize,
    pub height: usize,
}

struct FontParams {
    char_size_x: i32,
    char_size_y: i32,
    char_ascent: i32,
}

impl TextWriter {
    pub fn new(font_input: &str, screen_params: TextScreenParams) -> TextWriter {
        let font = Font::parse(font_input).unwrap();
        let font_params = FontParams {
            char_size_x: font
                .metadata
                .properties
                .try_get::<i32>(Property::NormSpace)
                .unwrap()
                .unwrap()
                + 1,
            char_size_y: font
                .metadata
                .properties
                .try_get::<i32>(Property::PixelSize)
                .unwrap()
                .unwrap()
                + 1,
            char_ascent: font
                .metadata
                .properties
                .try_get::<i32>(Property::FontAscent)
                .unwrap()
                .unwrap(),
        };

        TextWriter {
            font,
            font_params,
            screen_params,
        }
    }

    pub fn write(
        &self,
        frame: &mut [u8],
        mut pos_x: i32,
        mut pos_y: i32,
        s: &str,
        fg: &[u8; 4],
        bg: &[u8; 4],
    ) {
        let init_x = pos_x;
        for l in s.lines() {
            for c in l.chars() {
                // paint bg
                for y in 0..self.font_params.char_size_y {
                    for x in 0..self.font_params.char_size_x {
                        let py = (pos_y * self.font_params.char_size_y + y) as usize;
                        let px = (pos_x * self.font_params.char_size_x + x) as usize;
                        if px < self.screen_params.width && py < self.screen_params.height {
                            let p = (py * self.screen_params.width + px) * 4;
                            frame[p..p + 4].copy_from_slice(bg);
                        }
                    }
                }

                // paint fg
                let glyph = self.font.glyphs.get(c).unwrap();

                let start_x = (self.font_params.char_size_x - glyph.bounding_box.size.x) / 2
                    + glyph.bounding_box.offset.x;
                let start_y = self.font_params.char_ascent
                    - glyph.bounding_box.size.y
                    - glyph.bounding_box.offset.y;

                for y in 0..glyph.bounding_box.size.y {
                    for x in 0..glyph.bounding_box.size.x {
                        if glyph.pixel(x as usize, y as usize).unwrap() {
                            let py = (pos_y * self.font_params.char_size_y + start_y + y) as usize;
                            let px = (pos_x * self.font_params.char_size_x + start_x + x) as usize;
                            if px < self.screen_params.width && py < self.screen_params.height {
                                let p = (py * self.screen_params.width + px) * 4;
                                frame[p..p + 4].copy_from_slice(fg);
                            }
                        }
                    }
                }
                pos_x += 1;
            }
            pos_y += 1;
            pos_x = init_x;
        }
    }
}
