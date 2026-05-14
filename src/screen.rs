pub mod backend;
pub mod textwriter;

use backend::{Frame, ScreenBackend};
use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::{Fullscreen, Window};
use winit_input_helper::WinitInputHelper;

pub struct Screen<'a> {
    params: ScreenParams<'a>,
    window: Option<Arc<Window>>,
    input: WinitInputHelper,
    pixels: Option<Pixels<'static>>,
    time: Instant,
    fullscreen: bool,
}

pub struct ScreenParams<'a> {
    pub width: u32,
    pub height: u32,
    pub title: &'a str,
    pub backend: Box<dyn ScreenBackend>,
}

impl<'a> Screen<'a> {
    pub fn new(params: ScreenParams<'a>, fullscreen: bool) -> Screen<'a> {
        Self {
            params,
            window: None,
            input: WinitInputHelper::new(),
            pixels: None,
            time: Instant::now(),
            fullscreen,
        }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        let _ = event_loop.run_app(&mut self);
    }
}

impl ApplicationHandler for Screen<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let size = LogicalSize::new(self.params.width as f64, self.params.height as f64);
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(self.params.title)
                        .with_inner_size(size)
                        .with_min_inner_size(size),
                )
                .unwrap(),
        );

        self.window = Some(window.clone());
        self.pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            match Pixels::new(self.params.width, self.params.height, surface_texture) {
                Ok(pixels) => Some(pixels),
                Err(e) => {
                    eprintln!("Create pixels error: {e}");
                    event_loop.exit();
                    None
                }
            }
        };

        self.params.backend.init(Frame {
            frame: self.pixels.as_mut().unwrap().frame_mut(),
            width: self.params.width,
            height: self.params.height,
        });

        let fullscreen_cfg = Some(Fullscreen::Borderless(event_loop.primary_monitor()));
        match self.fullscreen {
            true => {
                window.set_fullscreen(fullscreen_cfg.clone());
                window.set_cursor_visible(false);
            }
            false => {
                window.set_fullscreen(None);
                window.set_cursor_visible(true);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if self.input.process_window_event(&event) {
            // Resize the window
            if let Some(size) = self.input.window_resized() {
                let pixels = self.pixels.as_mut().unwrap();
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    eprintln!("Pixels resize_surface: {err}");
                    self.params.backend.shutdown(false);
                    event_loop.exit();
                    return;
                }
                self.params.backend.init(Frame {
                    frame: pixels.frame_mut(),
                    width: self.params.width,
                    height: self.params.height,
                });
            }

            self.params.backend.draw(Frame {
                frame: self.pixels.as_mut().unwrap().frame_mut(),
                width: self.params.width,
                height: self.params.height,
            });
            if self.pixels.as_ref().unwrap().render().is_err() {
                self.params.backend.shutdown(false);
                event_loop.exit();
            }
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        self.input.process_device_event(&event);
    }

    fn new_events(&mut self, _: &ActiveEventLoop, _: StartCause) {
        self.input.step();
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.input.end_step();

        // Close events
        if self.input.key_pressed(KeyCode::Escape)
            || self.input.close_requested()
            || self.input.destroyed()
        {
            self.params.backend.shutdown(true);
            event_loop.exit();
            return;
        }

        // Toggle fullscreen
        if self.input.key_pressed(KeyCode::F11) {
            // toggle fullscreen
            self.fullscreen = !self.fullscreen;
            let window = self.window.as_ref().unwrap();
            match self.fullscreen {
                true => {
                    let fullscreen_cfg = Some(Fullscreen::Borderless(event_loop.primary_monitor()));
                    window.set_fullscreen(fullscreen_cfg.clone());
                    window.set_cursor_visible(false);
                }
                false => {
                    window.set_fullscreen(None);
                    window.set_cursor_visible(true);
                }
            }
        }

        // Let other input be handled by backend
        self.params.backend.handle_input(&self.input);

        // Update internal state and request a redraw
        let now = Instant::now();
        let dt = now.duration_since(self.time).as_secs_f64();
        self.time = now;

        self.params.backend.update(
            Frame {
                frame: self.pixels.as_mut().unwrap().frame_mut(),
                width: self.params.width,
                height: self.params.height,
            },
            dt,
        );
        self.window.as_ref().unwrap().request_redraw();
    }
}
