use crate::events;
use crate::events::SystemEvent;
use embedded_cli::Command;
use si473x::Si47xxProperty;

#[derive(Debug, Command, Clone, Copy)]
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
    pub async fn execute<T: core::fmt::Write>(self, writer: &mut T) {
        match self {
            PropertyCommand::Set { id, value } => {
                let property = Si47xxProperty::try_from(id);
                match property {
                    Ok(property) => {
                        writer
                            .write_fmt(format_args!("Setting property {:?} to {}", property, value))
                            .ok();
                        events::event_send(SystemEvent::RadioPropertySet(property, value)).await;
                    }
                    Err(e) => {
                        writer
                            .write_fmt(format_args!("Invalid property ID: {}, error: {:?}", id, e))
                            .ok();
                    }
                }
            }
            PropertyCommand::List => {
                events::event_send(SystemEvent::RadioPropertyList).await;
            }
        }
    }
}
