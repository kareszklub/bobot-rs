use embassy_rp::pwm::Pwm;

use crate::drivers::pwm::PWM;

pub struct Servo<'a> {
    pwm: PWM<'a>,
    min: u16,
    mid: u16,
    max: u16,
}

impl<'a> Servo<'a> {
    /// uses the A channel of the PWM
    pub fn new(pwm: Pwm<'a>, min: u16, mid: u16, max: u16) -> Self {
        let mut pwm = PWM::new(pwm);
        pwm.set_freq(50);

        let mut s = Self { pwm, min, mid, max };
        s.deg(0);
        s
    }

    fn duty(&mut self, d: u16) {
        // flip direction
        let d = 0xffff - d;
        let d = d as u32;

        let duty = if d < 0xffff / 2 {
            self.min as u32 + (self.mid - self.min) as u32 * d / 0xffff * 2
        } else {
            let d = d - 0xffff / 2;
            self.mid as u32 + (self.max - self.mid) as u32 * d / 0xffff * 2
        };

        self.pwm.set_duty_a(duty as u16);
    }

    /// set rotation between -90 and 90 degrees
    pub fn deg(&mut self, d: i8) {
        self.duty(((d as i16 + 90) as u32 * 0xffff / 180) as u16);
    }
}
