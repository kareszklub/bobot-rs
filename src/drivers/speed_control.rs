use core::sync::atomic::{AtomicI32, Ordering};

use embassy_executor::Spawner;
use embassy_rp::{
    Peri,
    gpio::{Input, Pin, Pull},
};
use embassy_time::{Duration, Ticker};

use crate::{
    drivers::h_bridge::HBridge,
    utils::{atomic_f32::AtomicF32, pid::PID},
};

const TICKS_PER_REV: f32 = 1440.0; // 360 * 4
const CONTROL_HZ: f32 = 50.0;
const LPF_ALPHA: f32 = 0.2;

#[embassy_executor::task]
async fn encoder_task(
    l_a: Input<'static>,
    l_b: Input<'static>,
    r_a: Input<'static>,
    r_b: Input<'static>,
    left_ticks: &'static AtomicI32,
    right_ticks: &'static AtomicI32,
) {
    // 20us = 50kHz
    let mut ticker = Ticker::every(Duration::from_micros(20));

    let mut l_state: u8 = 0;
    let mut r_state: u8 = 0;

    loop {
        let l_now = (l_a.is_high() as u8) << 1 | (l_b.is_high() as u8);
        let r_now = (r_a.is_high() as u8) << 1 | (r_b.is_high() as u8);

        // valid Gray code transitions: 00->01, 01->11, 11->10, 10->00
        match (l_state, l_now) {
            (0b00, 0b01) | (0b01, 0b11) | (0b11, 0b10) | (0b10, 0b00) => {
                left_ticks.fetch_add(1, Ordering::Relaxed);
            }
            (0b00, 0b10) | (0b10, 0b11) | (0b11, 0b01) | (0b01, 0b00) => {
                left_ticks.fetch_sub(1, Ordering::Relaxed);
            }
            _ => {}
        }
        l_state = l_now;

        match (r_state, r_now) {
            (0b00, 0b01) | (0b01, 0b11) | (0b11, 0b10) | (0b10, 0b00) => {
                right_ticks.fetch_add(1, Ordering::Relaxed);
            }
            (0b00, 0b10) | (0b10, 0b11) | (0b11, 0b01) | (0b01, 0b00) => {
                right_ticks.fetch_sub(1, Ordering::Relaxed);
            }
            _ => {}
        }
        r_state = r_now;

        ticker.next().await;
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

    for i in 0.. {
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

        // if i % 10 == 0 {
        //     log::info!(
        //         "{} {} {}",
        //         left_rpm_filtered,
        //         right_rpm_filtered,
        //         left_pid.sp
        //     );
        // }

        let left_effort = left_pid.step(left_rpm_filtered);
        let right_effort = right_pid.step(right_rpm_filtered);

        let left_duty = (left_effort * (MAX_DUTY / MAX_EFFORT)).clamp(-MAX_DUTY, MAX_DUTY) as i32;
        let right_duty = (right_effort * (MAX_DUTY / MAX_EFFORT)).clamp(-MAX_DUTY, MAX_DUTY) as i32;

        hb.drive(
            if left_pid.sp == 0.0 { 0 } else { left_duty },
            if right_pid.sp == 0.0 { 0 } else { right_duty },
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
        la_pin: Peri<'static, impl Pin>,
        lb_pin: Peri<'static, impl Pin>,
        ra_pin: Peri<'static, impl Pin>,
        rb_pin: Peri<'static, impl Pin>,
        left_ticks: &'static AtomicI32,
        right_ticks: &'static AtomicI32,
        left_rpm_sp: &'static AtomicF32,
        right_rpm_sp: &'static AtomicF32,
        spawner: &Spawner,
    ) -> Self {
        let l_a = Input::new(la_pin, Pull::Up);
        let l_b = Input::new(lb_pin, Pull::Up);
        let r_a = Input::new(ra_pin, Pull::Up);
        let r_b = Input::new(rb_pin, Pull::Up);

        spawner
            .spawn(encoder_task(l_a, l_b, r_a, r_b, left_ticks, right_ticks))
            .unwrap();
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
