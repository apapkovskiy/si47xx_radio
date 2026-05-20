use crate::events;
use crate::events::SystemEvent;
use embedded_cli::Command;

#[derive(Debug, Command)]
pub(crate) enum PropertyCommand {
    /// Set a radio property by ID and value.
    Set {
        /// Property ID
        id: u16,
        /// Property value
        value: u16,
    },
    /// List all properties
    List,
}

impl PropertyCommand {
    pub fn execute<T: core::fmt::Write>(self, writer: &mut T) {
        match self {
            PropertyCommand::Set { id, value } => {
                let _ = writer.write_fmt(format_args!("Setting property {} to {}", id, value));
                events::event_try_send(SystemEvent::RadioPropertySet(id, value));
            }
            PropertyCommand::List => {
                events::event_try_send(SystemEvent::RadioPropertyList);
            }
        }
    }
}
