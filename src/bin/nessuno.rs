use nessuno::cartridge::Cartridge;
use nessuno::cpu::{Disassembly, Flag};
use nessuno::ppu::{PatternTable, PixelRgba, PpuRenderParams};
use nessuno::screen::backend::{Frame, ScreenBackend};
use nessuno::screen::textwriter::{TextScreenParams, TextWriter};
use nessuno::screen::{Screen, ScreenParams};
use nessuno::system::System;
use std::env;
use std::io;
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

const SCREEN_WIDTH: u32 = 960;
const SCREEN_HEIGHT: u32 = 540;

const FG_COLOR: [u8; 4] = [0xff, 0xff, 0xff, 0xff];
const BG_COLOR: [u8; 4] = [0x00, 0x00, 0x7f, 0xff];
const OFF_COLOR: [u8; 4] = [0xbf, 0x00, 0x00, 0xff];
const ON_COLOR: [u8; 4] = [0x00, 0xbf, 0x00, 0xff];
const HL_COLOR: [u8; 4] = [0xbf, 0xbf, 0xff, 0xff];

const FRAME_DURATION: f64 = 1f64 / 60f64;

struct Nessuno {
    system: System,
    disasm: Disassembly,
    text_writer: TextWriter,

    run: bool,
    t_residual: f64,
    action: Option<UserAction>,
    palette_selected: usize,
    paint: bool,
}

enum UserAction {
    Reset,
    Step,
    Frame,
    PaletteSelect,
}

impl Nessuno {
    fn print_reg(&self, frame: &mut [u8], pos_x: i32, pos_y: i32) {
        self.print_status(frame, pos_x, pos_y);
        self.text_writer.write(
            frame,
            pos_x,
            pos_y + 1,
            &format!("PC: ${:04x}", self.system.cpu.pc),
            &FG_COLOR,
            &BG_COLOR,
        );
        self.text_writer.write(
            frame,
            pos_x,
            pos_y + 2,
            &format!("A:  ${:02x}   [{:3}]", self.system.cpu.a, self.system.cpu.a),
            &FG_COLOR,
            &BG_COLOR,
        );
        self.text_writer.write(
            frame,
            pos_x,
            pos_y + 3,
            &format!("X:  ${:02x}   [{:3}]", self.system.cpu.x, self.system.cpu.x),
            &FG_COLOR,
            &BG_COLOR,
        );
        self.text_writer.write(
            frame,
            pos_x,
            pos_y + 4,
            &format!("Y:  ${:02x}   [{:3}]", self.system.cpu.y, self.system.cpu.y),
            &FG_COLOR,
            &BG_COLOR,
        );
        self.text_writer.write(
            frame,
            pos_x,
            pos_y + 5,
            &format!("SP: ${:04x}", self.system.cpu.stkp as u16),
            &FG_COLOR,
            &BG_COLOR,
        );
    }

    fn print_status(&self, frame: &mut [u8], pos_x: i32, pos_y: i32) {
        self.text_writer
            .write(frame, pos_x, pos_y, "Status: ", &FG_COLOR, &BG_COLOR);
        for (i, f) in [
            Flag::N,
            Flag::V,
            Flag::U,
            Flag::B,
            Flag::D,
            Flag::I,
            Flag::Z,
            Flag::C,
        ]
        .into_iter()
        .enumerate()
        {
            let color = if self.system.cpu.get_flag(f) {
                &ON_COLOR
            } else {
                &OFF_COLOR
            };
            self.text_writer.write(
                frame,
                pos_x + 8 + (2 * i) as i32,
                pos_y,
                f.ch(),
                color,
                &BG_COLOR,
            );
        }
    }

    fn print_disasm(&self, frame: &mut [u8], addr: u16, pos_x: i32, pos_y: i32, range: usize) {
        // current position
        self.text_writer.write(
            frame,
            pos_x,
            pos_y + range as i32,
            &format!(
                "{:30}",
                &self.disasm.get(&addr).unwrap_or(&String::from(""))
            ),
            &HL_COLOR,
            &BG_COLOR,
        );

        // forward
        let mut it_forward = self.disasm.range(addr..).skip(1);
        for i in 0..range {
            let line = if let Some((_, v)) = it_forward.next() {
                v
            } else {
                ""
            };
            self.text_writer.write(
                frame,
                pos_x,
                pos_y + range as i32 + i as i32 + 1,
                &format!("{:30}", line),
                &FG_COLOR,
                &BG_COLOR,
            );
        }

        // backward
        let mut it_backward = self.disasm.range(..addr);
        for i in 0..range {
            let line = if let Some((_, v)) = it_backward.next_back() {
                v
            } else {
                ""
            };
            self.text_writer.write(
                frame,
                pos_x,
                pos_y + range as i32 - i as i32 - 1,
                &format!("{:30}", line),
                &FG_COLOR,
                &BG_COLOR,
            );
        }
    }

