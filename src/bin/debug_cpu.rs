use nessuno::bus::dummy::DummyBus;
use nessuno::cpu::{Cpu, Flag};
use nessuno::screen::backend::{Frame, ScreenBackend};
use nessuno::screen::textwriter::{TextScreenParams, TextWriter};
use nessuno::screen::{Screen, ScreenParams};
use std::num::Wrapping;
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 560;

const FG_COLOR: [u8; 4] = [0xff, 0xff, 0xff, 0xff];
const BG_COLOR: [u8; 4] = [0x00, 0x00, 0x7f, 0xff];
const OFF_COLOR: [u8; 4] = [0xbf, 0x00, 0x00, 0xff];
const ON_COLOR: [u8; 4] = [0x00, 0xbf, 0x00, 0xff];
const HL_COLOR: [u8; 4] = [0xbf, 0xbf, 0xff, 0xff];

struct DebugCpu {
    cpu: Cpu,
    text_writer: TextWriter,
    counter: i32,
}

impl DebugCpu {
    fn print_mem(&self, frame: &mut [u8], page: i32, pos_x: i32, pos_y: i32) {
        for i in 0..16 {
            let addr = (page + i * 16) as u16;
            let mem_line = format!(
                "${:04x}: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
                page + i * 16,
                self.cpu.read(addr),
                self.cpu.read(addr + 1),
                self.cpu.read(addr + 2),
                self.cpu.read(addr + 3),
                self.cpu.read(addr + 4),
                self.cpu.read(addr + 5),
                self.cpu.read(addr + 6),
                self.cpu.read(addr + 7),
                self.cpu.read(addr + 8),
                self.cpu.read(addr + 9),
                self.cpu.read(addr + 10),
                self.cpu.read(addr + 11),
                self.cpu.read(addr + 12),
                self.cpu.read(addr + 13),
                self.cpu.read(addr + 14),
                self.cpu.read(addr + 15),
            );
            self.text_writer
                .write(frame, pos_x, pos_y + i, &mem_line, &FG_COLOR, &BG_COLOR);
        }
    }

    fn print_reg(&self, frame: &mut [u8], pos_x: i32, pos_y: i32) {
        self.print_status(frame, pos_x, pos_y);
        self.text_writer.write(
            frame,
            pos_x,
            pos_y + 1,
            &format!("PC: ${:04x}", self.cpu.pc),
            &FG_COLOR,
            &BG_COLOR,
        );
        self.text_writer.write(
            frame,
            pos_x,
            pos_y + 2,
            &format!("A:  ${:02x}   [{:3}]", self.cpu.a, self.cpu.a),
            &FG_COLOR,
            &BG_COLOR,
        );
        self.text_writer.write(
            frame,
            pos_x,
            pos_y + 3,
            &format!("X:  ${:02x}   [{:3}]", self.cpu.x, self.cpu.x),
            &FG_COLOR,
            &BG_COLOR,
        );
        self.text_writer.write(
            frame,
            pos_x,
            pos_y + 4,
            &format!("Y:  ${:02x}   [{:3}]", self.cpu.y, self.cpu.y),
            &FG_COLOR,
            &BG_COLOR,
        );
        self.text_writer.write(
            frame,
            pos_x,
            pos_y + 5,
            &format!("SP: ${:04x}", 0x0100 + self.cpu.stkp as u16),
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
            let color = if self.cpu.get_flag(f) {
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

    fn print_disasm(&self, frame: &mut [u8], addr: u16, pos_x: i32, pos_y: i32, range: isize) {
        for (i, offset) in (-range..=range).enumerate() {
            let addr_offset = if offset < 0 {
                (Wrapping(addr) - Wrapping((-offset) as u16)).0
            } else {
                (Wrapping(addr) + Wrapping(offset as u16)).0
            };
            let color = if offset == 0 { &HL_COLOR } else { &FG_COLOR };
            self.text_writer.write(
                frame,
                pos_x,
                pos_y + i as i32,
                &format!("${:04x}: DISASM", addr_offset),
                color,
                &BG_COLOR,
            );
        }
    }
}

impl ScreenBackend for DebugCpu {
    fn init(&self, frame: Frame) {
        for pixel in frame.frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0x00, 0x00, 0x7f, 0xff]);
        }
        self.text_writer.write(
            frame.frame,
            1,
            37,
            "SPACE = step      R = reset      I = irq      N = nmi      ESC = quit",
            &FG_COLOR,
            &BG_COLOR,
        );
    }

    fn draw(&self, frame: Frame) {
        self.print_mem(frame.frame, 0x0000, 1, 1);
        self.print_mem(frame.frame, 0x8000, 1, 19);

        self.print_reg(frame.frame, 57, 1);
        self.print_disasm(frame.frame, 0x8000, 57, 8, 13);
    }

    fn update(&mut self) {}

    fn handle_input(&mut self, input: &WinitInputHelper) {
        if input.key_pressed(VirtualKeyCode::Left) {
            self.counter -= 1;
        } else if input.key_pressed(VirtualKeyCode::Right) {
            self.counter += 1;
        }
    }
}

impl DebugCpu {
    fn new() -> DebugCpu {
        DebugCpu {
            cpu: Cpu::new(Box::new(DummyBus::new())),
            text_writer: TextWriter::new(
                "res/cozette.bdf",
                TextScreenParams {
                    width: SCREEN_WIDTH as usize,
                    height: SCREEN_HEIGHT as usize,
                },
            ),
            counter: 0,
        }
    }
}

fn main() {
    let screen = Screen::new(ScreenParams {
        width: SCREEN_WIDTH,
        height: SCREEN_HEIGHT,
        title: "debug_cpu",
        backend: Box::new(DebugCpu::new()),
    })
    .unwrap();

    screen.run();
}
