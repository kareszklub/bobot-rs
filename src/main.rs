#![no_std]
#![no_main]

use core::future::pending;
use embassy_executor::Spawner;
use embassy_time::{Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

use crate::hardware::Hardware;

mod algo;
mod drivers;
mod hardware;
mod usb;
mod utils;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut hw = Hardware::new(p, spawner).await;

    // Timer::after_secs(3).await;
    // log::info!("Hello, world!");

    loop {
        for d in [-90, 0, 90, 0] {
            hw.servo.deg(d);
            Timer::after_secs(1).await;
        }
    }

    // rainbow_led(2000, &mut hw).await;

    // loop {
    //     let u = hw.ultra.measure().await;
    //     log::info!("{:?}", u);
    //     Timer::after_millis(60).await;
    // }

    // pending::<()>().await;
}
