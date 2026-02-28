use embassy_time::Instant;

pub struct PID {
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,

    int: f32,
    pub int_min: f32,
    pub int_max: f32,

    last_t: Option<Instant>,
    last_e: f32,

    pub sp: f32,
}

impl PID {
    pub fn new(kp: f32, ki: f32, kd: f32, int_min: f32, int_max: f32, sp: f32) -> Self {
        Self {
            kp,
            ki,
            kd,
            int: 0.,
            int_min,
            int_max,
            last_t: None,
            last_e: 0.,
            sp,
        }
    }

    pub fn step(&mut self, pv: f32) -> f32 {
        let now = Instant::now();

        match self.last_t {
            Some(last_t) => {
                let dt = (now - last_t).as_micros() as f32 / 1_000_000.0;
                self.last_t = Some(now);

                let e = self.sp - pv;

                self.int = (self.int + e * dt).clamp(self.int_min, self.int_max);

                let p = self.kp * e;
                let i = self.ki * self.int;
                let d = self.kd * (e - self.last_e) / dt;

                self.last_e = e;

                p + i + d
            }
            None => {
                self.last_t = Some(now);
                self.kp * (self.sp - pv)
            }
        }
    }
}
