use core::cell::Cell;
use core::fmt::Write as _;
use core::marker::PhantomData;

use crate::cli::cmd_mode::RadioMode;

pub(crate) struct PromptStatus<'d> {
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
        let precision = match self.mode {
            RadioMode::FM => format_args!("{:.1}", self.frequency),
            _ => format_args!("{:.3}", self.frequency),
        };
        let _ = write!(
            self.prompt.get_mut(),
            "{BOLD_GREEN}radio-cli ({BOLD_BLUE}{:?} {BOLD_YELLOW}{precision} MHz{BOLD_GREEN})>{RESET} ",
            self.mode,
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
