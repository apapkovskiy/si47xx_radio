use crate::events;
use crate::events::NtfPublisher;
use core::convert::{From, Into};
use embassy_futures::yield_now;
use linkme::distributed_slice;
use log::{info, warn};
use si473x::Si47xx;

use crate::settings::{OPTIONS, Settings, option::OptionString};

static CONFIG_RADIO_MODE_INST: OptionString<64> = OptionString::new("radio_mode", "AM");
#[distributed_slice(OPTIONS)]
static CONFIG_RADIO_MODE: &'static OptionString<64> = &CONFIG_RADIO_MODE_INST;

#[derive(Debug)]
pub enum RadioMode {
    /// FM Mode
    FM,
    /// AM Mode
    AM,
    /// Power down the radio
    Off,
}

impl RadioMode {
    pub async fn get() -> Self {
        CONFIG_RADIO_MODE.get().await.as_str().into()
    }
    pub async fn save(&self) {
        CONFIG_RADIO_MODE.set(self.into()).await;
    }
}

impl From<&str> for RadioMode {
    fn from(s: &str) -> Self {
        match s {
            "FM" => RadioMode::FM,
            "AM" => RadioMode::AM,
            "Off" => RadioMode::Off,
            _ => RadioMode::Off, // Default to Off for unknown values
        }
    }
}

impl From<&RadioMode> for &'static str {
    fn from(mode: &RadioMode) -> &'static str {
        match mode {
            RadioMode::FM => "FM",
            RadioMode::AM => "AM",
            RadioMode::Off => "Off",
        }
    }
}

pub struct Radio<T>
where
    T: Si47xx,
{
    pub mode: RadioMode,
    pub radio: T,
}

impl<T> Radio<T>
where
    T: Si47xx<Device = T>,
{
    pub fn new(radio: T) -> Self {
        Self {
            mode: RadioMode::Off,
            radio,
        }
    }

    pub async fn am(mut self) -> Result<Self, ()> {
        self.mode = RadioMode::AM;
        self.mode.save().await;
        self.radio = self.radio.am().await.expect("Failed to switch to AM mode");
        Settings::save().await.expect("Failed to save settings");
        Ok(self)
    }

    pub async fn fm(mut self) -> Result<Self, ()> {
        self.mode = RadioMode::FM;
        self.mode.save().await;
        self.radio = self.radio.fm().await.expect("Failed to switch to FM mode");
        Settings::save().await.expect("Failed to save settings");
        Ok(self)
    }

    pub async fn off(mut self) -> Result<Self, ()> {
        self.mode = RadioMode::Off;
        self.mode.save().await;
        Settings::save().await.expect("Failed to save settings");
        Ok(self)
    }

    pub async fn init(mut self, publisher: &NtfPublisher<'_>) -> Result<Self, ()> {
        self.radio = self.radio.reset().await;
        self.mode = RadioMode::get().await;
        warn!("Initializing radio in {:?} mode", self.mode);
        self.radio = match self.mode {
            RadioMode::FM => {
                let radio = self.radio.fm().await.expect("Failed to init to FM mode");
                publisher.publish(events::SystemNotify::RadioFmOn).await;
                yield_now().await;
                radio
            }
            RadioMode::AM => {
                let radio = self.radio.am().await.expect("Failed to init to AM mode");
                publisher.publish(events::SystemNotify::RadioAmOn).await;
                yield_now().await;
                radio
            }
            RadioMode::Off => {
                warn!("Radio initialized in Off mode!");
                return Ok(self); // No initialization needed for Off mode
            }
        };
        let revision = self
            .radio
            .revision_get()
            .await
            .expect("Failed to get revision");
        publisher
            .publish(events::SystemNotify::RevisionInfo(revision))
            .await;
        yield_now().await;
        let tune_status = self
            .radio
            .tune_status_get()
            .await
            .expect("Failed to get tune status");
        publisher
            .publish(events::SystemNotify::TuneStatus(tune_status))
            .await;
        self.radio.sound_on().await.expect("Failed to unmute sound");
        warn!("Radio initialized!");
        Ok(self)
    }

    pub async fn handle_event(&mut self, event: events::SystemEvent, publisher: &NtfPublisher<'_>) {
        match event {
            events::SystemEvent::RadioVolumeUp => {
                self.radio.volume_up().await.expect("Volume up failed");
                publisher
                    .publish(events::SystemNotify::VolumeChanged(0))
                    .await;
            }
            events::SystemEvent::RadioVolumeDown => {
                self.radio.volume_down().await.expect("Volume down failed");
                publisher
                    .publish(events::SystemNotify::VolumeChanged(0))
                    .await;
            }
            events::SystemEvent::RadioSetFrequency(freq) => {
                let tune_status = self
                    .radio
                    .tune_frequency(freq)
                    .await
                    .expect("Set frequency failed");
                publisher
                    .publish(events::SystemNotify::TuneStatus(tune_status))
                    .await;
            }
            events::SystemEvent::RadioSeekUp => {
                let tune_status = self.radio.seek_up().await.expect("Seek up failed");
                info!("Seeked up: {:?}", tune_status);
                publisher
                    .publish(events::SystemNotify::TuneStatus(tune_status))
                    .await;
            }
            _ => {
                info!("Event not handled in main loop");
            }
        }
    }
}
