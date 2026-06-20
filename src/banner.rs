use core::fmt::Write as _;

use crate::console::console_codes::*;
use crate::console::*;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub fn print_banner() {
    let mut out = stdout_get();

    let git_version = built_info::GIT_VERSION.unwrap_or("unknown");
    let dirty_suffix = match built_info::GIT_DIRTY {
        Some(true) => " (dirty)",
        _ => "",
    };
    let _ = write!(
        out,
        "{CRLF}{} v{} git: {}{}\n\r==============================={CRLF}",
        built_info::PKG_NAME,
        built_info::PKG_VERSION,
        git_version,
        dirty_suffix
    );
}
