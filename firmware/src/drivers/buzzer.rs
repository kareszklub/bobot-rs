use embassy_rp::pwm::Pwm;

use crate::drivers::pwm::PWM;

pub struct Buzzer<'a> {
    pub pwm: PWM<'a>,
}

impl<'a> Buzzer<'a> {
    /// uses the B channel of the PWM
    pub fn new(pwm: Pwm<'a>) -> Self {
        Self { pwm: PWM::new(pwm) }
    }

    /// a frequency of 0 turns the duty cycle to 0
    pub fn freq(&mut self, freq: u16) {
        if freq > 0 {
            self.pwm.set_duty_b(0xffff / 2);
            self.pwm.set_freq(freq);
        } else {
            self.pwm.set_duty_b(0);
        }
    }
}
