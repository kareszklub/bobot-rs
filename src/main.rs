#![no_std]
#![no_main]

use core::future::pending;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use crate::{algo::follow_line, hardware::Hardware};

mod algo;
mod drivers;
mod hardware;
mod usb;
mod utils;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut hw = Hardware::init(p, spawner);

    follow_line(225.0, &mut hw).await;

    //rainbow_led(3000, &mut hw).await;

    pending::<()>().await;
}
