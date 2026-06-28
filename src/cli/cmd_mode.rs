use crate::events;
use crate::events::SystemEvent;
use embedded_cli::Command;

#[derive(Debug, Command, Clone, Copy)]
pub(crate) enum RadioMode {
    /// FM Mode
    FM,
    /// AM Mode
    AM,
    /// Power down the radio
    Off,
}

impl RadioMode {
    pub async fn execute(self) {
        match self {
            RadioMode::FM => events::event_send(SystemEvent::RadioFmOn),
            RadioMode::AM => events::event_send(SystemEvent::RadioAmOn),
            RadioMode::Off => events::event_send(SystemEvent::RadioOff),
        }
        .await
    }
}
