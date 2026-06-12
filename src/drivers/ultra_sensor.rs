use embassy_rp::{
    Peri,
    gpio::{Input, Level, Output, Pin, Pull},
};
use embassy_time::{Duration, Instant, Timer, with_timeout};

const MIN_DIST: u16 = 2;
const MAX_DIST: u16 = 400;

pub struct UltraSensor<'a> {
    pub trig: Output<'a>,
    pub echo: Input<'a>,
}

impl<'a> UltraSensor<'a> {
    pub fn new(trig_pin: Peri<'a, impl Pin>, echo_pin: Peri<'a, impl Pin>) -> Self {
        Self {
            trig: Output::new(trig_pin, Level::Low),
            echo: Input::new(echo_pin, Pull::None),
        }
    }

    // wait for at least 60ms between function calls for accurate measurements
    pub async fn measure(&mut self) -> Option<u16> {
        self.trig.set_high();
        Timer::after_micros(10).await;
        self.trig.set_low();

        // hardware not responding, probably not powered on
        if with_timeout(Duration::from_millis(5), self.echo.wait_for_high())
            .await
            .is_err()
        {
            return None;
        }

        let rise = Instant::now();
        self.echo.wait_for_low().await;
        let fall = Instant::now();

        let dist = ((fall - rise).as_micros() * 343 / 10000 / 2) as u16;

        if MIN_DIST <= dist && dist <= MAX_DIST {
            Some(dist)
        } else {
            None
        }
    }
}
