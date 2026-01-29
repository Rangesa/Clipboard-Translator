use anyhow::Result;
use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

use super::common::setup_japanese_fonts;

enum ContentState {
    Loading,
    Ready(String),
    Error(String),
}

struct ResultApp {
    state: ContentState,
    receiver: Option<Receiver<Result<String, String>>>,
    markdown_cache: CommonMarkCache,
    is_translating: Option<Arc<AtomicBool>>,
}

impl eframe::App for ResultApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 結果を受信チェック
        if let Some(ref rx) = self.receiver {
            match rx.try_recv() {
                Ok(result) => {
                    match result {
                        Ok(content) => {
                            self.state = ContentState::Ready(content);
                            // 翻訳完了、フラグをクリア
                            if let Some(ref flag) = self.is_translating {
                                flag.store(false, Ordering::SeqCst);
                            }
                        }
                        Err(e) => {
                            // トースト通知でもエラーを表示
                            crate::notification::show_error("API エラー", &e);
                            self.state = ContentState::Error(e);
                            // エラーでもフラグをクリア
                            if let Some(ref flag) = self.is_translating {
                                flag.store(false, Ordering::SeqCst);
                            }
                        }
                    }
                    self.receiver = None;
                }
                Err(TryRecvError::Empty) => {
                    // まだ結果がない、再描画を要求
                    ctx.request_repaint();
                }
                Err(TryRecvError::Disconnected) => {
                    self.state = ContentState::Error("接続が切断されました".to_string());
                    self.receiver = None;
                    // エラーでもフラグをクリア
                    if let Some(ref flag) = self.is_translating {
                        flag.store(false, Ordering::SeqCst);
                    }
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.state {
                ContentState::Loading => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(150.0);
                        ui.spinner();
                        ui.add_space(10.0);
                        ui.label("翻訳中...");
                    });
                }
                ContentState::Ready(content) => {
                    egui::ScrollArea::vertical()
                        .max_height(550.0)
                        .show(ui, |ui| {
                            CommonMarkViewer::new().show(ui, &mut self.markdown_cache, content);
                        });

                    ui.add_space(10.0);

                    if ui.button("閉じる").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
                ContentState::Error(error) => {
                    ui.colored_label(egui::Color32::RED, format!("エラー: {}", error));
                    ui.add_space(10.0);

                    if ui.button("閉じる").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            }
        });

        // Escキーで閉じる
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn get_cursor_position() -> (f32, f32) {
    unsafe {
        let mut point = POINT { x: 0, y: 0 };
        let _ = GetCursorPos(&mut point);
        (point.x as f32, point.y as f32)
    }
}

pub fn show_result_with_receiver(
    receiver: Receiver<Result<String, String>>,
    is_translating: Option<Arc<AtomicBool>>,
) -> Result<()> {
    let (cursor_x, cursor_y) = get_cursor_position();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 400.0])
            .with_position([cursor_x + 20.0, cursor_y - 10.0])
            .with_always_on_top()
            .with_resizable(true),
        ..Default::default()
    };

    let result_app = ResultApp {
        state: ContentState::Loading,
        receiver: Some(receiver),
        markdown_cache: CommonMarkCache::default(),
        is_translating,
    };

    eframe::run_native(
        "Translation Result",
        options,
        Box::new(|cc| {
            setup_japanese_fonts(&cc.egui_ctx);
            Ok(Box::new(result_app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run result window: {}", e))?;

    Ok(())
}

// 旧API（後方互換のため残す）
pub fn show_result(content: &str) -> Result<()> {
    let (tx, rx) = mpsc::channel();
    let _ = tx.send(Ok(content.to_string()));
    show_result_with_receiver(rx, None)
}