    fn draw_ppu_data(&mut self, frame: &mut [u8]) {
        self.draw_pattern_table(frame, 0, 572, 372);
        self.draw_pattern_table(frame, 1, 756, 372);

        self.text_writer.write(
            frame,
            102,
            30,
            &format!("p={}", self.palette_selected),
            &FG_COLOR,
            &BG_COLOR,
        );

        for palette in 0..8 {
            for pixel_value in 0..4 {
                let color = self
                    .system
                    .ppu_get_color_from_palette(palette, pixel_value)
                    .clone();
                self.fill_rect(
                    frame,
                    572 + palette * 40 + pixel_value as usize * 8,
                    354,
                    8,
                    8,
                    &color,
                );
            }
        }
    }

    fn fill_rect(
        &mut self,
        frame: &mut [u8],
        pos_x: usize,
        pos_y: usize,
        size_x: usize,
        size_y: usize,
        color: &PixelRgba,
    ) {
        for y in pos_y..(pos_y + size_y) {
            for x in pos_x..(pos_x + size_x) {
                let offset_frame = (y * SCREEN_WIDTH as usize + x) * 4;
                frame[offset_frame..offset_frame + 4].copy_from_slice(color);
            }
        }
    }

    fn draw_pattern_table(
        &mut self,
        frame: &mut [u8],
        table_idx: usize,
        pos_x: usize,
        pos_y: usize,
    ) {
        let pattern_table = self
            .system
            .ppu_get_pattern_table(table_idx, self.palette_selected);
        for (offset_y, line) in pattern_table.iter().enumerate() {
            let offset_frame = ((pos_y + offset_y) * SCREEN_WIDTH as usize + pos_x) * 4;
            frame[offset_frame..offset_frame + 128 * 4].copy_from_slice(line);
        }
    }
}

impl ScreenBackend for Nessuno {
    fn init(&self, frame: Frame) {
        for pixel in frame.frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0x00, 0x00, 0x7f, 0xff]);
        }
        self.text_writer.write(
            frame.frame,
            5,
            37,
            "SPACE = run/pause      S = step      R = reset      F = frame      ESC = quit",
            &FG_COLOR,
            &BG_COLOR,
        );
    }

    fn draw(&self, frame: Frame) {
        self.print_reg(frame.frame, 82, 1);
        self.print_disasm(frame.frame, self.system.cpu.pc, 82, 8, 7);
    }

    fn update(&mut self, frame: Frame, dt: f64) {
        if self.run {
            if self.t_residual > 0f64 {
                self.t_residual -= dt;
            } else {
                self.t_residual += FRAME_DURATION - dt;
                self.system.frame(frame.frame, false);
                self.paint = true;
            }
        } else {
            if let Some(action) = &self.action {
                match *action {
                    UserAction::Reset => {
                        self.system.reset();
                        self.draw_ppu_data(frame.frame);
                    }
                    UserAction::Step => {
                        self.system.step(frame.frame);
                    }
                    UserAction::Frame => {
                        self.system.frame(frame.frame, true);
                    }
                    UserAction::PaletteSelect => {
                        self.draw_ppu_data(frame.frame);
                    }
                }
                self.action = None;
                self.paint = true;
            } else {
                self.paint = false;
            }
        }
    }

    fn handle_input(&mut self, input: &WinitInputHelper) {
        if input.key_pressed(VirtualKeyCode::Space) {
            self.run = !self.run;
            if !self.run {
                self.t_residual = 0f64;
            }
        } else if input.key_pressed(VirtualKeyCode::R) {
            self.action = Some(UserAction::Reset);
        } else if input.key_pressed(VirtualKeyCode::F) {
            self.action = Some(UserAction::Frame);
        } else if input.key_pressed(VirtualKeyCode::S) {
            self.action = Some(UserAction::Step);
        } else if input.key_pressed(VirtualKeyCode::P) {
            self.palette_selected += 1;
            self.palette_selected &= 0x07;
            self.action = Some(UserAction::PaletteSelect);
        }
    }
}

impl Nessuno {
    fn new(cart: Cartridge) -> Nessuno {
        let mut system = System::new(
            cart,
            PpuRenderParams {
                offset_x: 30,
                offset_y: 30,
                width_y: SCREEN_WIDTH as usize,
                scaling_factor: 2,
                bytes_per_pixel: 4,
            },
        );
        let disasm = system.cpu_disassemble(0x0000, 0xffff);

        Nessuno {
            system,
            disasm,
            text_writer: TextWriter::new(
                "res/cozette.bdf",
                TextScreenParams {
                    width: SCREEN_WIDTH as usize,
                    height: SCREEN_HEIGHT as usize,
                },
            ),
            run: false,
            t_residual: 0f64,
            action: Some(UserAction::Reset),
            palette_selected: 0,
            paint: false,
        }
    }
}

fn main() -> Result<(), io::Error> {
    let args: Vec<_> = env::args().collect();
    let cart = Cartridge::new(&args[1])?;

    let screen = Screen::new(ScreenParams {
        width: SCREEN_WIDTH,
        height: SCREEN_HEIGHT,
        title: "nessuno",
        backend: Box::new(Nessuno::new(cart)),
    })
    .unwrap();

    screen.run();

    Ok(())
}