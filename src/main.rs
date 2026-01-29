#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::peripherals::{SERIAL0, SERIAL1};
use embassy_nrf::{bind_interrupts, uarte};
use embassy_time::Timer;
use log::{info, warn};
use panic_probe as _;

use embassy_nrf::twim::{self, Twim};
use static_cell::ConstStaticCell;

mod cli;
pub mod console;
pub mod events;
mod serial_logger;
use si473x::Si47xxDevice;

bind_interrupts!(struct Irqs {
    SERIAL0 => uarte::InterruptHandler<SERIAL0>;
    SERIAL1 => twim::InterruptHandler<SERIAL1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut led = Output::new(p.P0_28, Level::Low, OutputDrive::Standard);

    let mut config = uarte::Config::default();
    config.parity = uarte::Parity::EXCLUDED;
    config.baudrate = uarte::Baudrate::BAUD115200;
    let uart: uarte::Uarte<'static> = uarte::Uarte::new(p.SERIAL0, p.P0_22, p.P0_20, Irqs, config);
    let (tx, rx) = uart.split();
    console::stdout_init(tx);
    serial_logger::init().unwrap();

    let config = twim::Config::default();
    static RAM_BUFFER: ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);
    let twi = Twim::new(p.SERIAL1, Irqs, p.P1_14, p.P1_13, config, RAM_BUFFER.take());
    let reset_pin = Output::new(p.P1_03, Level::High, OutputDrive::Standard);
    let mut radio_dev: Si47xxDevice<_, _> = Si47xxDevice::new(twi, reset_pin);
    radio_dev.reset().await;
    radio_dev.init_fm().await.expect("Radio init failed");
    warn!("Radio initialized!");
    let revision = radio_dev
        .revision_get()
        .await
        .expect("Failed to get revision");
    radio_dev.sound_on().await.expect("Failed to unmute sound");

    let _ = spawner.spawn(cli::my_task(rx));
    yield_now().await;

    let mut radio = radio_dev.fm().await.expect("Failed to switch to FM mode");
    let notification_publisher = events::notify_publisher().unwrap();
    notification_publisher
        .publish(events::SystemNotify::RadioFmOn)
        .await;
    yield_now().await;
    notification_publisher
        .publish(events::SystemNotify::RevisionInfo(revision))
        .await;
    yield_now().await;
    let tune_status = radio
        .tune_status_get()
        .await
        .expect("Failed to get tune status");
    notification_publisher
        .publish(events::SystemNotify::TuneStatus(tune_status))
        .await;

    loop {
        led.set_high();
        Timer::after_millis(300).await;
        led.set_low();
        Timer::after_millis(300).await;
        let event = events::event_receive().await;
        info!("Received event: {:?}", event);
        match event {
            events::SystemEvent::RadioVolumeUp => {
                radio.volume_up().await.expect("Volume up failed");
            }
            events::SystemEvent::RadioVolumeDown => {
                radio.volume_down().await.expect("Volume down failed");
            }
            events::SystemEvent::RadioSetFrequency(freq) => {
                let tune_status = radio
                    .tune_frequency(freq)
                    .await
                    .expect("Set frequency failed");
                notification_publisher
                    .publish(events::SystemNotify::TuneStatus(tune_status))
                    .await;
            }
            events::SystemEvent::RadioSeekUp => {
                let tune_status = radio.seek_up().await.expect("Seek up failed");
                info!("Seeked up: {:?}", tune_status);
                notification_publisher
                    .publish(events::SystemNotify::TuneStatus(tune_status))
                    .await;
            }
            _ => {
                info!("Event not handled in main loop");
            }
        }
    }
}
