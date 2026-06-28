use crate::settings::OPTIONS;
use embedded_cli::Command;

#[derive(Debug, Command, Clone, Copy)]
pub(crate) enum ConfigCommand {
    /// List all configuration values
    List,
}

impl ConfigCommand {
    pub async fn execute<T: core::fmt::Write>(self, writer: &mut T) {
        match self {
            ConfigCommand::List => {
                for option in OPTIONS {
                    writer
                        .write_fmt(format_args!(
                            "{}: {}",
                            option.get_key(),
                            option.get().await.as_str()
                        ))
                        .ok();
                }
            }
        }
    }
}
