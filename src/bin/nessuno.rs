use clap::Parser;
use crossbeam_channel::{bounded, Sender};
use nessuno::audio;
use nessuno::cartridge::Cartridge;
use nessuno::cpu::Flag;
use nessuno::input::{InputGilrs, InputKeyboard};
use nessuno::ppu::palette::PALETTE_2C02;
use nessuno::ppu::SetPixel;
use nessuno::screen::backend::{Frame, ScreenBackend};
use nessuno::screen::textwriter::{TextScreenParams, TextWriter};
use nessuno::screen::{Screen, ScreenParams};
use nessuno::system::System;
use std::io;
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

const SCREEN_WIDTH: u32 = 960;
const SCREEN_HEIGHT: u32 = 540;

const SCREEN_WIDTH_MIN: u32 = 256;
const SCREEN_HEIGHT_MIN: u32 = 240;

const FG_COLOR: [u8; 4] = [0xff, 0xff, 0xff, 0xff];
const BG_COLOR: [u8; 4] = [0x00, 0x00, 0x7f, 0xff];
const OFF_COLOR: [u8; 4] = [0xbf, 0x00, 0x00, 0xff];
const ON_COLOR: [u8; 4] = [0x00, 0xbf, 0x00, 0xff];
const HL_COLOR: [u8; 4] = [0xbf, 0xbf, 0xff, 0xff];

const FRAME_DURATION: f64 = 1f64 / 60f64;

const AUDIO_BUFFER_SIZE: usize = (crate::audio::BUFFER_SIZE as usize) * 2;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    rom_file: String,
    #[clap(short, long)]
    debug: bool,
    #[clap(short, long)]
    fullscreen: bool,
}

struct VideoRenderParams {
    offset_x: usize,
    offset_y: usize,
    width_y: usize,
    scaling_factor: usize,
    bytes_per_pixel: usize,
}

enum UserAction {
    Reset,
    Step,
    Frame,
    PaletteSelect,
}

struct Nessuno {
    system: System,
    text_writer: TextWriter,
    render_params: VideoRenderParams,

    audio_send: Sender<f32>,

    input_gilrs: InputGilrs,
    input_keyboard: InputKeyboard,

    run: bool,
    t_residual: f64,
    action: Option<UserAction>,
    display_oam: bool,
    palette_selected: usize,
    paint: bool,
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

    fn print_oam(&self, frame: &mut [u8], pos_x: i32, pos_y: i32, num_entries: usize) {
        for n in 0..num_entries {
            self.text_writer.write(
                frame,
                pos_x,
                pos_y + n as i32,
                &format!("{:30}", &self.system.ppu_debug_oam(n)),
                &FG_COLOR,
                &BG_COLOR,
            );
        }
    }

    fn print_disasm(&self, frame: &mut [u8], addr: u16, pos_x: i32, pos_y: i32, range: usize) {
        let addr_start = (addr as i32 - (range as i32 * 3)).max(0) as u16;
        let addr_end = (addr as i32 + (range as i32 * 3)).min(0xffff) as u16;
        let disasm = self.system.cpu_disassemble(addr_start, addr_end);

        // current position
        self.text_writer.write(
            frame,
            pos_x,
            pos_y + range as i32,
            &format!("{:30}", &disasm.get(&addr).unwrap_or(&String::from(""))),
            &HL_COLOR,
            &BG_COLOR,
        );

        // forward
        let mut it_forward = disasm.range(addr..).skip(1);
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
        let mut it_backward = disasm.range(..addr);
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
                    &PALETTE_2C02[color],
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
        color: &[u8; 4],
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
            let offset_frame_y = ((pos_y + offset_y) * SCREEN_WIDTH as usize + pos_x) * 4;
            for (offset_x, color_idx) in line.iter().enumerate() {
                let offset_frame_x = offset_frame_y + offset_x * 4;
                frame[offset_frame_x..offset_frame_x + 4]
                    .copy_from_slice(&PALETTE_2C02[*color_idx]);
            }
        }
    }

    fn new(cart: Cartridge, audio_send: Sender<f32>) -> Nessuno {
        Nessuno {
            system: System::new(cart, 44100),
            text_writer: TextWriter::new(
                "res/cozette.bdf",
                TextScreenParams {
                    width: SCREEN_WIDTH as usize,
                    height: SCREEN_HEIGHT as usize,
                },
            ),
            render_params: VideoRenderParams {
                offset_x: 30,
                offset_y: 30,
                width_y: SCREEN_WIDTH as usize,
                scaling_factor: 2,
                bytes_per_pixel: 4,
            },
            input_gilrs: InputGilrs::new(),
            input_keyboard: InputKeyboard::new(),
            audio_send,
            run: false,
            t_residual: 0f64,
            action: Some(UserAction::Reset),
            display_oam: false,
            palette_selected: 0,
            paint: false,
        }
    }

    pub fn frame(&mut self, frame: &mut [u8], wait_cpu_complete: bool, send_audio: bool) {
        let mut cpu_complete = loop {
            let clock_res = self.system.clock();
            if let Some(p) = clock_res.set_pixel {
                set_video_pixel(&self.render_params, frame, &p);
            }
            if send_audio {
                if let Some(s) = clock_res.audio_sample {
                    self.audio_send.try_send(s).unwrap_or(());
                }
            }
            if clock_res.frame_complete {
                break clock_res.cpu_complete;
            }
        };

        if wait_cpu_complete {
            while !cpu_complete {
                let clock_res = self.system.clock();
                if let Some(p) = clock_res.set_pixel {
                    set_video_pixel(&self.render_params, frame, &p);
                }
                if send_audio {
                    if let Some(s) = clock_res.audio_sample {
                        self.audio_send.try_send(s).unwrap();
                    }
                }
                cpu_complete = clock_res.cpu_complete;
            }
        }
    }

    pub fn run_until_audio(&mut self, frame: &mut [u8]) {
        loop {
            let clock_res = self.system.clock();
            if let Some(p) = clock_res.set_pixel {
                set_video_pixel(&self.render_params, frame, &p);
            }
            if let Some(s) = clock_res.audio_sample {
                self.audio_send.try_send(s).unwrap_or(());
                break;
            }
        }
    }

    pub fn step(&mut self, frame: &mut [u8]) {
        // Run cycles until the current CPU instruction has executed
        loop {
            let clock_res = self.system.clock();
            if let Some(p) = clock_res.set_pixel {
                set_video_pixel(&self.render_params, frame, &p);
            }
            if clock_res.cpu_complete {
                break;
            }
        }

        // Run additional system clock cycles (e.g. PPU) until the next CPU instruction starts
        loop {
            let clock_res = self.system.clock();
            if let Some(p) = clock_res.set_pixel {
                set_video_pixel(&self.render_params, frame, &p);
            }
            if !clock_res.cpu_complete {
                break;
            }
        }
    }
}

