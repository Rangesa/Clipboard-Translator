# Changelog

## [0.1.1] - 2025-01-15

### Changed

- **コード品質改善**
  - フォント設定を `ui/common.rs` に統合し、重複コードを削除
  - マジックナンバーを定数化（`DOUBLE_PRESS_WINDOW_MS`, `DEBOUNCE_DELAY_MS`, `HOTKEY_POLL_INTERVAL_MS`）
  - `unwrap()` をより安全なエラーハンドリングに置換（Mutex lock、Tokio Runtime作成時）

### Added

- **APIキー検証表示**: モデル取得成功時に「APIキーは有効です」と表示
- **APIタイムアウト**: 30秒のタイムアウトを設定（`API_TIMEOUT_SECS`）

### Fixed

- Mutex poisoning時のパニックを防止
- Tokio Runtime作成失敗時に適切なエラーメッセージを返すように修正

## [0.1.0] - 初回リリース

### Features

- Ctrl+C+C ホットキーによるクリップボード翻訳
- Google Gemini API 連携
- 詳細/簡潔の2つの出力モード
- 設定GUI（APIキー、モデル選択、出力モード）
- Windowsスタートアップ登録/解除
- 日本語フォント対応（メイリオ）
- マークダウン形式での結果表示
