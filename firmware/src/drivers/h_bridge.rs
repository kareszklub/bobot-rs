use embassy_rp::pwm::Pwm;

use crate::drivers::pwm::PWM;

pub struct HBridge<'a> {
    l_pwm: PWM<'a>,
    r_pwm: PWM<'a>,
}

impl<'a> HBridge<'a> {
    pub fn new(l_pwm: Pwm<'a>, r_pwm: Pwm<'a>, pwm_freq: u16) -> Self {
        let mut s = Self {
            l_pwm: PWM::new(l_pwm),
            r_pwm: PWM::new(r_pwm),
        };

        s.l_pwm.set_freq(pwm_freq);
        s.r_pwm.set_freq(pwm_freq);
        s
    }

    /// the input speed is clamped to be between -0xffff and 0xffff
    pub fn drive(&mut self, l: i32, r: i32) {
        let l = l.clamp(-0xffff, 0xffff);
        let r = r.clamp(-0xffff, 0xffff);

        self.l_pwm.set_duty_b(if l > 0 { l as u16 } else { 0 });
        self.l_pwm.set_duty_a(if l < 0 { (-l) as u16 } else { 0 });
        self.r_pwm.set_duty_a(if r > 0 { r as u16 } else { 0 });
        self.r_pwm.set_duty_b(if r < 0 { (-r) as u16 } else { 0 });
    }
}