impl ScreenBackend for Nessuno {
    fn init(&self, frame: Frame) {
        for pixel in frame.frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&BG_COLOR);
        }
        self.text_writer.write(
            frame.frame,
            5,
            37,
            "SPACE = run/pause      R = reset      S = step      F = frame      T = toggle oam/disasm      F11 = fullscreen      ESC = quit",
            &FG_COLOR,
            &BG_COLOR,
        );

        self.text_writer.write(
            frame.frame,
            112,
            1,
            "Controller 1\nArrow Keys - Joypad\n1 - B\n2 - A\n3 - Select\n4 - Start",
            &FG_COLOR,
            &BG_COLOR,
        );
    }

    fn draw(&self, frame: Frame) {
        self.print_reg(frame.frame, 82, 1);
        if self.display_oam {
            self.print_oam(frame.frame, 82, 8, 15);
        } else {
            self.print_disasm(frame.frame, self.system.cpu.pc, 82, 8, 7);
        }
    }

    fn update(&mut self, frame: Frame, dt: f64) {
        if self.run {
            if let Some((input_c1, input_c2)) = self.input_gilrs.get() {
                self.system.controller_update(&input_c1, &input_c2);
            }

            while self.audio_send.len() < AUDIO_BUFFER_SIZE / 2 {
                self.run_until_audio(frame.frame);
            }

            if self.t_residual > 0f64 {
                self.t_residual -= dt;
            } else {
                self.t_residual += FRAME_DURATION - dt;
                self.frame(frame.frame, false, true);
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
                        self.step(frame.frame);
                    }
                    UserAction::Frame => {
                        self.frame(frame.frame, true, false);
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
        // CONTROLLER INPUT
        if let Some((input_c1, input_c2)) = self.input_keyboard.get(input) {
            self.system.controller_update(&input_c1, &input_c2);
        }

        // DEBUG KEYS
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
        } else if input.key_pressed(VirtualKeyCode::T) {
            self.display_oam = !self.display_oam;
        } else if input.key_pressed(VirtualKeyCode::P) {
            self.palette_selected += 1;
            self.palette_selected &= 0x07;
            self.action = Some(UserAction::PaletteSelect);
        }
    }
}

struct NessunoMin {
    system: System,
    render_params: VideoRenderParams,

    input_gilrs: InputGilrs,
    input_keyboard: InputKeyboard,

    audio_send: Sender<f32>,

    run: bool,
    t_residual: f64,
}

impl NessunoMin {
    fn new(cart: Cartridge, audio_send: Sender<f32>) -> NessunoMin {
        let mut system = System::new(cart, 44100);
        system.reset();
        NessunoMin {
            system,
            render_params: VideoRenderParams {
                offset_x: 0,
                offset_y: 0,
                width_y: SCREEN_WIDTH_MIN as usize,
                scaling_factor: 1,
                bytes_per_pixel: 4,
            },
            input_gilrs: InputGilrs::new(),
            input_keyboard: InputKeyboard::new(),
            audio_send,
            run: true,
            t_residual: 0f64,
        }
    }

