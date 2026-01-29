use crate::console;
use crate::events;
use crate::events::SystemEvent;
use crate::events::SystemNotify;
use core::cell::Cell;
use core::fmt::{Debug, Write};
use core::marker::PhantomData;
use embassy_futures::select::{Either, select};
use embassy_nrf::uarte;
use embedded_cli::cli::CliBuilder;
use embedded_cli::{Command, codes};

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

struct PromptStatus<'d> {
    frequency: f32,
    mode: RadioMode,
    prompt: Cell<heapless::String<64>>,
    _p: PhantomData<&'d ()>,
}

impl<'d> PromptStatus<'d> {
    pub const fn new() -> Self {
        Self {
            frequency: 0.0,
            mode: RadioMode::FM,
            prompt: Cell::new(heapless::String::new()),
            _p: PhantomData {},
        }
    }

    fn get_prompt_str(&self) -> &'d str {
        unsafe {
            let ptr = self.prompt.as_ptr();
            let str = &*ptr;
            str.as_str()
        }
    }

    pub fn get_prompt(&mut self) -> &'d str {
        use crate::console::console_colors::*;
        self.prompt.get_mut().clear();
        let _ = write!(
            self.prompt.get_mut(),
            "{BOLD_GREEN}radio-cli {BOLD_BLUE}{:?} {BOLD_YELLOW}{:.1} MHz{BOLD_GREEN})>{RESET} ",
            self.mode,
            self.frequency,
        );
        self.get_prompt_str()
    }

    pub fn set_mode(&mut self, mode: RadioMode) -> &mut Self {
        self.mode = mode;
        self
    }
    pub fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.frequency = frequency;
        self
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
            write!(
                writer,
                "Tuned to frequency {} MHz, {:?}",
                tune_status.frequency, tune_status
            )
            .ok();
        }
        _ => {
            write!(writer, "Notification: {:?}", event).ok();
        }
    }
}

#[embassy_executor::task]
pub async fn my_task(mut rx: uarte::UarteRx<'static>) {
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
                    match command {
                        RadioMode::FM => events::event_try_send(SystemEvent::RadioFmOn),
                        RadioMode::AM => events::event_try_send(SystemEvent::RadioAmOn),
                        RadioMode::Off => events::event_try_send(SystemEvent::RadioOff),
                    }
                    Ok(())
                }
                BaseCommand::Volume { command } => {
                    match command {
                        VolumeCommand::Up => {
                            let _ = cli.writer().write_str("Volume increased");
                            events::event_try_send(SystemEvent::RadioVolumeUp);
                        }
                        VolumeCommand::Down => {
                            let _ = cli.writer().write_str("Volume decreased");
                            events::event_try_send(SystemEvent::RadioVolumeDown);
                        }
                        VolumeCommand::Set { level } => {
                            let _ = cli
                                .writer()
                                .write_fmt(format_args!("Volume set to {}", level));
                            events::event_try_send(SystemEvent::RadioVolumeSet(level));
                        }
                    }
                    Ok(())
                }
                BaseCommand::Tune { command } => {
                    match command {
                        TuneCommand::Up => {
                            let _ = cli.writer().write_str("Tuning up");
                            events::event_try_send(SystemEvent::RadioSeekUp);
                        }
                        TuneCommand::Down => {
                            let _ = cli.writer().write_str("Tuning down not supported");
                        }
                        TuneCommand::Frequency { frequency } => {
                            events::event_try_send(SystemEvent::RadioSetFrequency(frequency));
                        }
                    }
                    Ok(())
                }
            }),
        );
    }
}
