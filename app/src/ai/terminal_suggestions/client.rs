use anyhow::{Result, anyhow};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::TerminalSuggestionsConfig;

#[derive(Clone)]
pub struct OpenAICompatibleClient {
    http: reqwest::Client,
}

impl Default for OpenAICompatibleClient {
    fn default() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub role: &'static str,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
    content: String,
}

pub fn completion_url(endpoint: &str) -> Result<Url> {
    let trimmed = endpoint.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(anyhow!("OpenAI-compatible endpoint is empty"));
    }

    let url = if trimmed.ends_with("/chat/completions") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/chat/completions")
    };

    Url::parse(&url).map_err(|err| anyhow!("Invalid OpenAI-compatible endpoint: {err}"))
}

pub fn strip_json_fence(raw: &str) -> &str {
    let trimmed = raw.trim();
    let Some(without_start) = trimmed.strip_prefix("```") else {
        return trimmed;
    };
    let without_language = without_start
        .strip_prefix("json")
        .unwrap_or(without_start)
        .trim_start_matches('\n')
        .trim();
    without_language
        .strip_suffix("```")
        .unwrap_or(without_language)
        .trim()
}

pub fn parse_json_object(raw: &str) -> Result<Value> {
    let value: Value = serde_json::from_str(strip_json_fence(raw))?;
    if !value.is_object() {
        return Err(anyhow!(
            "OpenAI-compatible response content is not a JSON object"
        ));
    }
    Ok(value)
}

impl OpenAICompatibleClient {
    pub async fn complete_json(
        &self,
        config: &TerminalSuggestionsConfig,
        system: String,
        user: String,
    ) -> Result<Value> {
        let url = completion_url(&config.endpoint)?;
        let request = ChatCompletionRequest {
            model: config.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: system,
                },
                ChatMessage {
                    role: "user",
                    content: user,
                },
            ],
            temperature: 0.0,
            reasoning_effort: config.effort.config_value().map(str::to_string),
        };

        log::debug!(
            "[terminal-suggestions] sending chat completion url={} model={} effort={:?} system_bytes={} user_bytes={}",
            url,
            request.model,
            request.reasoning_effort,
            request
                .messages
                .first()
                .map_or(0, |message| message.content.len()),
            request
                .messages
                .get(1)
                .map_or(0, |message| message.content.len()),
        );

        let mut builder = self.http.post(url.clone()).json(&request);
        if !config.api_key.is_empty() {
            builder = builder.bearer_auth(&config.api_key);
        }

        let response = builder.send().await?;
        let status = response.status();
        let body = response.text().await?;
        log::debug!(
            "[terminal-suggestions] received chat completion url={} status={} response_bytes={}",
            url,
            status,
            body.len(),
        );
        if !status.is_success() {
            log::warn!(
                "[terminal-suggestions] chat completion failed url={} status={} body={}",
                url,
                status,
                truncate_for_log(&body),
            );
            return Err(anyhow!(
                "OpenAI-compatible endpoint returned {status}: {body}"
            ));
        }

        let completion: ChatCompletionResponse = serde_json::from_str(&body)?;
        let content = completion
            .choices
            .first()
            .map(|choice| choice.message.content.as_str())
            .ok_or_else(|| anyhow!("OpenAI-compatible response has no choices"))?;

        log::debug!(
            "[terminal-suggestions] parsing chat completion content content_bytes={}",
            content.len(),
        );
        parse_json_object(content).map_err(|err| {
            log::warn!(
                "[terminal-suggestions] failed to parse chat completion content error={} content={}",
                err,
                truncate_for_log(content),
            );
            err
        })
    }
}

fn truncate_for_log(value: &str) -> String {
    const MAX_LOG_BYTES: usize = 2048;
    if value.len() <= MAX_LOG_BYTES {
        value.to_string()
    } else {
        format!("{}...[truncated]", &value[..MAX_LOG_BYTES])
    }
}
