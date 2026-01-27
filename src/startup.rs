use anyhow::{Context, Result};
use std::env;
use winreg::enums::*;
use winreg::RegKey;

const APP_NAME: &str = "ClipboardTranslator";

pub fn install_startup() -> Result<()> {
    let exe_path = env::current_exe().context("Failed to get executable path")?;
    let exe_path_str = exe_path.to_string_lossy();

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
        .context("Failed to open registry key")?;

    key.set_value(APP_NAME, &exe_path_str.as_ref())
        .context("Failed to set registry value")?;

    Ok(())
}

pub fn uninstall_startup() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey_with_flags(r"Software\Microsoft\Windows\CurrentVersion\Run", KEY_WRITE)
        .context("Failed to open registry key")?;

    // 値が存在しない場合もエラーにしない
    let _ = key.delete_value(APP_NAME);

    Ok(())
}

pub fn is_installed() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run") {
        key.get_value::<String, _>(APP_NAME).is_ok()
    } else {
        false
    }
}
