use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_MENU, VK_SHIFT};

use crate::config::Hotkey;

/// ダブルプレスの有効期間（この時間内に2回目を押す必要がある）
const DOUBLE_PRESS_WINDOW_MS: u128 = 500;

/// キー検知後のデバウンス遅延（連続トリガー防止）
const DEBOUNCE_DELAY_MS: u64 = 200;

static LAST_KEY_STATE: AtomicBool = AtomicBool::new(false);
static KEY_PRESS_COUNT: AtomicU8 = AtomicU8::new(0);
static LAST_KEY_PRESS: Mutex<Option<Instant>> = Mutex::new(None);

/// 指定されたホットキーが押されたかをチェック
pub fn is_hotkey_pressed(hotkey: &Hotkey) -> bool {
    if hotkey.is_double_press {
        check_double_press_hotkey(hotkey)
    } else {
        check_single_press_hotkey(hotkey)
    }
}

/// シングルプレスのホットキーをチェック
fn check_single_press_hotkey(hotkey: &Hotkey) -> bool {
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

/// ダブルプレスのホットキーをチェック（例: Ctrl+C+C）
fn check_double_press_hotkey(hotkey: &Hotkey) -> bool {
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

    // エッジ検出: 修飾キー + メインキーが押された瞬間
    if modifiers_match && main_key_state && !last_state {
        let now = Instant::now();

        let mut last_press = match LAST_KEY_PRESS.lock() {
            Ok(guard) => guard,
            Err(_) => return false, // poisoned mutex
        };

        match *last_press {
            Some(last_time) => {
                let elapsed = now.duration_since(last_time);

                // 規定時間内に2回目が押された
                if elapsed.as_millis() < DOUBLE_PRESS_WINDOW_MS {
                    let count = KEY_PRESS_COUNT.fetch_add(1, Ordering::SeqCst) + 1;

                    if count >= 2 {
                        // ダブルプレス成功
                        KEY_PRESS_COUNT.store(0, Ordering::SeqCst);
                        *last_press = None;
                        drop(last_press);
                        LAST_KEY_STATE.store(main_key_state, Ordering::SeqCst);

                        std::thread::sleep(Duration::from_millis(DEBOUNCE_DELAY_MS));
                        return true;
                    }
                } else {
                    // タイムアウト、カウントリセット
                    KEY_PRESS_COUNT.store(1, Ordering::SeqCst);
                }

                *last_press = Some(now);
            }
            None => {
                // 初回のキー押下
                *last_press = Some(now);
                KEY_PRESS_COUNT.store(1, Ordering::SeqCst);
            }
        }
    }

    LAST_KEY_STATE.store(main_key_state, Ordering::SeqCst);
    false
}
