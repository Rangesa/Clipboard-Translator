use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const API_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

/// APIリクエストのタイムアウト（秒）
const API_TIMEOUT_SECS: u64 = 30;

/// タイムアウト付きのHTTPクライアントを作成
fn create_client() -> Result<Client> {
    Client::builder()
        .timeout(Duration::from_secs(API_TIMEOUT_SECS))
        .build()
        .context("HTTPクライアントの作成に失敗しました")
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
struct Part {
    text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiResponse {
    #[serde(default)]
    candidates: Vec<Candidate>,
    #[serde(default)]
    prompt_feedback: Option<PromptFeedback>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PromptFeedback {
    #[serde(default)]
    block_reason: Option<String>,
    #[serde(default)]
    safety_ratings: Vec<SafetyRating>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SafetyRating {
    category: String,
    #[serde(default)]
    #[allow(dead_code)]
    probability: Option<String>,
    #[serde(default)]
    blocked: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Candidate {
    #[serde(default)]
    content: Option<ResponseContent>,
    #[serde(default)]
    finish_reason: Option<String>,
    #[serde(default)]
    safety_ratings: Vec<SafetyRating>,
}

#[derive(Debug, Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Debug, Deserialize)]
struct ResponsePart {
    text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListModelsResponse {
    models: Vec<ModelInfo>,
    #[serde(default)]
    #[allow(dead_code)]
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub name: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub description: String,
    #[serde(default)]
    pub supported_generation_methods: Vec<String>,
}

impl ModelInfo {
    pub fn model_id(&self) -> &str {
        self.name.strip_prefix("models/").unwrap_or(&self.name)
    }

    pub fn supports_generate_content(&self) -> bool {
        self.supported_generation_methods
            .iter()
            .any(|m| m == "generateContent")
    }
}

pub async fn fetch_available_models(api_key: &str) -> Result<Vec<ModelInfo>> {
    let client = create_client()?;
    let url = format!("{}?key={}&pageSize=100", API_BASE_URL, api_key);

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to fetch models list")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("API Error {}: {}", status, error_text);
    }

    let list_response: ListModelsResponse = response
        .json()
        .await
        .context("Failed to parse models list")?;

    // generateContent をサポートするモデルのみフィルタ
    let models: Vec<ModelInfo> = list_response
        .models
        .into_iter()
        .filter(|m| m.supports_generate_content())
        .collect();

    Ok(models)
}

use crate::config::OutputMode;

pub struct GeminiClient {
    api_key: String,
    model: String,
    output_mode: OutputMode,
    client: Client,
}

impl GeminiClient {
    pub fn new(api_key: String, model: String, output_mode: OutputMode) -> Self {
        let client = create_client().unwrap_or_else(|_| Client::new());
        Self {
            api_key,
            model,
            output_mode,
            client,
        }
    }

    fn build_prompt(&self, text: &str) -> String {
        match self.output_mode {
            OutputMode::Detailed => format!(
                r#"以下のテキストを分析し、以下の形式で回答してください:

【言語判定】
検出言語: [言語名]

【翻訳】
[日本語の場合は英語へ、それ以外は日本語へ翻訳]

【スラング・特殊表現】
[該当する表現があれば解説、なければ「なし」]

【要約】
[テキストの要点を1-2文で]

---
テキスト:
{}"#,
                text
            ),
            OutputMode::Concise => format!(
                r#"以下のテキストを翻訳してください。
- 日本語なら英語へ、それ以外なら日本語へ
- 5行以内で要点のみ
- 余計な説明不要、翻訳結果だけ出力

テキスト:
{}"#,
                text
            ),
        }
    }

    pub async fn translate_and_explain(&self, text: &str) -> Result<String> {
        let prompt = self.build_prompt(text);

        let request_body = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part { text: prompt }],
            }],
        };

        let url = format!(
            "{}/{}:generateContent?key={}",
            API_BASE_URL, self.model, self.api_key
        );

        // リトライ設定
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY_MS: u64 = 1000;

        let mut last_error = String::new();

        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    RETRY_DELAY_MS * (attempt as u64 + 1),
                ))
                .await;
            }

            let response = match self.client.post(&url).json(&request_body).send().await {
                Ok(r) => r,
                Err(e) => {
                    last_error = e.to_string();
                    continue;
                }
            };

            let status = response.status();

            if status.is_success() {
                let gemini_response: GeminiResponse = response
                    .json()
                    .await
                    .context("Failed to parse Gemini response")?;

                // プロンプト自体がブロックされた場合
                if let Some(ref feedback) = gemini_response.prompt_feedback {
                    if let Some(ref reason) = feedback.block_reason {
                        let blocked_categories: Vec<&str> = feedback
                            .safety_ratings
                            .iter()
                            .filter(|r| r.blocked == Some(true))
                            .map(|r| r.category.as_str())
                            .collect();

                        let detail = if blocked_categories.is_empty() {
                            reason.clone()
                        } else {
                            format!("{} ({})", reason, blocked_categories.join(", "))
                        };

                        anyhow::bail!(
                            "コンテンツがブロックされました: {}\n\
                            入力テキストがGeminiの安全性ポリシーに抵触した可能性があります。",
                            detail
                        );
                    }
                }

                // candidatesが空の場合
                let candidate = gemini_response.candidates.first().ok_or_else(|| {
                    anyhow::anyhow!(
                        "APIからの応答が空です。\n\
                        サーバー側で処理できなかった可能性があります。"
                    )
                })?;

                // finishReasonのチェック
                if let Some(ref reason) = candidate.finish_reason {
                    match reason.as_str() {
                        "STOP" => {} // 正常終了
                        "SAFETY" => {
                            let blocked_categories: Vec<&str> = candidate
                                .safety_ratings
                                .iter()
                                .filter(|r| r.blocked == Some(true))
                                .map(|r| r.category.as_str())
                                .collect();

                            anyhow::bail!(
                                "安全性フィルターにより応答がブロックされました。\n\
                                カテゴリ: {}",
                                if blocked_categories.is_empty() {
                                    "不明".to_string()
                                } else {
                                    blocked_categories.join(", ")
                                }
                            );
                        }
                        "MAX_TOKENS" => {
                            // 途中で切れても返す（警告付き）
                            if let Some(ref content) = candidate.content {
                                if let Some(part) = content.parts.first() {
                                    return Ok(format!(
                                        "{}\n\n---\n[警告: 出力がトークン上限に達したため途中で切れています]",
                                        part.text
                                    ));
                                }
                            }
                            anyhow::bail!("トークン上限に達しましたが、応答内容がありません。");
                        }
                        "RECITATION" => {
                            anyhow::bail!(
                                "著作権保護により応答が制限されました。\n\
                                入力テキストに著作権で保護されたコンテンツが含まれている可能性があります。"
                            );
                        }
                        other => {
                            anyhow::bail!("予期しない終了理由: {}", other);
                        }
                    }
                }

                // 正常なレスポンス抽出
                let content = candidate.content.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("応答にコンテンツが含まれていません。")
                })?;

                let text = content.parts.first().ok_or_else(|| {
                    anyhow::anyhow!("応答コンテンツが空です。")
                })?;

                return Ok(text.text.clone());
            }

            // 503 または 429 はリトライ対象
            if status.as_u16() == 503 || status.as_u16() == 429 {
                last_error = format!("API Error {}: サーバー過負荷、リトライ中...", status);
                continue;
            }

            // その他のエラーは即座に失敗
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API Error {}: {}", status, error_text);
        }

        anyhow::bail!(
            "API呼び出しに失敗しました（{}回リトライ）: {}",
            MAX_RETRIES,
            last_error
        )
    }
}
