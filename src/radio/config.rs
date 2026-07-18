use crate::settings::{
    OPTIONS, option::ConfigOption, option::OptionString, option::OptionToString,
};
use heapless::String;
use linkme::distributed_slice;

use si473x::RadioBand;

static CONFIG_AM_BAND: ConfigOption<RadioBand> =
    ConfigOption::new("am_band", RadioBand::SW_120M, &CONFIG_AM_BAND, "AM band");
#[distributed_slice(OPTIONS)]
static CONFIG_AM_BAND_STR: &'static OptionString = &CONFIG_AM_BAND.option;

impl<const N: usize> OptionToString<N> for RadioBand {
    fn to_string(&self) -> String<N> {
        String::<N>::try_from(self.as_str()).unwrap_or_default()
    }
}

static CONFIG_AM_FREQ: ConfigOption<f32> =
    ConfigOption::new("am_freq", 11.600, &CONFIG_AM_FREQ, "AM frequency");
#[distributed_slice(OPTIONS)]
static CONFIG_AM_FREQ_STR: &'static OptionString = &CONFIG_AM_FREQ.option;

static CONFIG_AM_TUNE_SNR_THRESHOLD: ConfigOption<u16> = ConfigOption::new(
    "am_tune_snr_threshold",
    0,
    &CONFIG_AM_TUNE_SNR_THRESHOLD,
    "AM tune SNR threshold, dB (0–63)",
);
#[distributed_slice(OPTIONS)]
static CONFIG_AM_TUNE_SNR_THRESHOLD_STR: &'static OptionString =
    &CONFIG_AM_TUNE_SNR_THRESHOLD.option;

static CONFIG_AM_TUNE_RSSI_THRESHOLD: ConfigOption<u16> = ConfigOption::new(
    "am_tune_rssi_threshold",
    13,
    &CONFIG_AM_TUNE_RSSI_THRESHOLD,
    "AM tune RSSI threshold, dBµV (0–63)",
);
#[distributed_slice(OPTIONS)]
static CONFIG_AM_TUNE_RSSI_THRESHOLD_STR: &'static OptionString =
    &CONFIG_AM_TUNE_RSSI_THRESHOLD.option;

static CONFIG_FM_BAND: ConfigOption<RadioBand> =
    ConfigOption::new("fm_band", RadioBand::FM_US_EU, &CONFIG_FM_BAND, "FM band");
#[distributed_slice(OPTIONS)]
static CONFIG_FM_BAND_STR: &'static OptionString = &CONFIG_FM_BAND.option;

static CONFIG_FM_FREQ: ConfigOption<f32> =
    ConfigOption::new("fm_freq", 87.5, &CONFIG_FM_FREQ, "FM frequency");
#[distributed_slice(OPTIONS)]
static CONFIG_FM_FREQ_STR: &'static OptionString = &CONFIG_FM_FREQ.option;

static CONFIG_FM_TUNE_SNR_THRESHOLD: ConfigOption<u16> = ConfigOption::new(
    "fm_tune_snr_threshold",
    0,
    &CONFIG_FM_TUNE_SNR_THRESHOLD,
    "FM tune SNR threshold, dB (0–127)",
);
#[distributed_slice(OPTIONS)]
static CONFIG_FM_TUNE_SNR_THRESHOLD_STR: &'static OptionString =
    &CONFIG_FM_TUNE_SNR_THRESHOLD.option;

static CONFIG_FM_TUNE_RSSI_THRESHOLD: ConfigOption<u16> = ConfigOption::new(
    "fm_tune_rssi_threshold",
    20,
    &CONFIG_FM_TUNE_RSSI_THRESHOLD,
    "FM tune RSSI threshold, dBµV (0–127)",
);
#[distributed_slice(OPTIONS)]
static CONFIG_FM_TUNE_RSSI_THRESHOLD_STR: &'static OptionString =
    &CONFIG_FM_TUNE_RSSI_THRESHOLD.option;

pub struct RadioConfig;

impl RadioConfig {
    #![allow(dead_code)]
    pub async fn config_fm_band_get(&self) -> RadioBand {
        CONFIG_FM_BAND.get().await
    }
    pub async fn config_fm_band_set(&self, value: RadioBand) {
        CONFIG_FM_BAND.set(&value).await;
    }

    pub async fn config_fm_freq_get(&self) -> f32 {
        CONFIG_FM_FREQ.get().await
    }
    pub async fn config_fm_freq_set(&self, value: f32) {
        CONFIG_FM_FREQ.set(&value).await;
    }

    pub async fn config_fm_tune_snr_threshold_get(&self) -> u16 {
        CONFIG_FM_TUNE_SNR_THRESHOLD.get().await
    }
    pub async fn config_fm_tune_snr_threshold_set(&self, value: u16) {
        CONFIG_FM_TUNE_SNR_THRESHOLD.set(&value).await;
    }

    pub async fn config_fm_tune_rssi_threshold_get(&self) -> u16 {
        CONFIG_FM_TUNE_RSSI_THRESHOLD.get().await
    }
    pub async fn config_fm_tune_rssi_threshold_set(&self, value: u16) {
        CONFIG_FM_TUNE_RSSI_THRESHOLD.set(&value).await;
    }

    pub async fn config_am_band_get(&self) -> RadioBand {
        CONFIG_AM_BAND.get().await
    }
    pub async fn config_am_band_set(&self, value: RadioBand) {
        CONFIG_AM_BAND.set(&value).await;
    }

    pub async fn config_am_freq_get(&self) -> f32 {
        CONFIG_AM_FREQ.get().await
    }
    pub async fn config_am_freq_set(&self, value: f32) {
        CONFIG_AM_FREQ.set(&value).await;
    }

    pub async fn config_am_tune_rssi_threshold_get(&self) -> u16 {
        CONFIG_AM_TUNE_RSSI_THRESHOLD.get().await
    }
    pub async fn config_am_tune_rssi_threshold_set(&self, value: u16) {
        CONFIG_AM_TUNE_RSSI_THRESHOLD.set(&value).await;
    }

    pub async fn config_am_tune_snr_threshold_get(&self) -> u16 {
        CONFIG_AM_TUNE_SNR_THRESHOLD.get().await
    }
    pub async fn config_am_tune_snr_threshold_set(&self, value: u16) {
        CONFIG_AM_TUNE_SNR_THRESHOLD.set(&value).await;
    }
}
