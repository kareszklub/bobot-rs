use core::sync::atomic::{AtomicI32, Ordering};

use embassy_executor::Spawner;
use embassy_rp::Peri;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO1;
use embassy_rp::pio::{InterruptHandler, Pio, PioPin};
use embassy_rp::pio_programs::rotary_encoder::{Direction, PioEncoder, PioEncoderProgram};
use embassy_time::{Duration, Ticker};

use crate::{
    drivers::h_bridge::HBridge,
    utils::{atomic_f32::AtomicF32, pid::PID},
};

bind_interrupts!(struct Irqs {
    PIO1_IRQ_0 => InterruptHandler<PIO1>;
});

const TICKS_PER_REV: f32 = 1440.0;
const CONTROL_HZ: f32 = 50.0;
const LPF_ALPHA: f32 = 0.2;
const HARD_STOP_RPM: f32 = 0.05;

#[embassy_executor::task]
async fn encoder_left(mut encoder: PioEncoder<'static, PIO1, 0>, ticks: &'static AtomicI32) {
    loop {
        let diff = match encoder.read().await {
            Direction::Clockwise => 1,
            Direction::CounterClockwise => -1,
        };
        ticks.fetch_add(diff, Ordering::Relaxed);
    }
}

#[embassy_executor::task]
async fn encoder_right(mut encoder: PioEncoder<'static, PIO1, 1>, ticks: &'static AtomicI32) {
    loop {
        let diff = match encoder.read().await {
            Direction::Clockwise => 1,
            Direction::CounterClockwise => -1,
        };
        ticks.fetch_add(diff, Ordering::Relaxed);
    }
}

#[embassy_executor::task]
async fn control_task(
    mut hb: HBridge<'static>,
    left_ticks: &'static AtomicI32,
    right_ticks: &'static AtomicI32,
    left_rpm_sp: &'static AtomicF32,
    right_rpm_sp: &'static AtomicF32,
) {
    let mut ticker = Ticker::every(Duration::from_hz(CONTROL_HZ as u64));

    let mut last_left = 0;
    let mut last_right = 0;

    let mut left_rpm_filtered: f32 = 0.0;
    let mut right_rpm_filtered: f32 = 0.0;

    const MAX_DUTY: f32 = 0xffff as f32;
    const MAX_EFFORT: f32 = 100.0;

    let mut left_pid = PID::new(1.0, 0.4, 0.0, -100.0, 100.0, 0.0);
    let mut right_pid = PID::new(1.0, 0.4, 0.0, -100.0, 100.0, 0.0);

    loop {
        ticker.next().await;

        left_pid.sp = left_rpm_sp.load(Ordering::Relaxed);
        right_pid.sp = right_rpm_sp.load(Ordering::Relaxed);

        let curr_left = left_ticks.load(Ordering::Relaxed);
        let curr_right = right_ticks.load(Ordering::Relaxed);

        let delta_l = curr_left - last_left;
        let delta_r = curr_right - last_right;

        last_left = curr_left;
        last_right = curr_right;

        let left_rpm = (delta_l as f32 * CONTROL_HZ * 60.0) / TICKS_PER_REV;
        let right_rpm = (delta_r as f32 * CONTROL_HZ * 60.0) / TICKS_PER_REV;

        // apply the Low-Pass Filter
        left_rpm_filtered = (LPF_ALPHA * left_rpm) + ((1.0 - LPF_ALPHA) * left_rpm_filtered);
        right_rpm_filtered = (LPF_ALPHA * right_rpm) + ((1.0 - LPF_ALPHA) * right_rpm_filtered);

        let left_effort = left_pid.step(left_rpm_filtered);
        let right_effort = right_pid.step(right_rpm_filtered);

        let left_duty = (left_effort * (MAX_DUTY / MAX_EFFORT)).clamp(-MAX_DUTY, MAX_DUTY) as i32;
        let right_duty = (right_effort * (MAX_DUTY / MAX_EFFORT)).clamp(-MAX_DUTY, MAX_DUTY) as i32;

        hb.drive(
            if left_pid.sp.abs() <= HARD_STOP_RPM {
                0
            } else {
                left_duty
            },
            if right_pid.sp.abs() <= HARD_STOP_RPM {
                0
            } else {
                right_duty
            },
        );
    }
}

pub struct SpeedControl {
    left_rpm_sp: &'static AtomicF32,
    right_rpm_sp: &'static AtomicF32,
}

impl SpeedControl {
    pub fn new(
        hb: HBridge<'static>,
        pio: Peri<'static, PIO1>,
        la_pin: Peri<'static, impl PioPin>,
        lb_pin: Peri<'static, impl PioPin>,
        ra_pin: Peri<'static, impl PioPin>,
        rb_pin: Peri<'static, impl PioPin>,
        left_ticks: &'static AtomicI32,
        right_ticks: &'static AtomicI32,
        left_rpm_sp: &'static AtomicF32,
        right_rpm_sp: &'static AtomicF32,
        spawner: &Spawner,
    ) -> Self {
        let Pio {
            mut common,
            sm0,
            sm1,
            ..
        } = Pio::new(pio, Irqs);

        let prg = PioEncoderProgram::new(&mut common);
        let left = PioEncoder::new(&mut common, sm0, la_pin, lb_pin, &prg);
        let right = PioEncoder::new(&mut common, sm1, ra_pin, rb_pin, &prg);

        spawner.spawn(encoder_left(left, left_ticks)).unwrap();
        spawner.spawn(encoder_right(right, right_ticks)).unwrap();

        spawner
            .spawn(control_task(
                hb,
                left_ticks,
                right_ticks,
                left_rpm_sp,
                right_rpm_sp,
            ))
            .unwrap();

        Self {
            left_rpm_sp,
            right_rpm_sp,
        }
    }

    pub fn drive(&mut self, left_rpm: f32, right_rpm: f32) {
        self.left_rpm_sp.store(left_rpm, Ordering::Relaxed);
        self.right_rpm_sp.store(right_rpm, Ordering::Relaxed);
    }
}
