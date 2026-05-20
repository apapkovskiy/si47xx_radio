use crate::settings::{OPTIONS, option::OptionString};
use linkme::distributed_slice;

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
