#![no_std]
#![cfg_attr(not(feature = "host"), no_main)]

use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_time::Timer;
use embedded_hal::digital::OutputPin;
use linkme::distributed_slice;
use log::{info, warn};
use panic_probe as _;

pub mod boards;
mod cli;
pub mod console;
use crate::boards::hal::*;
pub mod events;
mod serial_logger;
use si473x::Si47xxDevice;
mod settings;
use settings::{OPTIONS, Settings, option::OptionString};
mod storage;

#[distributed_slice(OPTIONS)]
pub static CONFIG_RADIO_MODE: OptionString<64> = OptionString::new("radio_mode", "AM");

#[embassy_executor::main]
async fn run(spawner: Spawner) {
    let mut led = hal_led_create();

    let (tx, rx) = hal_uart_create();
    console::stdout_init(tx);
    serial_logger::init().unwrap();
    Settings::init(0, 32 * 4096).await.unwrap();
    CONFIG_RADIO_MODE.set("FM").await;
    Settings::save().await.unwrap();
    let radio_mode = CONFIG_RADIO_MODE.get().await;
    info!("CONFIG_RADIO_MODE: {}", radio_mode);
    CONFIG_RADIO_MODE.set("SBB").await;
    Settings::load().await.unwrap();
    let radio_mode = CONFIG_RADIO_MODE.get().await;
    info!("CONFIG_RADIO_MODE: {}", radio_mode);

    let twi = hal_twi_create();
    let reset_pin = hal_radio_reset_create();
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
        let _ = OutputPin::set_high(&mut led);
        Timer::after_millis(300).await;
        let _ = OutputPin::set_low(&mut led);
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
