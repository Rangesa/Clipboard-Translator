use anyhow::Result;
use eframe::egui;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use super::common::setup_japanese_fonts;
use crate::config::{self, Config, Hotkey, OutputMode, DEFAULT_MODEL, FALLBACK_MODELS};
use crate::gemini::{fetch_available_models, ModelInfo};

/// egui::KeyをWindows VKコードに変換
fn key_to_vk_code(key: egui::Key) -> Option<i32> {
    use egui::Key;
    match key {
        Key::A => Some(0x41),
        Key::B => Some(0x42),
        Key::C => Some(0x43),
        Key::D => Some(0x44),
        Key::E => Some(0x45),
        Key::F => Some(0x46),
        Key::G => Some(0x47),
        Key::H => Some(0x48),
        Key::I => Some(0x49),
        Key::J => Some(0x4A),
        Key::K => Some(0x4B),
        Key::L => Some(0x4C),
        Key::M => Some(0x4D),
        Key::N => Some(0x4E),
        Key::O => Some(0x4F),
        Key::P => Some(0x50),
        Key::Q => Some(0x51),
        Key::R => Some(0x52),
        Key::S => Some(0x53),
        Key::T => Some(0x54),
        Key::U => Some(0x55),
        Key::V => Some(0x56),
        Key::W => Some(0x57),
        Key::X => Some(0x58),
        Key::Y => Some(0x59),
        Key::Z => Some(0x5A),
        Key::Num0 => Some(0x30),
        Key::Num1 => Some(0x31),
        Key::Num2 => Some(0x32),
        Key::Num3 => Some(0x33),
        Key::Num4 => Some(0x34),
        Key::Num5 => Some(0x35),
        Key::Num6 => Some(0x36),
        Key::Num7 => Some(0x37),
        Key::Num8 => Some(0x38),
        Key::Num9 => Some(0x39),
        Key::F1 => Some(0x70),
        Key::F2 => Some(0x71),
        Key::F3 => Some(0x72),
        Key::F4 => Some(0x73),
        Key::F5 => Some(0x74),
        Key::F6 => Some(0x75),
        Key::F7 => Some(0x76),
        Key::F8 => Some(0x77),
        Key::F9 => Some(0x78),
        Key::F10 => Some(0x79),
        Key::F11 => Some(0x7A),
        Key::F12 => Some(0x7B),
        _ => None,
    }
}

enum ModelLoadState {
    NotLoaded,
    Loading,
    Loaded(Vec<ModelInfo>),
    Error(String),
}

struct SetupApp {
    api_key: String,
    selected_model_id: String,
    output_mode: OutputMode,
    hotkey: Hotkey,
    listening_for_hotkey: bool,
    models: ModelLoadState,
    model_receiver: Option<Receiver<Result<Vec<ModelInfo>, String>>>,
    error_message: Option<String>,
    api_key_validated: bool,
    saved: bool,
}

impl SetupApp {
    fn new() -> Self {
        let (api_key, selected_model_id, output_mode, hotkey) = match config::load_or_create() {
            Ok(cfg) => (cfg.api_key, cfg.model, cfg.output_mode, cfg.hotkey),
            Err(_) => (String::new(), DEFAULT_MODEL.to_string(), OutputMode::default(), Hotkey::default()),
        };

        Self {
            api_key,
            selected_model_id,
            output_mode,
            hotkey,
            listening_for_hotkey: false,
            models: ModelLoadState::NotLoaded,
            model_receiver: None,
            error_message: None,
            api_key_validated: false,
            saved: false,
        }
    }

    fn start_model_fetch(&mut self) {
        if self.api_key.trim().is_empty() {
            self.error_message = Some("APIキーを入力してください".to_string());
            return;
        }

        self.models = ModelLoadState::Loading;
        self.error_message = None;
        self.api_key_validated = false;

        let (tx, rx): (
            Sender<Result<Vec<ModelInfo>, String>>,
            Receiver<Result<Vec<ModelInfo>, String>>,
        ) = mpsc::channel();
        self.model_receiver = Some(rx);

        let api_key = self.api_key.clone();

        thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = tx.send(Err(format!("ランタイム作成失敗: {}", e)));
                    return;
                }
            };
            let result = rt.block_on(fetch_available_models(&api_key));

            let _ = tx.send(result.map_err(|e| e.to_string()));
        });
    }

    fn check_model_fetch(&mut self) {
        if let Some(ref rx) = self.model_receiver {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(models) => {
                        if models.is_empty() {
                            self.models = ModelLoadState::Error(
                                "利用可能なモデルが見つかりません".to_string(),
                            );
                        } else {
                            // APIキーが有効であることが確認された
                            self.api_key_validated = true;
                            // 現在選択されているモデルが一覧にあるか確認
                            let exists = models
                                .iter()
                                .any(|m| m.model_id() == self.selected_model_id);
                            if !exists && !models.is_empty() {
                                self.selected_model_id = models[0].model_id().to_string();
                            }
                            self.models = ModelLoadState::Loaded(models);
                        }
                    }
                    Err(e) => {
                        self.models = ModelLoadState::Error(e);
                    }
                }
                self.model_receiver = None;
            }
        }
    }

    fn get_fallback_models(&self) -> Vec<String> {
        FALLBACK_MODELS.iter().map(|s| s.to_string()).collect()
    }
}

