pub mod backend;
pub mod textwriter;

use backend::{Frame, ScreenBackend};
use pixels::{Error, Pixels, SurfaceTexture};
use std::sync::Arc;
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::{Fullscreen, Window, WindowBuilder};
use winit_input_helper::WinitInputHelper;

pub struct Screen<'a> {
    params: ScreenParams<'a>,
    window: Arc<Window>,
    input: WinitInputHelper,
    pixels: Pixels<'static>,
    event_loop: EventLoop<()>,
    time: Instant,
    fullscreen: bool,
}

pub struct ScreenParams<'a> {
    pub width: u32,
    pub height: u32,
    pub title: &'a str,
    pub backend: Box<dyn ScreenBackend>,
}

impl Screen<'_> {
    pub fn new(params: ScreenParams, fullscreen: bool) -> Result<Screen, Error> {
        let event_loop = EventLoop::new().unwrap();
        let input = WinitInputHelper::new();
        let window = Arc::new({
            let size = LogicalSize::new(params.width as f64, params.height as f64);
            WindowBuilder::new()
                .with_title(params.title)
                .with_inner_size(size)
                .with_min_inner_size(size)
                .build(&event_loop)
                .unwrap()
        });

        let mut pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            Pixels::new(params.width, params.height, surface_texture)?
        };

        params.backend.init(Frame {
            frame: pixels.frame_mut(),
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
            fullscreen,
        })
    }

    pub fn run(mut self) {
        let fullscreen_cfg = Some(Fullscreen::Borderless(self.event_loop.primary_monitor()));
        match self.fullscreen {
            true => {
                self.window.set_fullscreen(fullscreen_cfg.clone());
                self.window.set_cursor_visible(false);
            }
            false => {
                self.window.set_fullscreen(None);
                self.window.set_cursor_visible(true);
            }
        }

        self.time = Instant::now();
        self.event_loop
            .run(move |event, control_flow| {
                // Draw the current frame
                if let Event::WindowEvent {
                    window_id: _,
                    event: WindowEvent::RedrawRequested,
                } = event
                {
                    self.params.backend.draw(Frame {
                        frame: self.pixels.frame_mut(),
                        width: self.params.width,
                        height: self.params.height,
                    });
                    if self.pixels.render().is_err() {
                        self.params.backend.shutdown(false);
                        control_flow.exit();
                        return;
                    }
                }

                // Handle input events
                if self.input.update(&event) {
                    // Close events
                    if self.input.key_pressed(KeyCode::Escape)
                        || self.input.close_requested()
                        || self.input.destroyed()
                    {
                        self.params.backend.shutdown(true);
                        control_flow.exit();
                        return;
                    }

                    if self.input.key_pressed(KeyCode::F11) {
                        // toggle fullscreen
                        self.fullscreen = !self.fullscreen;
                        match self.fullscreen {
                            true => {
                                self.window.set_fullscreen(fullscreen_cfg.clone());
                                self.window.set_cursor_visible(false);
                            }
                            false => {
                                self.window.set_fullscreen(None);
                                self.window.set_cursor_visible(true);
                            }
                        }
                    }

                    // Let other input be handled by backend
                    self.params.backend.handle_input(&self.input);

                    // Resize the window
                    if let Some(size) = self.input.window_resized() {
                        self.pixels.resize_surface(size.width, size.height).unwrap();
                        self.params.backend.init(Frame {
                            frame: self.pixels.frame_mut(),
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
                            frame: self.pixels.frame_mut(),
                            width: self.params.width,
                            height: self.params.height,
                        },
                        dt,
                    );
                    self.window.request_redraw();
                }
            })
            .unwrap();
    }
}