    pub fn run_until_audio(&mut self, frame: &mut [u8]) {
        loop {
            let clock_res = self.system.clock();
            if let Some(p) = clock_res.set_pixel {
                set_video_pixel(&self.render_params, frame, &p);
            }
            if let Some(s) = clock_res.audio_sample {
                self.audio_send.try_send(s).unwrap_or(());
                break;
            }
        }
    }

    pub fn frame(&mut self, frame: &mut [u8]) {
        loop {
            let clock_res = self.system.clock();
            if let Some(p) = clock_res.set_pixel {
                set_video_pixel(&self.render_params, frame, &p);
            }
            if let Some(s) = clock_res.audio_sample {
                self.audio_send.try_send(s).unwrap_or(());
            }
            if clock_res.frame_complete {
                break;
            }
        }
    }
}

impl ScreenBackend for NessunoMin {
    fn init(&self, frame: Frame) {
        for pixel in frame.frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&BG_COLOR);
        }
    }

    fn draw(&self, _frame: Frame) {}

    fn update(&mut self, frame: Frame, dt: f64) {
        if self.run {
            if let Some((input_c1, input_c2)) = self.input_gilrs.get() {
                self.system.controller_update(&input_c1, &input_c2);
            }

            while self.audio_send.len() < AUDIO_BUFFER_SIZE / 2 {
                self.run_until_audio(frame.frame);
            }

            if self.t_residual > 0f64 {
                self.t_residual -= dt;
            } else {
                self.t_residual += FRAME_DURATION - dt;
                self.frame(frame.frame);
            }
        }
    }

    fn handle_input(&mut self, input: &WinitInputHelper) {
        // CONTROLLER INPUT
        if let Some((input_c1, input_c2)) = self.input_keyboard.get(input) {
            self.system.controller_update(&input_c1, &input_c2);
        }

        // DEBUG KEYS
        if input.key_pressed(VirtualKeyCode::Space) {
            self.run = !self.run;
            if !self.run {
                self.t_residual = 0f64;
            }
        }
    }
}

fn set_video_pixel(render_params: &VideoRenderParams, frame: &mut [u8], p: &SetPixel) {
    match render_params.scaling_factor {
        1 => {
            let py = render_params.offset_y + p.pos.0;
            let px = render_params.offset_x + p.pos.1;
            let off = (py * render_params.width_y + px) * render_params.bytes_per_pixel;
            frame[off..off + render_params.bytes_per_pixel].copy_from_slice(&PALETTE_2C02[p.color]);
        }
        2 => {
            let py = render_params.offset_y + p.pos.0 * 2;
            let px = render_params.offset_x + p.pos.1 * 2;
            let off0 = (py * render_params.width_y + px) * render_params.bytes_per_pixel;
            let off1 = (py * render_params.width_y + px + 1) * render_params.bytes_per_pixel;
            let off2 = ((py + 1) * render_params.width_y + px) * render_params.bytes_per_pixel;
            let off3 = ((py + 1) * render_params.width_y + px + 1) * render_params.bytes_per_pixel;

            frame[off0..off0 + render_params.bytes_per_pixel]
                .copy_from_slice(&PALETTE_2C02[p.color]);
            frame[off1..off1 + render_params.bytes_per_pixel]
                .copy_from_slice(&PALETTE_2C02[p.color]);
            frame[off2..off2 + render_params.bytes_per_pixel]
                .copy_from_slice(&PALETTE_2C02[p.color]);
            frame[off3..off3 + render_params.bytes_per_pixel]
                .copy_from_slice(&PALETTE_2C02[p.color]);
        }
        _ => unreachable!(),
    }
}

fn main() -> Result<(), io::Error> {
    let args = Args::parse();
    let cart = Cartridge::new(&args.rom_file)?;

    let (audio_send, audio_recv) = bounded(AUDIO_BUFFER_SIZE);

    let screen = if args.debug {
        Screen::new(
            ScreenParams {
                width: SCREEN_WIDTH,
                height: SCREEN_HEIGHT,
                title: "nessuno",
                backend: Box::new(Nessuno::new(cart, audio_send)),
            },
            args.fullscreen,
        )
        .unwrap()
    } else {
        Screen::new(
            ScreenParams {
                width: SCREEN_WIDTH_MIN,
                height: SCREEN_HEIGHT_MIN,
                title: "nessuno",
                backend: Box::new(NessunoMin::new(cart, audio_send)),
            },
            args.fullscreen,
        )
        .unwrap()
    };

    audio::run(audio_recv);

    screen.run();

    Ok(())
}
