use nessuno::cpu::{Disassembly, Flag};
use nessuno::screen::backend::{Frame, ScreenBackend};
use nessuno::screen::textwriter::{TextScreenParams, TextWriter};
use nessuno::screen::{Screen, ScreenParams};
use nessuno::system_debug_cpu::SystemDebugCpu;
use winit::keyboard::KeyCode;
use winit_input_helper::WinitInputHelper;

const SCREEN_WIDTH: u32 = 960;
const SCREEN_HEIGHT: u32 = 540;

const FG_COLOR: [u8; 4] = [0xff, 0xff, 0xff, 0xff];
const BG_COLOR: [u8; 4] = [0x00, 0x00, 0x7f, 0xff];
const OFF_COLOR: [u8; 4] = [0xbf, 0x00, 0x00, 0xff];
const ON_COLOR: [u8; 4] = [0x00, 0xbf, 0x00, 0xff];
const HL_COLOR: [u8; 4] = [0xbf, 0xbf, 0xff, 0xff];

struct DebugCpu {
    system: SystemDebugCpu,
    disasm: Disassembly,
    text_writer: TextWriter,

    action: Option<UserAction>,
    paint: bool,
}

enum UserAction {
    Init,
    Step,
    Reset,
    Irq,
    Nmi,
}

impl DebugCpu {
    fn print_mem(&self, frame: &mut [u8], page: i32, pos_x: i32, pos_y: i32) {
        for i in 0..16 {
            let addr = (page + i * 16) as u16;
            let mem_line = format!(
                "${:04x}: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
                page + i * 16,
                self.system.read(addr),
                self.system.read(addr + 1),
                self.system.read(addr + 2),
                self.system.read(addr + 3),
                self.system.read(addr + 4),
                self.system.read(addr + 5),
                self.system.read(addr + 6),
                self.system.read(addr + 7),
                self.system.read(addr + 8),
                self.system.read(addr + 9),
                self.system.read(addr + 10),
                self.system.read(addr + 11),
                self.system.read(addr + 12),
                self.system.read(addr + 13),
                self.system.read(addr + 14),
                self.system.read(addr + 15),
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
        let pc_page = self.system.cpu.pc & 0xf000;
        self.print_mem(frame.frame, pc_page as i32, 1, 19);

        self.print_reg(frame.frame, 57, 1);
        self.print_disasm(frame.frame, self.system.cpu.pc, 57, 8, 13);
    }

    fn update(&mut self, _frame: Frame, _dt: f64) {
        if let Some(action) = &self.action {
            match *action {
                UserAction::Reset => {
                    self.system.cpu_reset();
                }
                UserAction::Irq => {
                    self.system.cpu_irq();
                }
                UserAction::Nmi => {
                    self.system.cpu_nmi();
                }
                UserAction::Step => {
                    self.system.cpu_step();
                }
                _ => {}
            }
            self.action = None;
            self.paint = true;
        } else {
            self.paint = false;
        }
    }

    fn handle_input(&mut self, input: &WinitInputHelper) {
        if input.key_pressed(KeyCode::KeyR) {
            self.action = Some(UserAction::Reset);
        } else if input.key_pressed(KeyCode::KeyI) {
            self.action = Some(UserAction::Irq);
        } else if input.key_pressed(KeyCode::KeyN) {
            self.action = Some(UserAction::Nmi);
        } else if input.key_pressed(KeyCode::Space) {
            self.action = Some(UserAction::Step);
        }
    }
}

impl DebugCpu {
    fn new() -> DebugCpu {
        let mut system = SystemDebugCpu::new();
        system.load_from_str(
            "A2 0A 8E 00 00 A2 03 8E 01 00 AC 00 00 A9 00 18 6D 01 00 88 D0 FA 8D 02 00 EA EA EA",
            0x8000,
        );
        system.set_reset_vector(0x8000);
        let disasm = system.cpu_disassemble(0x0000, 0xffff);

        DebugCpu {
            system,
            disasm,
            text_writer: TextWriter::new(
                include_bytes!("../../res/cozette.bdf"),
                TextScreenParams {
                    width: SCREEN_WIDTH as usize,
                    height: SCREEN_HEIGHT as usize,
                },
            ),
            action: Some(UserAction::Init),
            paint: false,
        }
    }
}

fn main() {
    let screen = Screen::new(
        ScreenParams {
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            title: "debug_cpu",
            backend: Box::new(DebugCpu::new()),
        },
        false,
    )
    .unwrap();

    screen.run();
}
