use embassy_rp::pwm::Pwm;

use crate::drivers::pwm::PWM;

/// common anode RGB LED
pub struct RGBLed<'a> {
    pub rg_pwm: PWM<'a>,
    pub b_pwm: PWM<'a>,
}

impl<'a> RGBLed<'a> {
    /// the A channel is used for Blue
    pub fn new(rg_pwm: Pwm<'a>, b_pwm: Pwm<'a>, pwm_freq: u16) -> Self {
        let mut s = Self {
            rg_pwm: PWM::new(rg_pwm),
            b_pwm: PWM::new(b_pwm),
        };

        s.rg_pwm.set_freq(pwm_freq);
        s.b_pwm.set_freq(pwm_freq);
        s.set_color(0, 0, 0);
        s
    }

    /// set light intensity between 0 and 255
    pub fn set_color(&mut self, r: u8, g: u8, b: u8) {
        let r = (255 - r) as u16 * ((0xffff / 255) as u16);
        let g = (255 - g) as u16 * ((0xffff / 255) as u16);
        let b = (255 - b) as u16 * ((0xffff / 255) as u16);

        self.rg_pwm.set_duty_a(r);
        self.rg_pwm.set_duty_b(g);
        self.b_pwm.set_duty_a(b);
    }
}
