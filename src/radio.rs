use crate::events;
use crate::events::NtfPublisher;
use crate::settings::Settings;
use embassy_futures::yield_now;
use log::{error, info, warn};
use si473x::{RadioBand, Si47xx, Si47xxProperty};

mod config;
pub use config::RadioConfig;
mod mode;
pub use mode::RadioMode;

pub struct Radio<T>
where
    T: Si47xx,
{
    pub mode: RadioMode,
    pub radio: T,
    pub config: RadioConfig,
}

impl<T> Radio<T>
where
    T: Si47xx,
{
    pub fn new(radio: T) -> Self {
        Self {
            mode: RadioMode::Off,
            radio,
            config: RadioConfig,
        }
    }

    pub async fn am(&mut self, publisher: &NtfPublisher<'_>) {
        self.mode = RadioMode::AM;
        self.mode.save().await;
        self.radio.am().await.expect("Failed to switch to AM mode");
        Settings::save().await.expect("Failed to save settings");
        publisher.publish(events::SystemNotify::RadioAmOn).await;
        self.band(self.config.config_am_band_get().await, publisher)
            .await;
        self.property_set(
            Si47xxProperty::AmSeekTuneRssiThreshold,
            self.config.config_am_tune_rssi_threshold_get().await,
        )
        .await;
        self.property_set(
            Si47xxProperty::AmSeekTuneSnrThreshold,
            self.config.config_am_tune_snr_threshold_get().await,
        )
        .await;
        let freq = self.config.config_am_freq_get().await;
        self.radio
            .tune_frequency(freq)
            .await
            .inspect_err(|e| warn!("Failed to tune frequency {}: {:?}", freq, e))
            .ok();
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
                warn!("Failed to set band {}: {:?}", band, e);
            }
        }
    }

    pub async fn property_set(&mut self, property: Si47xxProperty, value: u16) {
        let _ = self
            .radio
            .property_set(property, value)
            .await
            .inspect_err(|e| warn!("Failed to set property: {:?}", e));
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

    pub async fn fm(&mut self, publisher: &NtfPublisher<'_>) {
        self.mode = RadioMode::FM;
        self.mode.save().await;
        self.radio.fm().await.expect("Failed to switch to FM mode");
        Settings::save().await.expect("Failed to save settings");
        publisher.publish(events::SystemNotify::RadioFmOn).await;
        self.band(self.config.config_fm_band_get().await, publisher)
            .await;
        self.property_set(
            Si47xxProperty::FmSeekTuneRssiThreshold,
            self.config.config_fm_tune_rssi_threshold_get().await,
        )
        .await;
        self.property_set(
            Si47xxProperty::FmSeekTuneSnrThreshold,
            self.config.config_fm_tune_snr_threshold_get().await,
        )
        .await;
        let freq = self.config.config_fm_freq_get().await;
        self.radio
            .tune_frequency(freq)
            .await
            .inspect_err(|e| warn!("Failed to tune frequency {}: {:?}", freq, e))
            .ok();
    }

    pub async fn off(&mut self, publisher: &NtfPublisher<'_>) {
        self.mode = RadioMode::Off;
        self.radio.power_down().await.expect("Radio off failed");
        self.mode.save().await;
        Settings::save().await.expect("Failed to save settings");
        publisher.publish(events::SystemNotify::RadioOff).await;
    }

    pub async fn init(&mut self, publisher: &NtfPublisher<'_>) {
        self.radio.reset().await;
        self.mode = RadioMode::get().await;
        warn!("Initializing radio in {:?} mode", self.mode);
        match self.mode {
            RadioMode::FM => {
                self.radio.fm().await.expect("Failed to init to FM mode");
                publisher.publish(events::SystemNotify::RadioFmOn).await;
                yield_now().await;
            }
            RadioMode::AM => {
                self.radio.am().await.expect("Failed to init to AM mode");
                publisher.publish(events::SystemNotify::RadioAmOn).await;
                yield_now().await;
            }
            RadioMode::Off => {
                warn!("Radio initialized in Off mode!");
                publisher.publish(events::SystemNotify::RadioOff).await;
                return;
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
    }

    pub async fn handle_event(&mut self, event: events::SystemEvent, publisher: &NtfPublisher<'_>) {
        match event {
            events::SystemEvent::RadioFmOn => {
                self.fm(publisher).await;
            }
            events::SystemEvent::RadioAmOn => {
                self.am(publisher).await;
            }
            events::SystemEvent::RadioOff => {
                self.off(publisher).await;
            }
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
                let ret = self
                    .radio
                    .seek_up(|ts| {
                        publisher
                            .try_publish(events::SystemNotify::TuneStatus(ts))
                            .unwrap_or_default()
                    })
                    .await;
                match ret {
                    Ok(tune_status) => {
                        info!("Seek up: {:?}", tune_status);
                        publisher
                            .publish(events::SystemNotify::TuneStatus(tune_status))
                            .await;
                    }
                    Err(e) => error!("Seek up failed; error: {:?}", e),
                }
            }
            events::SystemEvent::RadioBand(band) => {
                self.band(band, publisher).await;
            }
            events::SystemEvent::RadioPropertyList => {
                self.property_list(publisher).await;
            }
            events::SystemEvent::RadioPropertySet(property, value) => {
                self.property_set(property, value).await;
            }
            _ => {
                info!("Event not handled in main loop: {:?}", event);
            }
        }
    }
}
