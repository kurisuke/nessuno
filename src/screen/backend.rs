use winit_input_helper::WinitInputHelper;

pub struct Frame<'a> {
    pub frame: &'a mut [u8],
    pub width: u32,
    pub height: u32,
}

pub trait ScreenBackend {
    fn init(&self, frame: Frame);
    fn draw(&self, frame: Frame);
    fn update(&mut self, frame: Frame, dt: f64);
    fn handle_input(&mut self, input: &WinitInputHelper);
}
