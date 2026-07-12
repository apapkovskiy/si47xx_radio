use crate::settings::{
    OPTIONS, option::ConfigOption, option::OptionString, option::OptionToString,
};
use core::str::FromStr;
use heapless::String;
use linkme::distributed_slice;

static CONFIG_RADIO_MODE: ConfigOption<RadioMode> = ConfigOption::new(
    "radio_mode",
    RadioMode::AM,
    &CONFIG_RADIO_MODE,
    "Radio mode (FM, AM, Off)",
);
#[distributed_slice(OPTIONS)]
static CONFIG_RADIO_MODE_STR: &'static OptionString = &CONFIG_RADIO_MODE.option;

#[derive(Debug, Copy, Clone)]
pub enum RadioMode {
    /// FM Mode
    FM,
    /// AM Mode
    AM,
    /// Power down the radio
    Off,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ParseRadioModeError;

impl RadioMode {
    pub async fn get() -> Self {
        CONFIG_RADIO_MODE.get().await
    }
    pub async fn save(&self) {
        CONFIG_RADIO_MODE.set(self).await;
    }
}

impl FromStr for RadioMode {
    type Err = ParseRadioModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "FM" => Ok(RadioMode::FM),
            "AM" => Ok(RadioMode::AM),
            "Off" => Ok(RadioMode::Off),
            _ => Err(ParseRadioModeError),
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

impl AsRef<str> for RadioMode {
    fn as_ref(&self) -> &'static str {
        self.into()
    }
}

impl<const N: usize> OptionToString<N> for RadioMode {
    fn to_string(&self) -> String<N> {
        String::<N>::try_from(self.as_ref()).unwrap_or_default()
    }
}
