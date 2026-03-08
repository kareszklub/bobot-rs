use core::sync::atomic::{AtomicBool, Ordering};

use cyw43::JoinOptions;
use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use embassy_executor::Spawner;
use embassy_net::{Config, StackResources, tcp::TcpSocket};
use embassy_rp::{
    Peri, bind_interrupts,
    clocks::RoscRng,
    gpio::{Level, Output},
    peripherals::{DMA_CH0, PIN_23, PIN_24, PIN_25, PIN_29, PIO0},
    pio::{InterruptHandler, Pio},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Duration;
use embedded_io_async::Write;
use static_cell::StaticCell;

use crate::net::{
    config::{WIFI_PASSWORD, WIFI_SSID},
    packets::{RecvPacket, SendPacket},
};

mod config;
pub mod packets;

static TX: Channel<CriticalSectionRawMutex, SendPacket, 32> = Channel::new();
static TCP_UP: AtomicBool = AtomicBool::new(false);

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
pub async fn net_init(
    pin_23: Peri<'static, PIN_23>,
    pin_24: Peri<'static, PIN_24>,
    pin_25: Peri<'static, PIN_25>,
    pin_29: Peri<'static, PIN_29>,
    pio0: Peri<'static, PIO0>,
    dma_ch0: Peri<'static, DMA_CH0>,
    spawner: Spawner,
) {
    let mut rng = RoscRng;

    let fw = include_bytes!("../../bin/43439A0.bin");
    let clm = include_bytes!("../../bin/43439A0_clm.bin");

    let pwr = Output::new(pin_23, Level::Low);
    let cs = Output::new(pin_25, Level::High);
    let mut pio = Pio::new(pio0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        pin_24,
        pin_29,
        dma_ch0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    spawner.spawn(wifi_task(runner)).unwrap();

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::Performance)
        .await;

    let config = Config::dhcpv4(Default::default());

    let seed = rng.next_u64();

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        net_device,
        config,
        RESOURCES.init(StackResources::new()),
        seed,
    );

    spawner.spawn(net_task(runner)).unwrap();

    while let Err(err) = control
        .join(WIFI_SSID, JoinOptions::new(WIFI_PASSWORD.as_bytes()))
        .await
    {
        log::info!("join failed: {:?}", err);
    }

    log::info!("waiting for link...");
    stack.wait_link_up().await;

    log::info!("waiting for DHCP...");
    stack.wait_config_up().await;

    log::info!("Stack is up!");

    if let Some(config) = stack.config_v4() {
        log::info!("Assigned IP: {}", config.address.address());
    } else {
        log::error!("Stack is up, but no IPv4 address was found!");
    }

    static RX_BUF: StaticCell<[u8; 4096]> = StaticCell::new();
    static TX_BUF: StaticCell<[u8; 4096]> = StaticCell::new();
    let rx_buffer = RX_BUF.init([0; 4096]);
    let tx_buffer = TX_BUF.init([0; 4096]);

    loop {
        let mut socket = TcpSocket::new(stack, rx_buffer, tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        log::info!("Listening on TCP:1234...");
        if let Err(e) = socket.accept(1234).await {
            log::warn!("accept error: {:?}", e);
            continue;
        }

        let (mut reader, mut writer) = socket.split();

        TCP_UP.store(true, Ordering::Relaxed);

        let recv_fut = async {
            let mut buf = [0u8; 512];
            loop {
                match reader.read(&mut buf).await {
                    Ok(n) => {
                        if n == 0 {
                            log::warn!("Read EOF: client disconnected.");
                            break;
                        }

                        if let Ok(packet) = postcard::from_bytes::<RecvPacket>(&buf[..n]) {
                            log::info!("Received: {:?}", packet);
                            match packet {
                                RecvPacket::CommandToggleLed(state) => {
                                    control.gpio_set(0, state).await;
                                }
                                RecvPacket::Pong => log::info!("Pico got Pong!"),
                                _ => {}
                            }
                        }
                    }
                    Err(e) => log::error!("Failed to read from stream: {:?}", e),
                }
            }
        };

        let send_fut = async {
            loop {
                let packet = TX.receive().await;
                let mut buf = [0u8; 512];
                if let Ok(encoded) = postcard::to_slice(&packet, &mut buf) {
                    let _ = writer.write_all(encoded).await;
                }
            }
        };

        let _ = embassy_futures::select::select(recv_fut, send_fut).await;

        log::warn!("Client disconnected");
        TCP_UP.store(false, Ordering::Relaxed);
    }
}

pub async fn send_packet(packet: SendPacket) {
    if TCP_UP.load(Ordering::Relaxed) {
        TX.send(packet).await;
    }
}
