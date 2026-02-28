use core::sync::atomic::AtomicI32;

use embassy_executor::Spawner;
use embassy_rp::{
    Peripherals,
    gpio::{Input, Pull},
    pwm::{self, Pwm},
};

use crate::{
    drivers::{buzzer::Buzzer, h_bridge::HBridge, rgb_led::RGBLed, speed_control::SpeedControl},
    usb,
    utils::atomic_f32::AtomicF32,
};

/// wrapper for all external peripherals
#[allow(dead_code)]
pub struct Hardware<'a> {
    pub sc: SpeedControl,
    pub buzzer: Buzzer<'a>,
    pub track_left: Input<'a>,
    pub track_right: Input<'a>,
    pub rgb_led: RGBLed<'a>,
}

impl<'a> Hardware<'a> {
    /// initialize all hardware from the given peripherals singleton
    pub fn init(p: Peripherals, spawner: Spawner) -> Self {
        usb::logger_init(p.USB, &spawner);

        let hb = HBridge::new(
            Pwm::new_output_ab(p.PWM_SLICE6, p.PIN_12, p.PIN_13, pwm::Config::default()),
            Pwm::new_output_ab(p.PWM_SLICE5, p.PIN_10, p.PIN_11, pwm::Config::default()),
            2000,
        );

        let sc = {
            static LEFT_TICKS: AtomicI32 = AtomicI32::new(0);
            static RIGHT_TICKS: AtomicI32 = AtomicI32::new(0);
            static LEFT_RPM_SP: AtomicF32 = AtomicF32::new(0.0);
            static RIGHT_RPM_SP: AtomicF32 = AtomicF32::new(0.0);
            SpeedControl::new(
                hb,
                p.PIO0,
                p.PIN_6,
                p.PIN_7,
                p.PIN_8,
                p.PIN_9,
                &LEFT_TICKS,
                &RIGHT_TICKS,
                &LEFT_RPM_SP,
                &RIGHT_RPM_SP,
                &spawner,
            )
        };

        let buzzer = Buzzer::new(Pwm::new_output_b(
            p.PWM_SLICE0,
            p.PIN_17,
            pwm::Config::default(),
        ));

        let track_left = Input::new(p.PIN_28, Pull::Up);
        let track_right = Input::new(p.PIN_16, Pull::Up);

        let rgb_led = RGBLed::new(
            Pwm::new_output_ab(p.PWM_SLICE1, p.PIN_18, p.PIN_19, pwm::Config::default()),
            Pwm::new_output_a(p.PWM_SLICE2, p.PIN_20, pwm::Config::default()),
            2000,
        );

        Self {
            sc,
            buzzer,
            track_left,
            track_right,
            rgb_led,
        }
    }
}
