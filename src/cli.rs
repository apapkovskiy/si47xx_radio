use crate::boards::hal::*;
use crate::console;
use crate::events;
use crate::events::SystemNotify;
use core::fmt::{Arguments, Write};
use embassy_futures::select::{Either, select};
use embedded_cli::buffer::Buffer;
use embedded_cli::cli::{Cli, CliBuilder};

mod cli_base;
use cli_base::{BaseCommand, process_byte};
mod cmd_config;
mod cmd_mode;
use cmd_mode::RadioMode;
mod cmd_property;
mod cmd_tune;
mod cmd_volume;
mod prompt;
use prompt::PromptStatus;

const CLI_READ_BUFFER_SIZE: usize = 32;

struct CliApi<'a, W: embedded_io::Write<Error = E>, E: embedded_io::Error, C: Buffer, H: Buffer>(
    &'a mut Cli<W, E, C, H>,
);

impl<'a, W: embedded_io::Write<Error = E>, E: embedded_io::Error, C: Buffer, H: Buffer> Write
    for CliApi<'a, W, E, C, H>
{
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        let _ = self.0.write(|writer| writer.write_str(s));
        Ok(())
    }

    fn write_fmt(&mut self, args: Arguments<'_>) -> Result<(), core::fmt::Error> {
        let _ = self.0.write(|writer| {
            writer.write_fmt(args).ok();
            Ok(())
        });
        Ok(())
    }
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
        }
        SystemNotify::RadioPropertyInfo(id, value) => {
            write!(
                writer,
                "Property ID: {:?}({}), Value: {}",
                id, id as u16, value
            )
            .ok();
        }
        SystemNotify::BandChanged(band) => {
            prompt_status.set_band(band);
            write!(writer, "Band changed to {}", band).ok();
            prompt_status.set_band(band);
            prompt_status.set_band(band);
        }
        SystemNotify::VolumeChanged(volume) => {
            prompt_status.set_volume(volume);
        }
        _ => {
            write!(writer, "Notification: {:#?}", event).ok();
        }
    }
}

async fn handle_command(command: &BaseCommand, writer: &mut impl Write) {
    match command {
        BaseCommand::Status => {
            let _ = writer.write_str("System status: All systems operational");
        }
        BaseCommand::Mode { command } => {
            command.execute().await;
        }
        BaseCommand::Volume { command } => {
            command.execute(writer).await;
        }
        BaseCommand::Tune { command } => {
            command.execute(writer).await;
        }
        BaseCommand::Property { command } => {
            command.execute(writer).await;
        }
        BaseCommand::Config { command } => {
            command.execute(writer).await;
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
        let mut buffer = [0u8; CLI_READ_BUFFER_SIZE];

        loop {
            buffer.fill(0);
            let char = rx.read_until_idle(&mut buffer);
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
        let mut cmd = None::<BaseCommand>;
        for byte in buffer {
            if byte == 0 {
                break;
            }
            process_byte(&mut cli, byte, &mut cmd);
            if let Some(command) = cmd {
                handle_command(&command, &mut CliApi(&mut cli)).await;
            }
        }
    }
}
