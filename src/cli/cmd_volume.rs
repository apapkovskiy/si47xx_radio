use crate::events;
use crate::events::SystemEvent;
use embedded_cli::Command;
use si473x::Volume;

#[derive(Debug, Command, Clone, Copy)]
pub(crate) enum VolumeCommand {
    /// Increase volume
    Up,
    /// Decrease volume
    Down,
    /// Set volume to specific level
    Set {
        /// Volume level (0-100)
        level: u8,
    },
}

impl VolumeCommand {
    pub async fn execute<T: core::fmt::Write>(self, writer: &mut T) {
        match self {
            VolumeCommand::Up => {
                let _ = writer.write_str("Volume increased");
                events::event_send(SystemEvent::RadioVolumeUp).await;
            }
            VolumeCommand::Down => {
                let _ = writer.write_str("Volume decreased");
                events::event_send(SystemEvent::RadioVolumeDown).await;
            }
            VolumeCommand::Set { level } => {
                let volume = Volume::try_from(level);
                if let Ok(volume) = volume {
                    let _ = writer.write_fmt(format_args!("Volume set to {}", volume.get()));
                    events::event_send(SystemEvent::RadioVolumeSet(volume)).await;
                } else {
                    let _ = writer.write_fmt(format_args!(
                        "Invalid volume level: {}. Must be between 0 and 100.",
                        level
                    ));
                }
            }
        }
    }
}
