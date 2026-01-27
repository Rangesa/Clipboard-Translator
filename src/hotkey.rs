use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_C, VK_CONTROL};

/// ダブルプレスの有効期間（この時間内に2回目のCを押す必要がある）
const DOUBLE_PRESS_WINDOW_MS: u128 = 500;

/// キー検知後のデバウンス遅延（連続トリガー防止）
const DEBOUNCE_DELAY_MS: u64 = 200;

static C_PRESS_COUNT: AtomicU8 = AtomicU8::new(0);
static LAST_C_PRESS: Mutex<Option<Instant>> = Mutex::new(None);
static LAST_C_STATE: AtomicBool = AtomicBool::new(false);

pub fn is_ctrl_c_c_pressed() -> bool {
    // GetAsyncKeyStateはunsafeだが、static mutは排除
    let (ctrl_pressed, c_pressed) = unsafe {
        let ctrl_state = GetAsyncKeyState(VK_CONTROL.0 as i32);
        let c_state = GetAsyncKeyState(VK_C.0 as i32);
        (ctrl_state < 0, (c_state as u16 & 0x8000) != 0)
    };

    let last_c_state = LAST_C_STATE.load(Ordering::SeqCst);

    // Ctrl + C が押された瞬間を検知（エッジ検出）
    if ctrl_pressed && c_pressed && !last_c_state {
        let now = Instant::now();

        let mut last_press = match LAST_C_PRESS.lock() {
            Ok(guard) => guard,
            Err(_) => return false, // poisoned mutex - 安全に無視
        };

        match *last_press {
            Some(last_time) => {
                let elapsed = now.duration_since(last_time);

                // 規定時間内に2回目のCが押された
                if elapsed.as_millis() < DOUBLE_PRESS_WINDOW_MS {
                    let count = C_PRESS_COUNT.fetch_add(1, Ordering::SeqCst) + 1;

                    if count >= 2 {
                        // トリガー成功、カウントリセット
                        C_PRESS_COUNT.store(0, Ordering::SeqCst);
                        *last_press = None;
                        drop(last_press); // ロック解放
                        LAST_C_STATE.store(c_pressed, Ordering::SeqCst);

                        // キーが離されるまで待機（連続検知防止）
                        std::thread::sleep(Duration::from_millis(DEBOUNCE_DELAY_MS));
                        return true;
                    }
                } else {
                    // タイムアウト、カウントリセット
                    C_PRESS_COUNT.store(1, Ordering::SeqCst);
                }

                *last_press = Some(now);
            }
            None => {
                // 初回のC押下
                *last_press = Some(now);
                C_PRESS_COUNT.store(1, Ordering::SeqCst);
            }
        }
    }

    LAST_C_STATE.store(c_pressed, Ordering::SeqCst);
    false
}
