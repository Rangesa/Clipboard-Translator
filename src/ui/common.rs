use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

/// 日本語フォントのパス (Windows)
const JAPANESE_FONT_PATH: &str = "C:\\Windows\\Fonts\\meiryo.ttc";

/// 日本語フォントを設定する
pub fn setup_japanese_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    if let Ok(font_data) = std::fs::read(JAPANESE_FONT_PATH) {
        fonts.font_data.insert(
            "meiryo".to_owned(),
            FontData::from_owned(font_data).into(),
        );

        if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
            family.insert(0, "meiryo".to_owned());
        }

        if let Some(family) = fonts.families.get_mut(&FontFamily::Monospace) {
            family.insert(0, "meiryo".to_owned());
        }
    }

    ctx.set_fonts(fonts);
}
