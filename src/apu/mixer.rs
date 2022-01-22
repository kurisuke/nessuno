const LUT_PULSE_LEN: usize = 31;
const LUT_TND_LEN: usize = 203;

pub struct Mixer {
    lut_pulse: [f32; LUT_PULSE_LEN],
    lut_tnd: [f32; LUT_TND_LEN],
}

impl Mixer {
    pub fn new() -> Mixer {
        let mut lut_pulse = [0f32; LUT_PULSE_LEN];
        for (i, e) in lut_pulse.iter_mut().enumerate() {
            *e = 2.0 * 95.52 / (8128.0 / (i as f32) + 100.0) - 0.5;
        }

        let mut lut_tnd = [0f32; LUT_TND_LEN];
        for (i, e) in lut_tnd.iter_mut().enumerate() {
            *e = 2.0 * 163.67 / (24329.0 / (i as f32) + 100.0) - 0.5;
        }

        Mixer { lut_pulse, lut_tnd }
    }

    pub fn sample(&self, p1: u8, p2: u8, t: u8, n: u8, d: u8) -> f32 {
        let pulse_out = self.lut_pulse[(p1 + p2) as usize];
        let tnd_out = self.lut_tnd[(3 * t + 2 * n + d) as usize];
        pulse_out + tnd_out
    }
}
