use embassy_rp::pwm::{Config, Pwm};

/// lightweight PWM convenience wrapper
pub struct PWM<'d> {
    pub pwm: Pwm<'d>,
    pub config: Config,
}

impl<'d> PWM<'d> {
    pub fn new(pwm: Pwm<'d>) -> Self {
        Self {
            pwm,
            config: Config::default(),
        }
    }

    /// set frequency (because of hardware limitations, it must be at least 9)
    pub fn set_freq(&mut self, freq: u16) {
        if freq < 9 {
            return;
        }

        let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
        let div_max = u16::MAX - 1;
        let div = (clock_freq_hz / (div_max as u32 * freq as u32) + 1).min(u8::MAX as u32) as u8;
        let top = (clock_freq_hz / (freq as u32 * div as u32) - 1).min(u16::MAX as u32) as u16;

        self.config.divider = div.into();
        self.config.top = top;

        self.pwm.set_config(&self.config);
    }

    /// set duty cycle for channel A between 0 and 0xffff
    pub fn set_duty_a(&mut self, duty: u16) {
        self.config.compare_a = ((self.config.top) as u32 * duty as u32 / 0xffff) as u16;
        self.pwm.set_config(&self.config);
    }

    /// set duty cycle for channel B between 0 and 0xffff
    pub fn set_duty_b(&mut self, duty: u16) {
        self.config.compare_b = ((self.config.top) as u32 * duty as u32 / 0xffff) as u16;
        self.pwm.set_config(&self.config);
    }
}
