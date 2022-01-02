pub mod backend;
pub mod textwriter;

use backend::{Frame, ScreenBackend};
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use std::iter::Inspect;
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use winit_input_helper::WinitInputHelper;

pub struct Screen<'a> {
    params: ScreenParams<'a>,
    window: Window,
    input: WinitInputHelper,
    pixels: Pixels,
    event_loop: EventLoop<()>,
    time: Instant,
}

pub struct ScreenParams<'a> {
    pub width: u32,
    pub height: u32,
    pub title: &'a str,
    pub backend: Box<dyn ScreenBackend>,
}

impl<'a> Screen<'a> {
    pub fn new(params: ScreenParams) -> Result<Screen, Error> {
        let event_loop = EventLoop::new();
        let input = WinitInputHelper::new();
        let window = {
            let size = LogicalSize::new(params.width as f64, params.height as f64);
            WindowBuilder::new()
                .with_title(params.title)
                .with_inner_size(size)
                .with_min_inner_size(size)
                .build(&event_loop)
                .unwrap()
        };

        let mut pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            Pixels::new(params.width, params.height, surface_texture)?
        };

        params.backend.init(Frame {
            frame: pixels.get_frame(),
            width: params.width,
            height: params.height,
        });

        Ok(Screen {
            params,
            window,
            input,
            pixels,
            event_loop,
            time: Instant::now(),
        })
    }

    pub fn run(mut self) {
        self.time = Instant::now();
        self.event_loop.run(move |event, _, control_flow| {
            // Draw the current frame
            if let Event::RedrawRequested(_) = event {
                self.params.backend.draw(Frame {
                    frame: self.pixels.get_frame(),
                    width: self.params.width,
                    height: self.params.height,
                });
                if self
                    .pixels
                    .render()
                    .map_err(|e| error!("pixels.render() failed: {}", e))
                    .is_err()
                {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            // Handle input events
            if self.input.update(&event) {
                // Close events
                if self.input.key_pressed(VirtualKeyCode::Escape) || self.input.quit() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                //
                self.params.backend.handle_input(&self.input);

                // Resize the window
                if let Some(size) = self.input.window_resized() {
                    self.pixels.resize_surface(size.width, size.height);
                    self.params.backend.init(Frame {
                        frame: self.pixels.get_frame(),
                        width: self.params.width,
                        height: self.params.height,
                    });
                }

                // Update internal state and request a redraw
                let now = Instant::now();
                let dt = now.duration_since(self.time).as_secs_f64();
                self.time = now;

                self.params.backend.update(
                    Frame {
                        frame: self.pixels.get_frame(),
                        width: self.params.width,
                        height: self.params.height,
                    },
                    dt,
                );
                self.window.request_redraw();
            }
        });
    }
}
