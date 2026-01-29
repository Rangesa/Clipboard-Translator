use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VK_CONTROL, VK_MENU, VK_SHIFT, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
    HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
};

use crate::config::Hotkey;

/// ダブルプレスの有効期間（この時間内に2回目を押す必要がある）
const DOUBLE_PRESS_WINDOW_MS: u128 = 500;

/// 現在監視中のホットキー
static CURRENT_HOTKEY: Mutex<Option<Hotkey>> = Mutex::new(None);

/// ホットキーが押されたフラグ
static HOTKEY_TRIGGERED: AtomicBool = AtomicBool::new(false);

/// ダブルプレス検出用
static KEY_PRESS_COUNT: AtomicU8 = AtomicU8::new(0);
static LAST_KEY_PRESS: Mutex<Option<Instant>> = Mutex::new(None);

/// 修飾キーの状態
static CTRL_PRESSED: AtomicBool = AtomicBool::new(false);
static ALT_PRESSED: AtomicBool = AtomicBool::new(false);
static SHIFT_PRESSED: AtomicBool = AtomicBool::new(false);

/// Low-Level キーボードフックプロシージャ
unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        let vk_code = VIRTUAL_KEY(kb.vkCode as u16);

        // キーダウンイベントのみ処理
        if wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN {
            // 修飾キーの状態を追跡
            match vk_code {
                VK_CONTROL => {
                    CTRL_PRESSED.store(true, Ordering::SeqCst);
                }
                VK_MENU => {
                    ALT_PRESSED.store(true, Ordering::SeqCst);
                }
                VK_SHIFT => {
                    SHIFT_PRESSED.store(true, Ordering::SeqCst);
                }
                _ => {
                    // メインキーが押された
                    check_hotkey_match(kb.vkCode as i32);
                }
            }
        } else {
            // キーアップイベント
            match vk_code {
                VK_CONTROL => {
                    CTRL_PRESSED.store(false, Ordering::SeqCst);
                }
                VK_MENU => {
                    ALT_PRESSED.store(false, Ordering::SeqCst);
                }
                VK_SHIFT => {
                    SHIFT_PRESSED.store(false, Ordering::SeqCst);
                }
                _ => {}
            }
        }
    }

    CallNextHookEx(HHOOK(0), code, wparam, lparam)
}

/// ホットキーのマッチをチェック
fn check_hotkey_match(vk_code: i32) {
    let hotkey = match CURRENT_HOTKEY.lock() {
        Ok(guard) => match *guard {
            Some(hk) => hk,
            None => return,
        },
        Err(_) => return,
    };

    // キーコードが一致するか
    if vk_code != hotkey.key_code {
        return;
    }

    // 修飾キーの状態が一致するか
    let ctrl = CTRL_PRESSED.load(Ordering::SeqCst);
    let alt = ALT_PRESSED.load(Ordering::SeqCst);
    let shift = SHIFT_PRESSED.load(Ordering::SeqCst);

    if ctrl != hotkey.ctrl || alt != hotkey.alt || shift != hotkey.shift {
        return;
    }

    // ダブルプレスチェック
    if hotkey.is_double_press {
        if check_double_press() {
            HOTKEY_TRIGGERED.store(true, Ordering::SeqCst);
        }
    } else {
        HOTKEY_TRIGGERED.store(true, Ordering::SeqCst);
    }
}

/// ダブルプレスをチェック
fn check_double_press() -> bool {
    let now = Instant::now();

    let mut last_press = match LAST_KEY_PRESS.lock() {
        Ok(guard) => guard,
        Err(_) => return false,
    };

    match *last_press {
        Some(last_time) => {
            let elapsed = now.duration_since(last_time);

            if elapsed.as_millis() < DOUBLE_PRESS_WINDOW_MS {
                let count = KEY_PRESS_COUNT.fetch_add(1, Ordering::SeqCst) + 1;

                if count >= 2 {
                    // ダブルプレス成功
                    KEY_PRESS_COUNT.store(0, Ordering::SeqCst);
                    *last_press = None;
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

    false
}

/// ホットキー監視を開始
pub fn start_hook(hotkey: Hotkey) -> windows::core::Result<()> {
    // 現在のホットキーを設定
    if let Ok(mut guard) = CURRENT_HOTKEY.lock() {
        *guard = Some(hotkey);
    }

    unsafe {
        // Low-Level キーボードフックを設定
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0)?;

        if hook.is_invalid() {
            return Err(windows::core::Error::from_win32());
        }

        // メッセージループ
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = DispatchMessageW(&msg);
        }

        // クリーンアップ
        let _ = UnhookWindowsHookEx(hook);
    }

    Ok(())
}

/// ホットキーがトリガーされたかチェック（メインスレッドから呼ぶ）
pub fn check_triggered() -> bool {
    HOTKEY_TRIGGERED.swap(false, Ordering::SeqCst)
}
