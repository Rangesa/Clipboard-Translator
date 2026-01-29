use winrt_notification::{Toast, Duration};

const APP_ID: &str = "ClipboardTranslator";

/// エラー通知を表示
pub fn show_error(title: &str, message: &str) {
    if let Err(e) = Toast::new(APP_ID)
        .title(title)
        .text1(message)
        .duration(Duration::Short)
        .show()
    {
        eprintln!("Failed to show notification: {}", e);
        eprintln!("{}: {}", title, message);
    }
}

/// 成功通知を表示
pub fn show_success(title: &str, message: &str) {
    if let Err(e) = Toast::new(APP_ID)
        .title(title)
        .text1(message)
        .duration(Duration::Short)
        .show()
    {
        eprintln!("Failed to show notification: {}", e);
        eprintln!("{}: {}", title, message);
    }
}

/// 情報通知を表示
pub fn show_info(message: &str) {
    if let Err(e) = Toast::new(APP_ID)
        .text1(message)
        .duration(Duration::Short)
        .show()
    {
        eprintln!("Failed to show notification: {}", e);
        eprintln!("{}", message);
    }
}
