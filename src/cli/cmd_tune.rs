use core::str::FromStr;

use crate::events;
use crate::events::SystemEvent;
use embedded_cli::Command;
use embedded_cli::arguments::*;
use si473x::RadioBand;

#[derive(Debug, Clone, Copy)]
pub struct RadioBandArg {
    pub(crate) band: RadioBand,
}

#[derive(Debug, Command, Clone, Copy)]
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
    /// List available bands
    List,
    /// Set a band
    Band {
        /// Band to set
        band: RadioBandArg,
    },
}

impl<'a> FromArgument<'a> for RadioBandArg {
    fn from_arg(arg: &'a str) -> Result<Self, FromArgumentError<'a>>
    where
        Self: Sized,
    {
        let radio_band = RadioBand::from_str(arg);
        radio_band
            .map(|band| RadioBandArg { band })
            .map_err(|_| FromArgumentError {
                value: arg,
                expected: "sss",
            })
    }
}

impl TuneCommand {
    pub async fn execute<T: core::fmt::Write>(self, writer: &mut T) {
        match self {
            TuneCommand::Up => {
                let _ = writer.write_str("Tuning up");
                events::event_send(SystemEvent::RadioSeekUp).await;
            }
            TuneCommand::Down => {
                let _ = writer.write_str("Tuning down not supported");
            }
            TuneCommand::Frequency { frequency } => {
                events::event_send(SystemEvent::RadioSetFrequency(frequency)).await;
            }
            TuneCommand::List => {
                let _ = writer.write_str("Available bands: FM, AM");
                RadioBand::for_each(|band| {
                    write!(writer, "- {}", band).ok();
                });
            }
            TuneCommand::Band { band } => {
                write!(writer, "{:?}", band).ok();
                events::event_send(SystemEvent::RadioBand(band.band)).await;
            }
        }
    }
}
