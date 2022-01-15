pub trait Channel {
    fn clock(&mut self);
    fn clock_quarter_frame(&mut self);
    fn clock_half_frame(&mut self);
}
