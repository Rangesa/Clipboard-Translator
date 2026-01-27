// コンソールウィンドウを非表示にする（リリースビルド時）
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use std::env;
use std::io::{self, Read};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::sync::{LazyLock, Mutex};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;

/// ホットキー監視のポーリング間隔
const HOTKEY_POLL_INTERVAL_MS: u64 = 100;

static RESULT_PROCESS: LazyLock<Mutex<Option<Child>>> = LazyLock::new(|| Mutex::new(None));

mod clipboard;
mod config;
mod gemini;
mod hotkey;
mod startup;
mod ui;

fn print_help() {
    println!("Clipboard Translator - Ctrl+C+C で翻訳");
    println!();
    println!("使い方:");
    println!("  clipboard-translator            通常起動（バックグラウンド）");
    println!("  clipboard-translator --setup    設定画面を開く");
    println!("  clipboard-translator --install  スタートアップに登録");
    println!("  clipboard-translator --uninstall スタートアップから削除");
    println!("  clipboard-translator --help     このヘルプを表示");
    println!();
    println!("設定ファイルの場所:");
    if let Ok(path) = config::config_path() {
        println!("  {}", path.display());
    }
    println!();
    println!("スタートアップ登録状態: {}", if startup::is_installed() { "登録済み" } else { "未登録" });
}

fn spawn_result_process(clipboard_text: &str) -> Result<()> {
    // 既存の結果ウィンドウプロセスを終了
    if let Ok(mut guard) = RESULT_PROCESS.lock() {
        if let Some(mut old_process) = guard.take() {
            let _ = old_process.kill();
            let _ = old_process.wait();
        }
    }

    let exe_path = env::current_exe()?;

    let mut child = Command::new(exe_path)
        .arg("--translate")
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(clipboard_text.as_bytes())?;
    }

    // 新しいプロセスを保存
    if let Ok(mut guard) = RESULT_PROCESS.lock() {
        *guard = Some(child);
    }

    Ok(())
}

fn run_translate_mode() -> Result<()> {
    // 標準入力からクリップボードテキストを読み取り
    let mut clipboard_text = String::new();
    io::stdin().read_to_string(&mut clipboard_text)?;

    // 設定読み込み
    let config = config::load_or_create()?;

    // チャンネル作成
    let (tx, rx) = mpsc::channel::<Result<String, String>>();

    // バックグラウンドでAPI呼び出し
    let api_key = config.api_key.clone();
    let model = config.model.clone();
    let output_mode = config.output_mode;

    thread::spawn(move || {
        let rt = match Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                let _ = tx.send(Err(format!("Tokioランタイム作成失敗: {}", e)));
                return;
            }
        };
        let client = gemini::GeminiClient::new(api_key, model, output_mode);

        let result = rt.block_on(async { client.translate_and_explain(&clipboard_text).await });

        let _ = tx.send(result.map_err(|e| e.to_string()));
    });

    // ローディング表示付きのウィンドウを表示
    ui::result::show_result_with_receiver(rx)?;

    Ok(())
}

fn main() -> Result<()> {
    // コマンドライン引数をチェック
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--setup" | "-s" | "--config" => {
                ui::setup::show_setup_window()?;
                return Ok(());
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--install" => {
                match startup::install_startup() {
                    Ok(_) => println!("スタートアップに登録しました"),
                    Err(e) => eprintln!("スタートアップ登録に失敗: {}", e),
                }
                return Ok(());
            }
            "--uninstall" => {
                match startup::uninstall_startup() {
                    Ok(_) => println!("スタートアップから削除しました"),
                    Err(e) => eprintln!("スタートアップ削除に失敗: {}", e),
                }
                return Ok(());
            }
            "--translate" => {
                // 翻訳モード：クリップボードテキストを受け取り、API呼び出し、結果表示
                return run_translate_mode();
            }
            "--show-result" => {
                // 旧API（後方互換）
                let mut content = String::new();
                io::stdin().read_to_string(&mut content)?;
                ui::result::show_result(&content)?;
                return Ok(());
            }
            _ => {
                println!("不明なオプション: {}", args[1]);
                print_help();
                return Ok(());
            }
        }
    }

    // 設定読み込み
    let config = config::load_or_create()?;

    // APIキー未設定の場合は設定画面を表示
    if config.api_key.is_empty() {
        ui::setup::show_setup_window()?;
        return Ok(());
    }

    // ホットキー監視ループ
    println!(
        "Clipboard Translator started. Model: {}. Press Ctrl+C+C to translate.",
        config.model
    );

    loop {
        if hotkey::is_ctrl_c_c_pressed() {
            // クリップボード取得
            match clipboard::get_text() {
                Ok(text) if !text.trim().is_empty() => {
                    println!("Detected Ctrl+C+C. Processing clipboard content...");

                    // 別プロセスで翻訳とウィンドウ表示を実行
                    if let Err(e) = spawn_result_process(&text) {
                        eprintln!("Failed to spawn result process: {}", e);
                    }
                }
                Ok(_) => {} // 空のクリップボードは無視
                Err(e) => eprintln!("Clipboard error: {}", e),
            }
        }

        thread::sleep(Duration::from_millis(HOTKEY_POLL_INTERVAL_MS));
    }
}
