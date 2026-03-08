#![no_std]
#![no_main]

use core::{future::pending, sync::atomic::Ordering};
use embassy_executor::Spawner;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use crate::{
    algo::follow_line,
    hardware::Hardware,
    net::{packets::SendPacket, send_packet},
};

mod algo;
mod drivers;
mod hardware;
mod net;
mod usb;
mod utils;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut hw = Hardware::init(p, spawner).await;

    // follow_line(225.0, &mut hw).await;

    //rainbow_led(3000, &mut hw).await;

    log::info!("Hello World!");

    for i in 0.. {
        send_packet(SendPacket::DataRead(i)).await;
        Timer::after_millis(50).await;
    }
    pending::<()>().await;
}
