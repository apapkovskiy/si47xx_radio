#![no_std]
#![cfg_attr(not(feature = "host"), no_main)]

use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_time::Timer;
use embedded_hal::digital::OutputPin;
use log::info;
use panic_probe as _;

pub mod boards;
mod cli;
pub mod console;
use crate::boards::hal::*;
pub mod events;
mod serial_logger;
use si473x::{Si47xxDevice, Si47xxRadio};
mod settings;
use settings::Settings;
mod radio;
use radio::Radio;
mod storage;

#[embassy_executor::main]
async fn run(spawner: Spawner) {
    let mut led = hal_led_create();
    let (tx, rx) = hal_uart_create();
    console::stdout_init(tx);
    serial_logger::init().unwrap();
    Settings::init(0, 32 * 4096).await.unwrap();
    Settings::load().await.expect("Failed to load settings");
    let twi = hal_twi_create();
    let reset_pin = hal_radio_reset_create();
    let radio_dev: Si47xxDevice<_, _> = Si47xxDevice::new(twi, reset_pin);
    let _ = spawner.spawn(cli::my_task(rx));
    yield_now().await;
    let notification_publisher = events::notify_publisher().unwrap();
    let mut radio = Radio::new(Si47xxRadio::Off(radio_dev));
    radio = radio.init(&notification_publisher).await.unwrap();

    loop {
        let _ = OutputPin::set_high(&mut led);
        Timer::after_millis(300).await;
        let _ = OutputPin::set_low(&mut led);
        Timer::after_millis(300).await;
        let event = events::event_receive().await;
        info!("Received event: {:?}", event);
        match event {
            events::SystemEvent::RadioFmOn => {
                radio = radio.fm().await.unwrap();
            }
            events::SystemEvent::RadioAmOn => {
                radio = radio.am().await.unwrap();
            }
            events::SystemEvent::RadioOff => {
                radio = radio.off().await.unwrap();
            }
            _ => {
                radio.handle_event(event, &notification_publisher).await;
            }
        }
    }
}
