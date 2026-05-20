use crate::events;
use crate::events::NtfPublisher;
use crate::settings::Settings;
use embassy_futures::yield_now;
use log::{info, warn};
use si473x::{RadioBand, Si47xx};

mod mode;
pub use mode::RadioMode;

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

    pub async fn band(&mut self, band: RadioBand, publisher: &NtfPublisher<'_>) {
        match &self.mode {
            RadioMode::Off => {
                warn!("Cannot set band while radio is off");
            }
            _ => {
                let Err(e) = self.radio.band_set(band).await else {
                    publisher
                        .publish(events::SystemNotify::BandChanged(band))
                        .await;
                    return;
                };
                warn!("Failed to set band, error: {:?}", e);
            }
        }
    }

    pub async fn property_list(&mut self, publisher: &NtfPublisher<'_>) {
        let _ = self
            .radio
            .property_for_each(|id, value| {
                let _ = publisher.try_publish(events::SystemNotify::RadioPropertyInfo(id, value));
            })
            .await
            .inspect_err(|e| warn!("Failed to list properties: {:?}", e));
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
                let volume = self.radio.volume_up().await;
                match volume {
                    Ok(v) => {
                        publisher
                            .publish(events::SystemNotify::VolumeChanged(v))
                            .await;
                    }
                    Err(e) => warn!("Failed to increase volume: {:?}", e),
                }
            }
            events::SystemEvent::RadioVolumeDown => {
                let volume = self.radio.volume_down().await;
                match volume {
                    Ok(v) => {
                        publisher
                            .publish(events::SystemNotify::VolumeChanged(v))
                            .await;
                    }
                    Err(e) => warn!("Failed to decrease volume: {:?}", e),
                }
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
            events::SystemEvent::RadioBand(band) => {
                self.band(band, publisher).await;
            }
            events::SystemEvent::RadioPropertyList => {
                self.property_list(publisher).await;
            }
            _ => {
                info!("Event not handled in main loop: {:?}", event);
            }
        }
    }
}
