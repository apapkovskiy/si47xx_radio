use core::cell::Cell;
use core::fmt::Write as _;
use core::marker::PhantomData;
use si473x::{RadioBand, Si47xxTuneStatus, Volume};

use crate::cli::cmd_mode::RadioMode;

pub(crate) struct PromptStatus<'d> {
    tune_status: Si47xxTuneStatus,
    band: RadioBand,
    mode: RadioMode,
    prompt: Cell<heapless::String<128>>,
    volume: Option<Volume>,
    _p: PhantomData<&'d ()>,
}

impl<'d> PromptStatus<'d> {
    pub const fn new() -> Self {
        Self {
            tune_status: Si47xxTuneStatus::new(),
            band: RadioBand::UNKNOWN,
            mode: RadioMode::Off,
            prompt: Cell::new(heapless::String::new()),
            volume: Volume::new(0),
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
        use crate::console::console_codes::*;
        use crate::console::console_colors::*;
        self.prompt.get_mut().clear();
        let precision = match self.mode {
            RadioMode::FM => format_args!("{:.1}", self.tune_status.frequency),
            _ => format_args!("{:.3}", self.tune_status.frequency),
        };
        let _ = write!(
            self.prompt.get_mut(),
            "{CURSOR_OFF}{BOLD_GREEN}radio-cli ({BOLD_BLUE}{:?} {BOLD_YELLOW}{precision} MHz, {}, SNR: {} V: {}{BOLD_GREEN})>{RESET}{CURSOR_ON} ",
            self.mode,
            self.band,
            self.tune_status.snr,
            self.volume.map_or(0, |v| v.get())
        );
        self.get_prompt_str()
    }

    pub fn set_mode(&mut self, mode: RadioMode) -> &mut Self {
        self.mode = mode;
        self
    }
    pub fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.tune_status.frequency = frequency;
        self
    }
    pub fn set_volume(&mut self, volume: Volume) -> &mut Self {
        self.volume = Some(volume);
        self
    }
    pub fn set_band(&mut self, band: RadioBand) -> &mut Self {
        self.band = band;
        self
    }
}
