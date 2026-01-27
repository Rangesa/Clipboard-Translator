use anyhow::{Context, Result};
use clipboard_win::{formats, get_clipboard};

pub fn get_text() -> Result<String> {
    get_clipboard(formats::Unicode).context("Failed to read clipboard")
}
