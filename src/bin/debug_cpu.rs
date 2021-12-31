use nessuno::bus::dummy::DummyBus;
use nessuno::cpu::Cpu;
use nessuno::screen::backend::{Frame, ScreenBackend};
use nessuno::screen::textwriter::{TextScreenParams, TextWriter};
use nessuno::screen::{Screen, ScreenParams};
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 480;

const FG_COLOR: [u8; 4] = [0xff, 0xff, 0xff, 0xff];
const BG_COLOR: [u8; 4] = [0x00, 0x00, 0x7f, 0xff];

struct DebugCpu {
    cpu: Cpu,
    text_writer: TextWriter,
    counter: i32,
}

impl DebugCpu {
    fn print_mem(&self, frame: &mut [u8], pos_x: i32, pos_y: i32) {
        for i in 0..16 {
            let mem_line = format!(
                "${:04x}: 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00",
                i
            );
            self.text_writer
                .write(frame, pos_x, pos_y + i, &mem_line, &FG_COLOR, &BG_COLOR);
        }
    }
}

impl ScreenBackend for DebugCpu {
    fn init(&self, frame: Frame) {
        for pixel in frame.frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0x00, 0x00, 0x7f, 0xff]);
        }
    }

    fn draw(&self, frame: Frame) {
        self.text_writer.write(
            frame.frame,
            1,
            1,
            &format!("Hello, World!\nCounter: {:+04}", self.counter),
            &[0xff, 0xff, 0xff, 0xff],
            &[0x00, 0x00, 0x7f, 0xff],
        );

        self.print_mem(frame.frame, 1, 4);
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
        backend: Box::new(DebugCpu::new()),
    })
    .unwrap();

    screen.run();
}
