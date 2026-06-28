#![allow(clippy::large_stack_frames)]

use embedded_cli::cli::Cli;
use embedded_cli::{Command, buffer::Buffer, codes};

use crate::cli::cmd_config::ConfigCommand;
use crate::cli::cmd_mode::RadioMode;
use crate::cli::cmd_property::PropertyCommand;
use crate::cli::cmd_tune::TuneCommand;
use crate::cli::cmd_volume::VolumeCommand;

const DEL: u8 = 127; // Delete character

#[derive(Debug, Command, Clone, Copy)]
pub enum BaseCommand {
    Mode {
        #[command(subcommand)]
        command: RadioMode,
    },
    Volume {
        #[command(subcommand)]
        command: VolumeCommand,
    },
    Tune {
        #[command(subcommand)]
        command: TuneCommand,
    },
    Property {
        #[command(subcommand)]
        command: PropertyCommand,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Show some status
    Status,
}

pub fn process_byte<
    W: embedded_io::Write<Error = E>,
    E: embedded_io::Error,
    C: Buffer,
    H: Buffer,
>(
    cli: &mut Cli<W, E, C, H>,
    mut byte: u8,
    command: &mut Option<BaseCommand>,
) {
    if byte == DEL {
        // Currently CLI does not handle DEL
        byte = codes::BACKSPACE; // To overcome map DEL to BACKSPACE
    }

    // Process incoming byte
    // Command type is specified for autocompletion and help
    // Processor accepts closure where we can process parsed command
    // we can use different command and processor with each call
    let _ = cli.process_byte::<BaseCommand, _>(
        byte,
        &mut BaseCommand::processor(|_cli, cmd| {
            *command = Some(cmd);
            Ok(())
        }),
    );
}
