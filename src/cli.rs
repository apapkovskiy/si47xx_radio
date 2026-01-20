use core::fmt::Write;

use embassy_nrf::uarte;
use embedded_cli::cli::CliBuilder;
use embedded_cli::{Command, codes};
use crate::events;
use crate::console;

pub const DEL: u8 = 127; // Delete character

#[derive(Debug, Command)]
enum BaseCommand {
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
    /// Show some status
    Status,
}

#[derive(Debug, Command)]
enum RadioMode {
    /// FM Mode
    FM,
    /// AM Mode
    AM,
    /// Power down the radio
    Off,
}

#[derive(Debug, Command)]
enum TuneCommand {
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

#[derive(Debug, Command)]
enum VolumeCommand {
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

#[embassy_executor::task]
pub async fn my_task(mut rx: uarte::UarteRx<'static>) {

    let (command_buffer, history_buffer) = unsafe {
        static mut COMMAND_BUFFER: [u8; 40] = [0; 40];
        static mut HISTORY_BUFFER: [u8; 41] = [0; 41];
        #[allow(static_mut_refs)]
        (COMMAND_BUFFER.as_mut(), HISTORY_BUFFER.as_mut())
    };
    let mut cli = CliBuilder::default()
        .writer(console::stdout_get())
        .command_buffer(command_buffer)
        .history_buffer(history_buffer)
        .build()
        .ok().unwrap();

    loop {
        let buffer = &mut [0u8; 1];
        rx.read(buffer).await.unwrap();
        if buffer[0] == DEL { // Currently CLI does not handle DEL
            buffer[0] = codes::BACKSPACE; // To overcome map DEL to BACKSPACE
        }

        // Process incoming byte
        // Command type is specified for autocompletion and help
        // Processor accepts closure where we can process parsed command
        // we can use different command and processor with each call
        let _ = cli.process_byte::<BaseCommand, _>(
            buffer[0],
            &mut BaseCommand::processor(|cli, command| match command {
                BaseCommand::Status => {
                    let _ = cli.writer().write_str("System status: All systems operational");
                    Ok(())
                },
                BaseCommand::Mode { command } => {
                    match command {
                        RadioMode::FM => {
                            let _ = cli.writer().write_str("Switched to FM mode");
                            events::event_try_send(events::SystemEvent::RadioFmOn);
                        },
                        RadioMode::AM => {
                            let _ = cli.writer().write_str("Switched to AM mode");
                            events::event_try_send(events::SystemEvent::RadioAmOn);
                        },
                        RadioMode::Off => {
                            let _ = cli.writer().write_str("Radio powered off");
                            events::event_try_send(events::SystemEvent::RadioOff);
                        },
                    }
                    Ok(())
                },
                BaseCommand::Volume { command } => {
                    match command {
                        VolumeCommand::Up => {
                            let _ = cli.writer().write_str("Volume increased");
                            events::event_try_send(events::SystemEvent::RadioVolumeUp);
                        },
                        VolumeCommand::Down => {
                            let _ = cli.writer().write_str("Volume decreased");
                            events::event_try_send(events::SystemEvent::RadioVolumeDown);
                        },
                        VolumeCommand::Set { level } => {
                            let _ = cli.writer().write_fmt(format_args!("Volume set to {}", level));
                            events::event_try_send(events::SystemEvent::RadioVolumeSet(level));
                        },
                    }
                    Ok(())
                },
                BaseCommand::Tune { command } => {
                    match command {
                        TuneCommand::Up => {
                            let _ = cli.writer().write_str("Tuning up");
                            events::event_try_send(events::SystemEvent::RadioSeekUp);
                        },
                        TuneCommand::Down => {
                            let _ = cli.writer().write_str("Tuning down not supported");
                        },
                        TuneCommand::Frequency { frequency } => {
                            let _ = cli.writer().write_fmt(format_args!("Frequency set to {} MHz", frequency));
                            events::event_try_send(events::SystemEvent::RadioSetFrequency(frequency));
                        },
                    }
                    Ok(())
                },
            }),
        );
    }
}
