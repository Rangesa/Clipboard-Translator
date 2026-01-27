# Clipboard Translator

[![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg?logo=rust)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows-0078D6.svg?logo=windows)](https://www.microsoft.com/windows/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Windows作業を加速する、AI搭載クリップボード翻訳ツール**

`Ctrl+C` を2回押すだけで、クリップボードの内容を **Google Gemini API** が瞬時に解析・翻訳し、ポップアップで表示します。単なる翻訳だけでなく、スラングの解説や要約も可能な強力なデスクトップアシスタントです。

---

<!-- ここにデモ画像やGIFを配置すると非常に効果的です -->
<!-- ![Demo Screenshot](docs/demo.png) -->

## 主な機能

- **完全カスタマイズ可能なホットキー**: 好きなキーコンビネーションを自由に設定
  - 設定画面でボタンクリック→実際にキーを押すだけで記録
  - Ctrl、Alt、Shiftと任意のキー（A-Z、0-9、F1-F12など）を組み合わせ可能
  - デフォルトは `Ctrl+C`
  - ウィンドウを切り替える必要はありません
- **AIによる高度な解析**: Google Gemini Proモデルを使用し、文脈を理解した翻訳を実現。
- **スマートな双方向翻訳**:
  - **日本語** → 英語へ翻訳
  - **その他** → 日本語へ翻訳
- **選べる2つのモード**:
  - **詳細モード**: 言語判定・翻訳・文化的背景やスラングの解説・要約をフルセットで。
  - **簡潔モード**: 忙しい時向け。要点のみを5行以内でサッと表示。
- **軽量 & 高速**: Rust + egui で構築され、シングルプロセス方式により起動オーバーヘッドを削減。メモリ使用量も少なく動作も軽快です。
- **常駐 & 自動起動**: タスクトレイに常駐し、Windows起動時に自動で立ち上がる設定も可能。
- **セキュリティ**:
  - APIキーをWindows Credential Managerに安全に保存
  - シングルインスタンス制限で多重起動を防止

## インストール

### 方法 1: ソースコードからビルド

Rust環境（Cargo）が必要です。

```bash
# リポジトリをクローン
git clone https://github.com/YOUR_USERNAME/clipboard-translator.git
cd clipboard-translator

# リリースビルドを実行
cargo build --release
```

生成された `target/release/clipboard-translator.exe` を任意のフォルダに配置してください。

## 初期セットアップ

初めて起動する場合、以下の手順でAPIキーの設定が必要です。

1. アプリを起動（またはコマンドラインで `--setup`）します。
2. 設定ウィンドウが開きます。
3. [Google AI Studio](https://aistudio.google.com/app/apikey) で **Gemini APIキー** を取得します（無料枠あり）。
4. 設定画面にAPIキーを貼り付け、**「モデル取得」** をクリックして通信テストを行います。
5. 使用するモデル（`gemini-2.0-flash` 等）と出力モードを選択します。
6. **ホットキーを設定**: ボタンをクリックして、実際に使いたいキーの組み合わせを押します（例: `Ctrl+Shift+T`）。
7. **「保存」** をクリックします。

## 使い方

### 基本操作

1. 翻訳したいテキストを選択してコピーします（<kbd>Ctrl</kbd> + <kbd>C</kbd>）。
2. 設定したホットキーを押します（デフォルトは <kbd>Ctrl</kbd> + <kbd>C</kbd>）。
3. マウスカーソルのそばに翻訳結果がポップアップ表示されます。
   - <kbd>ESC</kbd> キー、またはウィンドウ外をクリックすると閉じます。

### コマンドラインオプション

```bash
clipboard-translator.exe [OPTIONS]

Options:
  --setup      設定画面を強制的に開く
  --install    Windowsのスタートアップに登録（自動起動）
  --uninstall  スタートアップから登録解除
  --help       ヘルプを表示
```

### 設定ファイルの場所

**設定ファイル:** `%APPDATA%\ClipboardTranslator\config.json`
（モデル名、出力モード、ホットキー設定）

**APIキー:** Windows Credential Manager
（`コントロールパネル` → `資格情報マネージャー` → `Windows資格情報` → `ClipboardTranslator_APIKey`）

## 技術スタック

- **言語**: [Rust](https://www.rust-lang.org/) (2021 Edition)
- **GUI**: [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
- **非同期処理**: [Tokio](https://tokio.rs/)
- **AI API**: Google Gemini API
- **クリップボード監視**: arboard / device_query

## ライセンス

このプロジェクトは [MIT License](./LICENSE) の元で公開されています。