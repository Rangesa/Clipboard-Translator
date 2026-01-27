use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_MENU, VK_SHIFT};

use crate::config::Hotkey;

/// キー検知後のデバウンス遅延（連続トリガー防止）
const DEBOUNCE_DELAY_MS: u64 = 200;

static LAST_KEY_STATE: AtomicBool = AtomicBool::new(false);

/// 指定されたホットキーが押されたかをチェック
pub fn is_hotkey_pressed(hotkey: &Hotkey) -> bool {
    let (ctrl_state, alt_state, shift_state, main_key_state) = unsafe {
        let ctrl = (GetAsyncKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0;
        let alt = (GetAsyncKeyState(VK_MENU.0 as i32) as u16 & 0x8000) != 0;
        let shift = (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0;
        let main_key = (GetAsyncKeyState(hotkey.key_code) as u16 & 0x8000) != 0;
        (ctrl, alt, shift, main_key)
    };

    // 修飾キーが一致しているかチェック
    let modifiers_match = ctrl_state == hotkey.ctrl
        && alt_state == hotkey.alt
        && shift_state == hotkey.shift;

    let last_state = LAST_KEY_STATE.load(Ordering::SeqCst);

    // エッジ検出: 修飾キー + メインキーが押された瞬間のみtrueを返す
    if modifiers_match && main_key_state && !last_state {
        LAST_KEY_STATE.store(true, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(DEBOUNCE_DELAY_MS));
        return true;
    }

    LAST_KEY_STATE.store(main_key_state, Ordering::SeqCst);
    false
}