impl eframe::App for SetupApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ホットキー入力待ちの場合、キーを記録
        if self.listening_for_hotkey {
            ctx.input(|i| {
                for event in &i.events {
                    if let egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } = event
                    {
                        // egui::Keyをi32のVKコードに変換
                        if let Some(vk_code) = key_to_vk_code(*key) {
                            self.hotkey = Hotkey {
                                ctrl: modifiers.ctrl,
                                alt: modifiers.alt,
                                shift: modifiers.shift,
                                key_code: vk_code,
                                is_double_press: false, // 手動設定時はシングルプレス
                            };
                            self.listening_for_hotkey = false;
                        }
                    }
                }
            });
        }

        // モデル取得の完了をチェック
        self.check_model_fetch();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Clipboard Translator - 設定");
            ui.add_space(20.0);

            ui.label("Google AI Studio で取得した Gemini API キーを入力してください:");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("APIキー:");
                let response =
                    ui.add(egui::TextEdit::singleline(&mut self.api_key).desired_width(300.0));

                if ui.button("モデル取得").clicked() {
                    self.start_model_fetch();
                }

                // フォーカスを外したときも取得開始
                if response.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && matches!(self.models, ModelLoadState::NotLoaded)
                {
                    self.start_model_fetch();
                }
            });

            ui.add_space(10.0);

            // モデル選択
            ui.horizontal(|ui| {
                ui.label("モデル:");

                match &self.models {
                    ModelLoadState::NotLoaded => {
                        ui.label("(APIキー入力後「モデル取得」を押してください)");
                    }
                    ModelLoadState::Loading => {
                        ui.spinner();
                        ui.label("モデル一覧を取得中...");
                    }
                    ModelLoadState::Loaded(models) => {
                        let selected_display = models
                            .iter()
                            .find(|m| m.model_id() == self.selected_model_id)
                            .map(|m| m.display_name.clone())
                            .unwrap_or_else(|| self.selected_model_id.clone());

                        egui::ComboBox::from_id_salt("model_selector")
                            .selected_text(&selected_display)
                            .width(300.0)
                            .show_ui(ui, |ui| {
                                for model in models {
                                    let label = if model.display_name.is_empty() {
                                        model.model_id().to_string()
                                    } else {
                                        format!(
                                            "{} ({})",
                                            model.display_name,
                                            model.model_id()
                                        )
                                    };
                                    let model_id = model.model_id().to_string();
                                    ui.selectable_value(
                                        &mut self.selected_model_id,
                                        model_id,
                                        label,
                                    );
                                }
                            });
                    }
                    ModelLoadState::Error(err) => {
                        ui.colored_label(egui::Color32::YELLOW, format!("取得失敗: {}", err));

                        // フォールバックモデルを表示
                        let fallback = self.get_fallback_models();
                        egui::ComboBox::from_id_salt("model_selector_fallback")
                            .selected_text(&self.selected_model_id)
                            .show_ui(ui, |ui| {
                                for model in &fallback {
                                    ui.selectable_value(
                                        &mut self.selected_model_id,
                                        model.clone(),
                                        model,
                                    );
                                }
                            });
                    }
                }
            });

            // APIキー検証成功メッセージ
            if self.api_key_validated {
                ui.add_space(5.0);
                ui.colored_label(egui::Color32::GREEN, "APIキーは有効です");
            }

            ui.add_space(15.0);

            // 出力モード選択
            ui.horizontal(|ui| {
                ui.label("出力モード:");
                egui::ComboBox::from_id_salt("output_mode_selector")
                    .selected_text(self.output_mode.label())
                    .width(300.0)
                    .show_ui(ui, |ui| {
                        for mode in OutputMode::all() {
                            ui.selectable_value(&mut self.output_mode, *mode, mode.label());
                        }
                    });
            });

            ui.add_space(15.0);

            // ホットキー設定
            ui.horizontal(|ui| {
                ui.label("ホットキー:");
                let hotkey_text = if self.listening_for_hotkey {
                    "キーを押してください...".to_string()
                } else {
                    self.hotkey.to_string()
                };

                if ui.button(&hotkey_text).clicked() {
                    self.listening_for_hotkey = true;
                }

                if ui.button("リセット").clicked() {
                    self.hotkey = Hotkey::default();
                    self.listening_for_hotkey = false;
                }
            });

            ui.horizontal(|ui| {
                ui.add_space(80.0);
                ui.checkbox(&mut self.hotkey.is_double_press, "ダブルプレス（例: Ctrl+C+C）");
            });

            ui.add_space(10.0);
            ui.hyperlink_to(
                "Google AI Studio でAPIキーを取得",
                "https://aistudio.google.com/app/apikey",
            );

            ui.add_space(20.0);

            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::RED, error);
                ui.add_space(10.0);
            }

            if self.saved {
                ui.colored_label(
                    egui::Color32::GREEN,
                    "設定を保存しました。アプリケーションを再起動してください。",
                );
                ui.add_space(10.0);

                if ui.button("閉じる").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            } else {
                ui.horizontal(|ui| {
                    if ui.button("保存").clicked() {
                        if self.api_key.trim().is_empty() {
                            self.error_message = Some("APIキーを入力してください".to_string());
                        } else {
                            let config = Config {
                                api_key: self.api_key.clone(),
                                model: self.selected_model_id.clone(),
                                output_mode: self.output_mode,
                                hotkey: self.hotkey,
                            };

                            match config::save(&config) {
                                Ok(_) => {
                                    self.saved = true;
                                    self.error_message = None;
                                }
                                Err(e) => {
                                    self.error_message = Some(format!("保存エラー: {}", e));
                                }
                            }
                        }
                    }

                    if ui.button("キャンセル").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            }
        });

        // ローディング中は定期的に再描画
        if matches!(self.models, ModelLoadState::Loading) {
            ctx.request_repaint();
        }
    }
}

pub fn show_setup_window() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([550.0, 400.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Clipboard Translator Setup",
        options,
        Box::new(|cc| {
            setup_japanese_fonts(&cc.egui_ctx);
            Ok(Box::new(SetupApp::new()))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run setup window: {}", e))?;

    Ok(())
}
