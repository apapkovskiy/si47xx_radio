use core::str::FromStr;

use crate::events;
use crate::events::SystemEvent;
use embedded_cli::Command;
use si473x::RadioBand;

#[derive(Debug, Command)]
pub(crate) enum TuneCommand<'a> {
    /// Seek up
    Up,
    /// Seek down
    Down,
    /// Set frequency
    Frequency {
        /// Frequency in MHz
        frequency: f32,
    },
    /// List available bands
    List,
    /// Set a band
    Band {
        /// Band to set
        band: &'a str,
    },
}

impl<'a> TuneCommand<'a> {
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
            TuneCommand::List => {
                let _ = writer.write_str("Available bands: FM, AM");
                RadioBand::for_each(|band| {
                    write!(writer, "\n- {}", band).ok();
                });
            }
            TuneCommand::Band { band } => {
                let radio_band = RadioBand::from_str(band);
                if let Ok(radio_band) = radio_band {
                    events::event_try_send(SystemEvent::RadioBand(radio_band));
                } else {
                    let _ = writer.write_fmt(format_args!("Invalid band: {}", band));
                }
            }
        }
    }
}
