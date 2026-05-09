use crate::boards::hal::*;
use crate::console;
use crate::events;
use crate::events::SystemNotify;
use core::fmt::{Debug, Write};
use embassy_futures::select::{Either, select};
use embedded_cli::cli::CliBuilder;
use embedded_cli::{Command, codes};

mod cmd_mode;
use cmd_mode::RadioMode;
mod cmd_tune;
use cmd_tune::TuneCommand;
mod cmd_volume;
use cmd_volume::VolumeCommand;
mod prompt;
use prompt::PromptStatus;

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

fn cli_handle_notification(
    writer: &mut dyn Write,
    event: SystemNotify,
    prompt_status: &mut PromptStatus,
) {
    match event {
        SystemNotify::RadioAmOn => {
            prompt_status.set_mode(RadioMode::AM);
            write!(writer, "Switched to AM mode").ok();
        }
        SystemNotify::RadioFmOn => {
            prompt_status.set_mode(RadioMode::FM);
            write!(writer, "Switched to FM mode").ok();
        }
        SystemNotify::RadioOff => {
            prompt_status.set_mode(RadioMode::Off);
            write!(writer, "Radio powered off").ok();
        }
        SystemNotify::TuneStatus(tune_status) => {
            prompt_status.set_frequency(tune_status.frequency);
            write!(
                writer,
                "Tuned to frequency {} MHz, {:#?}",
                tune_status.frequency, tune_status
            )
            .ok();
        }
        _ => {
            write!(writer, "Notification: {:#?}", event).ok();
        }
    }
}

#[embassy_executor::task]
pub async fn my_task(mut rx: HalUartRx) {
    let (command_buffer, history_buffer) = unsafe {
        static mut COMMAND_BUFFER: [u8; 40] = [0; 40];
        static mut HISTORY_BUFFER: [u8; 41] = [0; 41];
        #[allow(static_mut_refs)]
        (COMMAND_BUFFER.as_mut(), HISTORY_BUFFER.as_mut())
    };
    let mut prompt_status: PromptStatus = PromptStatus::new();
    let mut cli = CliBuilder::default()
        .writer(console::stdout_get())
        .command_buffer(command_buffer)
        .history_buffer(history_buffer)
        .prompt(prompt_status.get_prompt())
        .build()
        .ok()
        .unwrap();

    let mut notification_subscriber = events::notify_subscriber().unwrap();

    loop {
        let buffer = &mut [0u8; 1];

        loop {
            let char = rx.read(buffer);
            match select(char, notification_subscriber.next_message_pure()).await {
                Either::First(_) => break,
                Either::Second(event) => {
                    cli.write(|writer| {
                        cli_handle_notification(writer, event, &mut prompt_status);
                        Ok(())
                    })
                    .ok();
                    cli.set_prompt(prompt_status.get_prompt()).ok();
                }
            }
        }

        if buffer[0] == DEL {
            // Currently CLI does not handle DEL
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
                    let _ = cli
                        .writer()
                        .write_str("System status: All systems operational");
                    Ok(())
                }
                BaseCommand::Mode { command } => {
                    command.execute();
                    Ok(())
                }
                BaseCommand::Volume { command } => {
                    command.execute(cli.writer());
                    Ok(())
                }
                BaseCommand::Tune { command } => {
                    command.execute(cli.writer());
                    Ok(())
                }
            }),
        );
    }
}
