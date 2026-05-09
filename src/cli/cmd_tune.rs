use crate::events;
use crate::events::SystemEvent;
use embedded_cli::Command;

#[derive(Debug, Command)]
pub(crate) enum TuneCommand {
    /// Seek up
    Up,
    /// Seek down
    Down,
    /// Set frequency
    Frequency {
        /// Frequency in MHz
        frequency: f32,
    },
}

impl TuneCommand {
    pub fn execute<T: core::fmt::Write>(self, writer: &mut T) {
        match self {
            TuneCommand::Up => {
                let _ = writer.write_str("Tuning up");
                events::event_try_send(SystemEvent::RadioSeekUp);
            }
            TuneCommand::Down => {
                let _ = writer.write_str("Tuning down not supported");
            }
            TuneCommand::Frequency { frequency } => {
                events::event_try_send(SystemEvent::RadioSetFrequency(frequency));
            }
        }
    }
}
