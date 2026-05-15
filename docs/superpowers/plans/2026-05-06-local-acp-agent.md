# ACP Agent Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 Warp 改成单机 GUI：自然语言 Agent 入口只对接全局安装的 ACP adapter，Next Command 和 Prompt Suggestions 只对接用户配置的 OpenAI-compatible endpoint。

**Architecture:** Agent 主路径新增 `AcpAgentModel`，它只通过 ACP stdio JSON-RPC 启动 `codex-acp` 或 `claude-agent-acp`，不再进入 Warp server、AgentDriver、登录、用量、团队或云端 transcript。Suggestions 主路径新增 `local_suggestions`，复用现有上下文构造和 UI 展示，只把请求发送到本地设置里的 OpenAI-compatible `/chat/completions` endpoint。

**Tech Stack:** Rust 2021、Warp `Entity`/`ModelHandle`、`agent-client-protocol` 0.11.1、`agent-client-protocol-tokio` 0.11.1、`reqwest`、`serde`、现有 `settings` 宏、现有 terminal input 和 settings view。

---

## Scope Check

这个需求包含三个子系统：ACP Agent、OpenAI-compatible Suggestions、单机化 UI/启动面。它们共用同一个目标，且每个任务都能独立编译和测试，所以保留在一个计划内，但按阶段提交。不要实现向后兼容，不提供二进制路径覆盖，不接旧 Warp Agent server。

用户安装前置条件由 UI 提示，不由 Warp 自动安装：

```bash
npm i -g @zed-industries/codex-acp
npm i -g @agentclientprotocol/claude-agent-acp
```

## File Structure

**Create**
- `app/src/ai/acp/mod.rs`: ACP 模块出口。
- `app/src/ai/acp/backend.rs`: `AcpAgentBackend` 的 adapter 命令、展示名、安装命令。
- `app/src/ai/acp/events.rs`: Warp 内部使用的 ACP 事件 DTO。
- `app/src/ai/acp/model.rs`: `AcpAgentModel`，负责启动 adapter、创建 session、发送 prompt、转发 update。
- `app/src/ai/acp/mapping.rs`: ACP `SessionUpdate` 到 Warp 本地事件的纯映射。
- `app/src/ai/acp/tests.rs`: ACP backend 和 mapping 单元测试。
- `app/src/ai/local_suggestions/mod.rs`: Local Suggestions 模块出口。
- `app/src/ai/local_suggestions/client.rs`: OpenAI-compatible HTTP client。
- `app/src/ai/local_suggestions/provider.rs`: Next Command / Prompt Suggestions provider trait 和本地实现。
- `app/src/ai/local_suggestions/tests.rs`: payload、JSON parsing、mapping 单元测试。

**Modify**
- `Cargo.toml`: workspace dependency 增加 ACP crates。
- `app/Cargo.toml`: app dependency 增加 ACP crates。
- `app/src/ai/mod.rs`: 导出 `acp` 和 `local_suggestions`。
- `app/src/settings/ai.rs`: 增加本地 AI 设置和 getter。
- `app/src/settings/ai_tests.rs`: 本地设置默认值和本地-only 行为测试。
- `app/src/lib.rs`: 注册 `AcpAgentModel`。
- `app/src/terminal/input.rs`: `submit_ai_query` 改为发送到 ACP。
- `app/src/ai/predict/next_command_model.rs`: Next Command LLM fallback 改走 local suggestions provider。
- `app/src/ai/blocklist/passive_suggestions/legacy.rs`: Prompt Suggestions 改走 local suggestions provider，并关闭被 Warp Agent 绑定的 code diff/unit test suggestion。
- `app/src/settings_view/ai_page.rs`: 用 Local AI 设置 UI 替换 Warp Agent/Usage/BYOK/Bedrock/CLI Agent 旧 UI。
- `app/src/settings_view/mod.rs`: 隐藏 Account、Billing、Teams、Warp Drive、Cloud Platform、Referrals 入口。
- `app/src/root_view.rs`: 未登录也直接进入 terminal workspace。

---

### Task 1: Add ACP Dependencies and Local AI Settings

**Files:**
- Modify: `Cargo.toml`
- Modify: `app/Cargo.toml`
- Modify: `app/src/settings/ai.rs`
- Modify: `app/src/settings/ai_tests.rs`

- [ ] **Step 1: Write failing settings tests**

Append these tests to `app/src/settings/ai_tests.rs`:

```rust
#[test]
fn test_local_ai_settings_defaults() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);

        AISettings::handle(&app).read(&app, |settings, _ctx| {
            assert_eq!(
                *settings.acp_agent_backend,
                AcpAgentBackend::Codex
            );
            assert_eq!(settings.acp_model.as_str(), "");
            assert_eq!(*settings.acp_effort, LocalAiEffort::Medium);
            assert_eq!(settings.local_openai_endpoint.as_str(), "");
            assert_eq!(settings.local_openai_api_key.as_str(), "");
            assert_eq!(settings.local_openai_model.as_str(), "");
            assert_eq!(*settings.local_openai_effort, LocalAiEffort::Medium);
            assert!(*settings.local_next_command_enabled);
            assert!(*settings.local_prompt_suggestions_enabled);
        });
    });
}

#[test]
fn test_local_ai_getters_do_not_require_auth() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);

        AISettings::handle(&app).read(&app, |settings, _ctx| {
            assert!(settings.is_local_next_command_enabled());
            assert!(settings.is_local_prompt_suggestions_enabled());
            assert_eq!(
                settings.acp_agent_backend.adapter_command(),
                "codex-acp"
            );
        });
    });
}
```

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
cargo test -p warp settings::ai_tests::test_local_ai_settings_defaults settings::ai_tests::test_local_ai_getters_do_not_require_auth
```

Expected: fail with unresolved `AcpAgentBackend`, `LocalAiEffort`, and missing local settings fields.

- [ ] **Step 3: Add workspace dependencies**

In root `Cargo.toml`, add these under `[workspace.dependencies]`:

```toml
agent-client-protocol = "0.11.1"
agent-client-protocol-tokio = "0.11.1"
```

In `app/Cargo.toml`, add these under `[dependencies]`:

```toml
agent-client-protocol.workspace = true
agent-client-protocol-tokio.workspace = true
```

- [ ] **Step 4: Add local AI enums**

In `app/src/settings/ai.rs`, near `DefaultSessionMode`, add:

```rust
#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    EnumIter,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(description = "ACP agent backend.", rename_all = "snake_case")]
pub enum AcpAgentBackend {
    #[default]
    Codex,
    Claude,
}

settings::macros::implement_setting_for_enum!(
    AcpAgentBackend,
    AISettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Never,
    private: false,
    toml_path: "ai.acp.agent_backend",
    description: "The ACP agent backend.",
);

impl AcpAgentBackend {
    pub fn display_name(&self) -> &'static str {
        match self {
            AcpAgentBackend::Codex => "Codex",
            AcpAgentBackend::Claude => "Claude",
        }
    }

    pub fn adapter_command(&self) -> &'static str {
        match self {
            AcpAgentBackend::Codex => "codex-acp",
            AcpAgentBackend::Claude => "claude-agent-acp",
        }
    }

    pub fn install_command(&self) -> &'static str {
        match self {
            AcpAgentBackend::Codex => "npm i -g @zed-industries/codex-acp",
            AcpAgentBackend::Claude => "npm i -g @agentclientprotocol/claude-agent-acp",
        }
    }
}

#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    EnumIter,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(description = "Local AI effort level.", rename_all = "snake_case")]
pub enum LocalAiEffort {
    Low,
    #[default]
    Medium,
    High,
    XHigh,
}

settings::macros::implement_setting_for_enum!(
    LocalAiEffort,
    AISettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Never,
    private: false,
    toml_path: "local_ai.effort",
    description: "The local AI effort level.",
);

impl LocalAiEffort {
    pub fn display_name(&self) -> &'static str {
        match self {
            LocalAiEffort::Low => "Low",
            LocalAiEffort::Medium => "Medium",
            LocalAiEffort::High => "High",
            LocalAiEffort::XHigh => "XHigh",
        }
    }

    pub fn config_value(&self) -> &'static str {
        match self {
            LocalAiEffort::Low => "low",
            LocalAiEffort::Medium => "medium",
            LocalAiEffort::High => "high",
            LocalAiEffort::XHigh => "xhigh",
        }
    }
}
```

- [ ] **Step 5: Add local AI settings fields**

Inside `define_settings_group!(AISettings, settings: [ ... ])`, add this block before the existing `is_any_ai_enabled` field:

```rust
    acp_agent_backend: AcpAgentBackendSetting {
        type: AcpAgentBackend,
        default: AcpAgentBackend::Codex,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        private: false,
        toml_path: "ai.acp.agent_backend",
        description: "The ACP agent backend.",
    }
    acp_model: ACPModel {
        type: String,
        default: String::new(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        private: true,
        toml_path: "ai.acp.model",
        description: "The model selected for the ACP agent.",
    }
    acp_effort: ACPEffort {
        type: LocalAiEffort,
        default: LocalAiEffort::Medium,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        private: false,
        toml_path: "ai.acp.effort",
        description: "The effort level selected for the ACP agent.",
    }
    local_openai_endpoint: LocalOpenAIEndpoint {
        type: String,
        default: String::new(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        private: true,
        toml_path: "local_ai.suggestions.endpoint",
        description: "OpenAI-compatible endpoint for local suggestions.",
    }
    local_openai_api_key: LocalOpenAIAPIKey {
        type: String,
        default: String::new(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        private: true,
        toml_path: "local_ai.suggestions.api_key",
        description: "API key for the OpenAI-compatible suggestions endpoint.",
    }
    local_openai_model: LocalOpenAIModel {
        type: String,
        default: String::new(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        private: true,
        toml_path: "local_ai.suggestions.model",
        description: "Model for OpenAI-compatible local suggestions.",
    }
    local_openai_effort: LocalOpenAIEffort {
        type: LocalAiEffort,
        default: LocalAiEffort::Medium,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        private: false,
        toml_path: "local_ai.suggestions.effort",
        description: "Effort level for OpenAI-compatible local suggestions.",
    }
    local_next_command_enabled: LocalNextCommandEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        private: false,
        toml_path: "local_ai.suggestions.next_command_enabled",
        description: "Controls whether local Next Command suggestions are enabled.",
    }
    local_prompt_suggestions_enabled: LocalPromptSuggestionsEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        private: false,
        toml_path: "local_ai.suggestions.prompt_suggestions_enabled",
        description: "Controls whether local Prompt Suggestions are enabled.",
    }
```

- [ ] **Step 6: Add local getters**

In `impl AISettings`, add:

```rust
    pub fn is_local_next_command_enabled(&self) -> bool {
        *self.local_next_command_enabled
    }

    pub fn is_local_prompt_suggestions_enabled(&self) -> bool {
        *self.local_prompt_suggestions_enabled
    }
```

- [ ] **Step 7: Run tests and verify they pass**

Run:

```bash
cargo test -p warp settings::ai_tests::test_local_ai_settings_defaults settings::ai_tests::test_local_ai_getters_do_not_require_auth
```

Expected: both tests pass.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml app/Cargo.toml app/src/settings/ai.rs app/src/settings/ai_tests.rs
git commit -m "feat: add local ai settings"
```

---

### Task 2: Build the OpenAI-Compatible Suggestions Client

**Files:**
- Create: `app/src/ai/local_suggestions/mod.rs`
- Create: `app/src/ai/local_suggestions/client.rs`
- Create: `app/src/ai/local_suggestions/tests.rs`
- Modify: `app/src/ai/mod.rs`

- [ ] **Step 1: Write failing client tests**

Create `app/src/ai/local_suggestions/tests.rs`:

```rust
use super::client::{completion_url, parse_json_object, strip_json_fence};

#[test]
fn test_completion_url_appends_chat_completions() {
    assert_eq!(
        completion_url("https://example.com/v1").unwrap().as_str(),
        "https://example.com/v1/chat/completions"
    );
}

#[test]
fn test_completion_url_keeps_full_chat_completions_url() {
    assert_eq!(
        completion_url("https://example.com/openai/chat/completions")
            .unwrap()
            .as_str(),
        "https://example.com/openai/chat/completions"
    );
}

#[test]
fn test_strip_json_fence() {
    let raw = "```json\n{\"commands\":[\"ls\"],\"ai_queries\":[],\"most_likely_action\":\"ls\"}\n```";
    assert_eq!(
        strip_json_fence(raw),
        "{\"commands\":[\"ls\"],\"ai_queries\":[],\"most_likely_action\":\"ls\"}"
    );
}

#[test]
fn test_parse_json_object_from_fenced_response() {
    let raw = "```json\n{\"id\":\"one\",\"suggestion\":{\"simple\":{\"query\":\"explain failure\",\"should_plan_task\":false}}}\n```";
    let value = parse_json_object(raw).unwrap();
    assert_eq!(value["id"], "one");
    assert!(value["suggestion"]["simple"]["query"].is_string());
}
```

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
cargo test -p warp ai::local_suggestions::tests
```

Expected: fail because `local_suggestions` does not exist.

- [ ] **Step 3: Export the module**

In `app/src/ai/mod.rs`, add:

```rust
pub mod local_suggestions;
```

Create `app/src/ai/local_suggestions/mod.rs`:

```rust
pub mod client;

#[cfg(test)]
mod tests;

use crate::settings::LocalAiEffort;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalSuggestionsConfig {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub effort: LocalAiEffort,
}
```

Then add this getter to `impl AISettings` in `app/src/settings/ai.rs`:

```rust
    pub fn local_suggestions_config(&self) -> Option<crate::ai::local_suggestions::LocalSuggestionsConfig> {
        let endpoint = self.local_openai_endpoint.trim();
        let model = self.local_openai_model.trim();
        if endpoint.is_empty() || model.is_empty() {
            return None;
        }

        Some(crate::ai::local_suggestions::LocalSuggestionsConfig {
            endpoint: endpoint.to_string(),
            api_key: self.local_openai_api_key.trim().to_string(),
            model: model.to_string(),
            effort: *self.local_openai_effort,
        })
    }
```

- [ ] **Step 4: Implement URL and JSON helpers**

Create `app/src/ai/local_suggestions/client.rs`:

```rust
use anyhow::{anyhow, Result};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::LocalSuggestionsConfig;

#[derive(Clone)]
pub struct LocalOpenAIClient {
    http: reqwest::Client,
}

impl Default for LocalOpenAIClient {
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
    without_language.strip_suffix("```").unwrap_or(without_language).trim()
}

pub fn parse_json_object(raw: &str) -> Result<Value> {
    let value: Value = serde_json::from_str(strip_json_fence(raw))?;
    if !value.is_object() {
        return Err(anyhow!("OpenAI-compatible response content is not a JSON object"));
    }
    Ok(value)
}

impl LocalOpenAIClient {
    pub async fn complete_json(
        &self,
        config: &LocalSuggestionsConfig,
        system: String,
        user: String,
    ) -> Result<Value> {
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
            reasoning_effort: Some(config.effort.config_value().to_string()),
        };

        let mut builder = self.http.post(completion_url(&config.endpoint)?).json(&request);
        if !config.api_key.is_empty() {
            builder = builder.bearer_auth(&config.api_key);
        }

        let response = builder.send().await?;
        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(anyhow!("OpenAI-compatible endpoint returned {status}: {body}"));
        }

        let completion: ChatCompletionResponse = serde_json::from_str(&body)?;
        let content = completion
            .choices
            .first()
            .map(|choice| choice.message.content.as_str())
            .ok_or_else(|| anyhow!("OpenAI-compatible response has no choices"))?;

        parse_json_object(content)
    }
}
```

- [ ] **Step 5: Run client tests**

Run:

```bash
cargo test -p warp ai::local_suggestions::tests
```

Expected: tests pass.

- [ ] **Step 6: Commit**

```bash
git add app/src/ai/mod.rs app/src/ai/local_suggestions
git commit -m "feat: add local openai suggestions client"
```

---

### Task 3: Add Local Suggestions Provider

**Files:**
- Modify: `app/src/ai/local_suggestions/mod.rs`
- Create: `app/src/ai/local_suggestions/provider.rs`
- Modify: `app/src/ai/local_suggestions/tests.rs`

- [ ] **Step 1: Write provider mapping tests**

Append to `app/src/ai/local_suggestions/tests.rs`:

```rust
use crate::ai::predict::generate_ai_input_suggestions::{
    AgentModeSuggestionV2, GenerateAIInputSuggestionsResponseV2,
};
use crate::ai::predict::generate_am_query_suggestions::{
    GenerateAMQuerySuggestionsResponse, SimpleQuery, Suggestion,
};

use super::provider::{map_input_suggestions_value, map_prompt_suggestions_value};

#[test]
fn test_map_input_suggestions_value() {
    let value = serde_json::json!({
        "commands": ["cargo test"],
        "ai_queries": [{"query": "explain the failure", "context_block_ids": ["block-1"]}],
        "most_likely_action": "cargo test"
    });

    let actual = map_input_suggestions_value(value).unwrap();
    assert_eq!(
        actual,
        GenerateAIInputSuggestionsResponseV2 {
            commands: vec!["cargo test".to_string()],
            ai_queries: vec![AgentModeSuggestionV2 {
                query: "explain the failure".to_string(),
                context_block_ids: vec!["block-1".to_string()],
            }],
            most_likely_action: "cargo test".to_string(),
        }
    );
}

#[test]
fn test_map_prompt_suggestions_value() {
    let value = serde_json::json!({
        "id": "local-one",
        "suggestion": {
            "simple": {
                "query": "explain why the command failed",
                "should_plan_task": false
            }
        }
    });

    let actual = map_prompt_suggestions_value(value).unwrap();
    assert_eq!(
        actual,
        GenerateAMQuerySuggestionsResponse {
            id: "local-one".to_string(),
            suggestion: Some(Suggestion::Simple(SimpleQuery {
                query: "explain why the command failed".to_string(),
                should_plan_task: false,
            })),
        }
    );
}
```

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
cargo test -p warp ai::local_suggestions::tests::test_map_input_suggestions_value ai::local_suggestions::tests::test_map_prompt_suggestions_value
```

Expected: fail because `provider` does not exist.

- [ ] **Step 3: Export provider**

In `app/src/ai/local_suggestions/mod.rs`, change to:

```rust
pub mod client;
pub mod provider;

#[cfg(test)]
mod tests;

pub use provider::{LocalSuggestionProvider, SuggestionProvider};

use crate::settings::LocalAiEffort;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalSuggestionsConfig {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub effort: LocalAiEffort,
}
```

- [ ] **Step 4: Implement provider**

Create `app/src/ai/local_suggestions/provider.rs`:

```rust
use async_trait::async_trait;
use serde_json::Value;

use crate::ai::predict::generate_ai_input_suggestions::{
    GenerateAIInputSuggestionsRequest, GenerateAIInputSuggestionsResponseV2,
};
use crate::ai::predict::generate_am_query_suggestions::{
    GenerateAMQuerySuggestionsRequest, GenerateAMQuerySuggestionsResponse,
};
use crate::server::server_api::AIApiError;

use super::client::LocalOpenAIClient;
use super::LocalSuggestionsConfig;

#[async_trait]
pub trait SuggestionProvider: Send + Sync {
    async fn generate_input_suggestions(
        &self,
        config: LocalSuggestionsConfig,
        request: GenerateAIInputSuggestionsRequest,
    ) -> Result<GenerateAIInputSuggestionsResponseV2, AIApiError>;

    async fn generate_prompt_suggestions(
        &self,
        config: LocalSuggestionsConfig,
        request: GenerateAMQuerySuggestionsRequest,
    ) -> Result<GenerateAMQuerySuggestionsResponse, AIApiError>;
}

#[derive(Default)]
pub struct LocalSuggestionProvider {
    client: LocalOpenAIClient,
}

pub fn input_suggestions_system_prompt() -> String {
    [
        "You generate terminal next-command suggestions.",
        "Return only JSON with keys commands, ai_queries, most_likely_action.",
        "commands is an array of shell command strings.",
        "ai_queries is an array of objects with query and context_block_ids.",
        "most_likely_action must be the single best shell command.",
    ]
    .join("\n")
}

pub fn prompt_suggestions_system_prompt() -> String {
    [
        "You generate one follow-up agent prompt for a completed terminal command.",
        "Return only JSON with keys id and suggestion.",
        "Use {\"suggestion\":{\"simple\":{\"query\":\"...\",\"should_plan_task\":false}}}.",
        "Return {\"id\":\"local\",\"suggestion\":null} if no useful prompt exists.",
    ]
    .join("\n")
}

pub fn map_input_suggestions_value(
    value: Value,
) -> Result<GenerateAIInputSuggestionsResponseV2, AIApiError> {
    serde_json::from_value(value).map_err(AIApiError::from)
}

pub fn map_prompt_suggestions_value(
    value: Value,
) -> Result<GenerateAMQuerySuggestionsResponse, AIApiError> {
    serde_json::from_value(value).map_err(AIApiError::from)
}

#[async_trait]
impl SuggestionProvider for LocalSuggestionProvider {
    async fn generate_input_suggestions(
        &self,
        config: LocalSuggestionsConfig,
        request: GenerateAIInputSuggestionsRequest,
    ) -> Result<GenerateAIInputSuggestionsResponseV2, AIApiError> {
        let user = serde_json::to_string(&request)?;
        let value = self
            .client
            .complete_json(&config, input_suggestions_system_prompt(), user)
            .await
            .map_err(AIApiError::Other)?;
        map_input_suggestions_value(value)
    }

    async fn generate_prompt_suggestions(
        &self,
        config: LocalSuggestionsConfig,
        request: GenerateAMQuerySuggestionsRequest,
    ) -> Result<GenerateAMQuerySuggestionsResponse, AIApiError> {
        let user = serde_json::to_string(&request)?;
        let value = self
            .client
            .complete_json(&config, prompt_suggestions_system_prompt(), user)
            .await
            .map_err(AIApiError::Other)?;
        map_prompt_suggestions_value(value)
    }
}
```

- [ ] **Step 5: Run provider tests**

Run:

```bash
cargo test -p warp ai::local_suggestions::tests
```

Expected: all local suggestions tests pass.

- [ ] **Step 6: Commit**

```bash
git add app/src/ai/local_suggestions
git commit -m "feat: add local suggestions provider"
```

---

### Task 4: Route Next Command to Local Suggestions

**Files:**
- Modify: `app/src/ai/predict/next_command_model.rs`
- Modify: `app/src/ai/predict/next_command_model_test.rs`
- Modify: `app/src/lib.rs`

- [ ] **Step 1: Write a provider unit test seam**

Append to `app/src/ai/predict/next_command_model_test.rs`:

```rust
#[test]
fn test_next_command_enabled_uses_local_setting_only() {
    App::test((), |mut app| async move {
        crate::settings::AISettings::register(&mut app);

        app.read(|ctx| {
            assert!(is_next_command_enabled(ctx));
        });

        crate::settings::AISettings::handle(&app).update(&mut app, |settings, ctx| {
            settings.local_next_command_enabled.set_value(false, ctx).unwrap();
        });

        app.read(|ctx| {
            assert!(!is_next_command_enabled(ctx));
        });
    });
}
```

- [ ] **Step 2: Run test and verify it fails**

Run:

```bash
cargo test -p warp ai::predict::next_command_model_test::test_next_command_enabled_uses_local_setting_only
```

Expected: fail because `is_next_command_enabled` still depends on old active AI and `UserWorkspaces`.

- [ ] **Step 3: Replace enablement**

In `app/src/ai/predict/next_command_model.rs`, change `is_next_command_enabled` to:

```rust
pub fn is_next_command_enabled(app: &warpui::AppContext) -> bool {
    AISettings::as_ref(app).is_local_next_command_enabled()
}
```

Remove the unused `UserWorkspaces` import from this file.

- [ ] **Step 4: Replace `ServerApi` with `SuggestionProvider`**

Change imports in `app/src/ai/predict/next_command_model.rs`:

```rust
use crate::ai::local_suggestions::SuggestionProvider;
use crate::server::server_api::AIApiError;
```

Change the struct field:

```rust
    suggestion_provider: Arc<dyn SuggestionProvider>,
```

Change `new` signature and body:

```rust
    pub fn new(
        sessions: ModelHandle<Sessions>,
        model: Arc<FairMutex<TerminalModel>>,
        suggestion_provider: Arc<dyn SuggestionProvider>,
    ) -> Self {
        #[cfg(feature = "local_fs")]
        let conn = crate::persistence::database_file_path()
            .to_str()
            .and_then(|db_url| {
                crate::persistence::establish_ro_connection(db_url)
                    .ok()
                    .map(|conn| Arc::new(Mutex::new(conn)))
            });
        Self {
            sessions,
            model,
            suggestion_provider,
            #[cfg(feature = "local_fs")]
            conn,
            next_command_state: NextCommandSuggestionState::None,
            cached_zerostate_next_command_context: None,
            zerostate_suggestion_info: None,
            next_command_abort_handle: None,
        }
    }
```

Inside `generate_next_command_suggestion_with_prefix`, replace:

```rust
        let server_api = self.server_api.clone();
```

with:

```rust
        let suggestion_provider = self.suggestion_provider.clone();
        let local_config = AISettings::as_ref(ctx).local_suggestions_config();
```

Replace each `server_api.generate_ai_input_suggestions(&request).await` with:

```rust
                            match local_config.clone() {
                                Some(config) => suggestion_provider
                                    .generate_input_suggestions(config, request.clone())
                                    .await,
                                None => Err(AIApiError::Other(anyhow::anyhow!(
                                    "Local suggestions endpoint and model must be configured"
                                ))),
                            }
```

For the final partial fallback, use the same expression assigned to `response`.

- [ ] **Step 5: Register provider in app initialization**

In `app/src/lib.rs`, find the existing `NextCommandModel::new` registration. Replace the `ServerApi` argument with:

```rust
Arc::new(crate::ai::local_suggestions::LocalSuggestionProvider::default())
```

Use the existing `Arc` import if present; otherwise add `use std::sync::Arc;` at the top of the smallest scope that needs it.

- [ ] **Step 6: Run Next Command tests**

Run:

```bash
cargo test -p warp ai::predict::next_command_model_test::test_next_command_enabled_uses_local_setting_only
```

Expected: test passes.

- [ ] **Step 7: Commit**

```bash
git add app/src/ai/predict/next_command_model.rs app/src/ai/predict/next_command_model_test.rs app/src/lib.rs
git commit -m "feat: route next command to local suggestions"
```

---

### Task 5: Route Prompt Suggestions to Local Suggestions

**Files:**
- Modify: `app/src/ai/blocklist/passive_suggestions/legacy.rs`

- [ ] **Step 1: Write focused predicate tests**

Add this test module to the bottom of `app/src/ai/blocklist/passive_suggestions/legacy.rs`:

```rust
#[cfg(test)]
mod local_prompt_suggestion_tests {
    use super::*;
    use warpui::App;

    #[test]
    fn test_local_prompt_suggestions_do_not_require_network_or_workspace() {
        App::test((), |mut app| async move {
            crate::settings::AISettings::register(&mut app);

            app.read(|ctx| {
                assert!(AISettings::as_ref(ctx).is_local_prompt_suggestions_enabled());
            });

            crate::settings::AISettings::handle(&app).update(&mut app, |settings, ctx| {
                settings
                    .local_prompt_suggestions_enabled
                    .set_value(false, ctx)
                    .unwrap();
            });

            app.read(|ctx| {
                assert!(!AISettings::as_ref(ctx).is_local_prompt_suggestions_enabled());
            });
        });
    }
}
```

- [ ] **Step 2: Run test and verify it fails or compile fails**

Run:

```bash
cargo test -p warp ai::blocklist::passive_suggestions::legacy::local_prompt_suggestion_tests::test_local_prompt_suggestions_do_not_require_network_or_workspace
```

Expected before implementation: compile succeeds only after Task 1; old production predicate still uses network/workspace and will be replaced in the next step.

- [ ] **Step 3: Remove old server dependencies**

In `app/src/ai/blocklist/passive_suggestions/legacy.rs`, remove these imports:

```rust
use crate::network::NetworkStatus;
use crate::server::server_api::ServerApiProvider;
use crate::workspaces::user_workspaces::UserWorkspaces;
```

Add:

```rust
use crate::ai::local_suggestions::{LocalSuggestionProvider, SuggestionProvider};
```

- [ ] **Step 4: Replace prompt suggestion request future**

Inside `generate_prompt_suggestions`, replace the server API future:

```rust
        let server_api = ServerApiProvider::handle(ctx).as_ref(ctx).get();
        let request_future =
            async move { server_api.generate_am_query_suggestions(&request).await };
```

with:

```rust
        let local_config = AISettings::as_ref(ctx).local_suggestions_config();
        let provider = LocalSuggestionProvider::default();
        let request_future = async move {
            match local_config {
                Some(config) => provider.generate_prompt_suggestions(config, request).await,
                None => Err(crate::server::server_api::AIApiError::Other(anyhow::anyhow!(
                    "Local suggestions endpoint and model must be configured"
                ))),
            }
        };
```

- [ ] **Step 5: Replace prompt suggestion predicate**

Change `should_generate_prompt_suggestions` to:

```rust
fn should_generate_prompt_suggestions(
    block_completed: &UserBlockCompleted,
    ctx: &ModelContext<PassiveSuggestionsModel>,
) -> bool {
    !block_completed.command.trim().is_empty()
        && AISettings::as_ref(ctx).is_local_prompt_suggestions_enabled()
}
```

Change `should_generate_unit_test_suggestion` to:

```rust
fn should_generate_unit_test_suggestion(
    _block_completed: &UserBlockCompleted,
    _ctx: &ModelContext<PassiveSuggestionsModel>,
) -> bool {
    false
}
```

Change `passive_code_diffs_enabled` to:

```rust
fn passive_code_diffs_enabled(_ctx: &ModelContext<PassiveSuggestionsModel>) -> bool {
    false
}
```

- [ ] **Step 6: Run prompt suggestion tests**

Run:

```bash
cargo test -p warp ai::blocklist::passive_suggestions::legacy::local_prompt_suggestion_tests::test_local_prompt_suggestions_do_not_require_network_or_workspace
```

Expected: test passes.

- [ ] **Step 7: Commit**

```bash
git add app/src/ai/blocklist/passive_suggestions/legacy.rs
git commit -m "feat: route prompt suggestions to local endpoint"
```

---

### Task 6: Add ACP Backend Types and Event Mapping

**Files:**
- Create: `app/src/ai/acp/mod.rs`
- Create: `app/src/ai/acp/backend.rs`
- Create: `app/src/ai/acp/events.rs`
- Create: `app/src/ai/acp/mapping.rs`
- Create: `app/src/ai/acp/tests.rs`
- Modify: `app/src/ai/mod.rs`

- [ ] **Step 1: Write mapping tests**

Create `app/src/ai/acp/tests.rs`:

```rust
use agent_client_protocol::schema::{ContentBlock, ContentChunk, SessionUpdate, TextContent};

use crate::settings::AcpAgentBackend;

use super::mapping::map_session_update;
use super::events::AcpEvent;

#[test]
fn test_backend_commands_are_fixed() {
    assert_eq!(AcpAgentBackend::Codex.adapter_command(), "codex-acp");
    assert_eq!(AcpAgentBackend::Claude.adapter_command(), "claude-agent-acp");
    assert_eq!(
        AcpAgentBackend::Codex.install_command(),
        "npm i -g @zed-industries/codex-acp"
    );
    assert_eq!(
        AcpAgentBackend::Claude.install_command(),
        "npm i -g @agentclientprotocol/claude-agent-acp"
    );
}

#[test]
fn test_map_agent_message_chunk() {
    let update = SessionUpdate::AgentMessageChunk(ContentChunk {
        content: ContentBlock::Text(TextContent::new("hello".to_string())),
    });

    assert_eq!(
        map_session_update(update),
        Some(AcpEvent::AssistantTextDelta {
            text: "hello".to_string()
        })
    );
}
```

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
cargo test -p warp ai::acp::tests
```

Expected: fail because `ai::acp` module does not exist.

- [ ] **Step 3: Export ACP module**

In `app/src/ai/mod.rs`, add:

```rust
pub mod acp;
```

Create `app/src/ai/acp/mod.rs`:

```rust
pub mod backend;
pub mod events;
pub mod mapping;
pub mod model;

#[cfg(test)]
mod tests;
```

- [ ] **Step 4: Add internal ACP events**

Create `app/src/ai/acp/events.rs`:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcpEvent {
    AdapterMissing {
        command: String,
        install_command: String,
    },
    SessionStarted,
    AssistantTextDelta {
        text: String,
    },
    PlanUpdated {
        entries: Vec<String>,
    },
    PermissionRequested {
        request_id: String,
        options: Vec<String>,
    },
    Completed,
    Failed {
        message: String,
    },
}
```

Create `app/src/ai/acp/backend.rs`:

```rust
use std::process::Command;

use crate::settings::AcpAgentBackend;

pub fn adapter_is_available(backend: AcpAgentBackend) -> bool {
    Command::new(backend.adapter_command())
        .arg("--version")
        .output()
        .is_ok()
}
```

- [ ] **Step 5: Implement pure mapping**

Create `app/src/ai/acp/mapping.rs`:

```rust
use agent_client_protocol::schema::{ContentBlock, SessionUpdate};

use super::events::AcpEvent;

pub fn map_session_update(update: SessionUpdate) -> Option<AcpEvent> {
    match update {
        SessionUpdate::AgentMessageChunk(chunk) => match chunk.content {
            ContentBlock::Text(text) => Some(AcpEvent::AssistantTextDelta {
                text: text.text,
            }),
            _ => None,
        },
        SessionUpdate::Plan(plan) => Some(AcpEvent::PlanUpdated {
            entries: plan.entries.into_iter().map(|entry| entry.content).collect(),
        }),
        _ => None,
    }
}
```

- [ ] **Step 6: Run ACP mapping tests**

Run:

```bash
cargo test -p warp ai::acp::tests
```

Expected: tests pass. If ACP crate field names differ from the snippet, inspect `~/.cargo/registry/src/*/agent-client-protocol-0.11.1/src/schema` and update only the field names, not the architecture.

- [ ] **Step 7: Commit**

```bash
git add app/src/ai/mod.rs app/src/ai/acp
git commit -m "feat: add ACP event mapping"
```

---

### Task 7: Add ACP Agent Model

**Files:**
- Create: `app/src/ai/acp/model.rs`
- Modify: `app/src/ai/acp/mod.rs`
- Modify: `app/src/lib.rs`

- [ ] **Step 1: Write model smoke test**

Append to `app/src/ai/acp/tests.rs`:

```rust
use super::model::{AcpAgentModel, AcpAgentState};

#[test]
fn test_acp_model_starts_idle() {
    warpui::App::test((), |mut app| async move {
        app.add_singleton_model(AcpAgentModel::new);

        app.read(|ctx| {
            assert_eq!(
                AcpAgentModel::as_ref(ctx).state(),
                AcpAgentState::Idle
            );
        });
    });
}
```

- [ ] **Step 2: Run test and verify it fails**

Run:

```bash
cargo test -p warp ai::acp::tests::test_acp_model_starts_idle
```

Expected: fail because `AcpAgentModel` does not exist.

- [ ] **Step 3: Implement model state and constructor**

Create `app/src/ai/acp/model.rs`:

```rust
use std::path::PathBuf;

use agent_client_protocol::schema::{
    ContentBlock, InitializeRequest, NewSessionRequest, PromptRequest, ProtocolVersion,
    TextContent,
};
use agent_client_protocol::{Agent, Client, ConnectionTo};
use agent_client_protocol_tokio::AcpAgent;
use warpui::{Entity, ModelContext, SingletonEntity};

use crate::settings::{AISettings, AcpAgentBackend};

use super::backend::adapter_is_available;
use super::events::AcpEvent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcpAgentState {
    Idle,
    Starting,
    Ready,
    Running,
    Failed(String),
}

pub struct AcpAgentModel {
    state: AcpAgentState,
    events: Vec<AcpEvent>,
}

impl AcpAgentModel {
    pub fn new(_: &mut ModelContext<Self>) -> Self {
        Self {
            state: AcpAgentState::Idle,
            events: Vec::new(),
        }
    }

    pub fn state(&self) -> AcpAgentState {
        self.state.clone()
    }

    pub fn events(&self) -> &[AcpEvent] {
        &self.events
    }

    pub fn submit_prompt(
        &mut self,
        prompt: String,
        cwd: PathBuf,
        ctx: &mut ModelContext<Self>,
    ) {
        let backend = *AISettings::as_ref(ctx).acp_agent_backend;
        if !adapter_is_available(backend) {
            self.state = AcpAgentState::Failed(format!(
                "{} is not installed",
                backend.adapter_command()
            ));
            ctx.emit(AcpEvent::AdapterMissing {
                command: backend.adapter_command().to_string(),
                install_command: backend.install_command().to_string(),
            });
            return;
        }

        self.state = AcpAgentState::Starting;
        ctx.spawn(
            run_one_prompt(backend, prompt, cwd),
            |me, result, ctx| match result {
                Ok(events) => {
                    me.state = AcpAgentState::Ready;
                    for event in events {
                        ctx.emit(event.clone());
                        me.events.push(event);
                    }
                }
                Err(err) => {
                    let message = err.to_string();
                    me.state = AcpAgentState::Failed(message.clone());
                    ctx.emit(AcpEvent::Failed { message });
                }
            },
        );
    }
}

async fn run_one_prompt(
    backend: AcpAgentBackend,
    prompt: String,
    cwd: PathBuf,
) -> anyhow::Result<Vec<AcpEvent>> {
    let agent = AcpAgent::from_args([backend.adapter_command()])?;
    let mut collected = Vec::new();

    Client
        .builder()
        .on_receive_notification(
            async move |notification: agent_client_protocol::schema::SessionNotification, _cx| {
                if let Some(event) = crate::ai::acp::mapping::map_session_update(notification.update)
                {
                    collected.push(event);
                }
                Ok(())
            },
            agent_client_protocol::on_receive_notification!(),
        )
        .connect_to(agent)
        .await?
        .connect_with(|connection: ConnectionTo<Agent>| async move {
            connection
                .send_request(InitializeRequest::new(ProtocolVersion::V1))
                .block_task()
                .await?;

            let new_session = connection
                .send_request(NewSessionRequest::new(cwd))
                .block_task()
                .await?;

            connection
                .send_request(PromptRequest::new(
                    new_session.session_id,
                    vec![ContentBlock::Text(TextContent::new(prompt))],
                ))
                .block_task()
                .await?;

            Ok(())
        })
        .await?;

    collected.push(AcpEvent::Completed);
    Ok(collected)
}

impl Entity for AcpAgentModel {
    type Event = AcpEvent;
}

impl SingletonEntity for AcpAgentModel {}
```

If the borrow checker rejects the `collected` capture inside the notification handler, replace `Vec<AcpEvent>` with `Arc<Mutex<Vec<AcpEvent>>>` inside `run_one_prompt`, clone it into the notification handler, and clone the locked vector after `connect_with` completes.

- [ ] **Step 4: Register model**

In `app/src/lib.rs`, near other AI singleton registrations, add:

```rust
ctx.add_singleton_model(crate::ai::acp::model::AcpAgentModel::new);
```

- [ ] **Step 5: Run ACP model test**

Run:

```bash
cargo test -p warp ai::acp::tests::test_acp_model_starts_idle
```

Expected: test passes.

- [ ] **Step 6: Commit**

```bash
git add app/src/ai/acp/model.rs app/src/ai/acp/tests.rs app/src/lib.rs
git commit -m "feat: add ACP agent model"
```

---

### Task 8: Route Natural Language Input to ACP

**Files:**
- Modify: `app/src/terminal/input.rs`
- Modify: `app/src/terminal/input_test.rs`

- [ ] **Step 1: Write input routing test**

In `app/src/terminal/input_test.rs`, add a focused test beside existing AI submit tests:

```rust
#[test]
fn test_submit_ai_query_uses_acp_model() {
    App::test((), |mut app| async move {
        initialize_app_for_input_tests(&mut app);
        app.add_singleton_model(crate::ai::acp::model::AcpAgentModel::new);

        app.update(|ctx| {
            let mut input = create_input_for_test(ctx);
            input.editor.update(ctx, |editor, ctx| {
                editor.set_text("explain the last error", ctx);
            });
            input.submit_ai_query(None, ctx);
        });

        app.read(|ctx| {
            let state = crate::ai::acp::model::AcpAgentModel::as_ref(ctx).state();
            assert!(matches!(
                state,
                crate::ai::acp::model::AcpAgentState::Starting
                    | crate::ai::acp::model::AcpAgentState::Failed(_)
            ));
        });
    });
}
```

If `create_input_for_test` has a different helper name in this file, use the helper already used by neighboring `submit_ai_query` tests and keep the assertions unchanged.

- [ ] **Step 2: Run test and verify it fails**

Run:

```bash
cargo test -p warp terminal::input_test::test_submit_ai_query_uses_acp_model
```

Expected: fail because `submit_ai_query` still enters AgentView or `AIController`.

- [ ] **Step 3: Add ACP route at top of `submit_ai_query`**

In `app/src/terminal/input.rs`, add import:

```rust
use crate::ai::acp::model::AcpAgentModel;
```

Inside `submit_ai_query`, after the shared-session viewer guard and before the `FeatureFlag::AgentView` branch, insert:

```rust
        let ai_query = self.editor.as_ref(ctx).buffer_text(ctx);
        let ai_query = ai_query.trim().to_string();
        if ai_query.is_empty() {
            return;
        }

        let cwd = self
            .active_session
            .as_ref(ctx)
            .current_working_directory()
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("/"));

        self.ai_input_model.update(ctx, |model, ctx| {
            model.handle_input_buffer_submitted(ctx);
        });
        self.editor.update(ctx, |editor, ctx| {
            editor.clear_buffer(ctx);
        });
        AcpAgentModel::handle(ctx).update(ctx, |model, ctx| {
            model.submit_prompt(ai_query, cwd, ctx);
        });
        ctx.emit(Event::ExecuteAIQuery);
        return;
```

Add `use std::path::PathBuf;` if the file does not already import it.

Delete the old duplicate `let ai_query = self.editor.as_ref(ctx).buffer_text(ctx);` block lower in the function after this route is proven compiling.

- [ ] **Step 4: Run input routing test**

Run:

```bash
cargo test -p warp terminal::input_test::test_submit_ai_query_uses_acp_model
```

Expected: test passes.

- [ ] **Step 5: Commit**

```bash
git add app/src/terminal/input.rs app/src/terminal/input_test.rs
git commit -m "feat: route natural language input to acp"
```

---

### Task 9: Replace AI Settings Page with Local AI Controls

**Files:**
- Modify: `app/src/settings_view/ai_page.rs`

- [ ] **Step 1: Add a compile-only UI test target**

Run the existing settings view test subset before editing:

```bash
cargo test -p warp settings_view --no-default-features
```

Expected: command may select no tests in this repo configuration. If it selects no tests, use `cargo check -p warp --lib` as the verification command for this task.

- [ ] **Step 2: Add Local AI widget**

In `app/src/settings_view/ai_page.rs`, create a new widget near existing AI settings widgets:

```rust
struct LocalAIWidget;

impl SettingsItem for LocalAIWidget {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> Box<dyn Element> {
        let settings = AISettings::as_ref(cx);
        SettingsSection::new("Local AI")
            .child(settings_dropdown(
                "Agent backend",
                *settings.acp_agent_backend,
                AcpAgentBackend::iter().collect(),
                |backend| backend.display_name(),
                |settings, value, cx| settings.acp_agent_backend.set_value(value, cx),
            ))
            .child(settings_text_field(
                "Agent model",
                settings.acp_model.to_string(),
                |settings, value, cx| settings.acp_model.set_value(value, cx),
            ))
            .child(settings_dropdown(
                "Agent effort",
                *settings.acp_effort,
                LocalAiEffort::iter().collect(),
                |effort| effort.display_name(),
                |settings, value, cx| settings.acp_effort.set_value(value, cx),
            ))
            .child(settings_text_field(
                "Suggestions endpoint",
                settings.local_openai_endpoint.to_string(),
                |settings, value, cx| settings.local_openai_endpoint.set_value(value, cx),
            ))
            .child(settings_secret_field(
                "Suggestions API key",
                settings.local_openai_api_key.to_string(),
                |settings, value, cx| settings.local_openai_api_key.set_value(value, cx),
            ))
            .child(settings_text_field(
                "Suggestions model",
                settings.local_openai_model.to_string(),
                |settings, value, cx| settings.local_openai_model.set_value(value, cx),
            ))
            .child(settings_dropdown(
                "Suggestions effort",
                *settings.local_openai_effort,
                LocalAiEffort::iter().collect(),
                |effort| effort.display_name(),
                |settings, value, cx| settings.local_openai_effort.set_value(value, cx),
            ))
            .child(settings_toggle(
                "Next Command",
                *settings.local_next_command_enabled,
                |settings, value, cx| settings.local_next_command_enabled.set_value(value, cx),
            ))
            .child(settings_toggle(
                "Prompt Suggestions",
                *settings.local_prompt_suggestions_enabled,
                |settings, value, cx| {
                    settings.local_prompt_suggestions_enabled.set_value(value, cx)
                },
            ))
            .finish()
    }
}
```

Use the exact helper names already present in `ai_page.rs`. If this file uses concrete types instead of `settings_dropdown`, keep the same control list and bind each control to the setting shown above.

- [ ] **Step 3: Replace old widgets in `build_page`**

In `build_page`, remove these children from the rendered AI page:

```rust
GlobalAIWidget
UsageWidget
ActiveAIWidget
AgentsWidget
AIInputWidget
ApiKeysWidget
AwsBedrockWidget
CLIAgentWidget
```

Add only:

```rust
LocalAIWidget
```

MCP server UI is no longer retained in the Warp app. ACP adapter configuration belongs to the ACP agent process.

- [ ] **Step 4: Run UI compile check**

Run:

```bash
cargo check -p warp --lib
```

Expected: compile passes.

- [ ] **Step 5: Commit**

```bash
git add app/src/settings_view/ai_page.rs
git commit -m "feat: replace ai settings with local ai controls"
```

---

### Task 10: Remove Login and Cloud Navigation from the User Surface

**Files:**
- Modify: `app/src/root_view.rs`
- Modify: `app/src/settings_view/mod.rs`

- [ ] **Step 1: Verify current root view behavior**

Run:

```bash
rg -n "Auth|Onboarding|is_logged_in|RootViewState|SettingsPageKind::Teams|WarpDrive|Billing|Cloud" app/src/root_view.rs app/src/settings_view/mod.rs
```

Expected: output lists auth/onboarding branches in `root_view.rs` and cloud/team pages in `settings_view/mod.rs`.

- [ ] **Step 2: Force terminal workspace as root view**

In `app/src/root_view.rs`, change the state selection in `RootView::new` so it always constructs the terminal/workspace state. The resulting branch should not inspect `auth_state.is_logged_in()`.

Use this shape:

```rust
let state = RootViewState::Terminal;
```

Then remove unreachable `Auth` and `Onboarding` branches from the constructor. Leave type definitions in place until the compile check identifies every reference that can be removed safely.

- [ ] **Step 3: Hide cloud settings pages**

In `app/src/settings_view/mod.rs`, remove these page kinds from the sidebar construction:

```rust
Account
BillingAndUsage
CloudPlatform
Teams
WarpDrive
Referrals
SharedBlocks
```

Keep local terminal, appearance, keyboard, AI, and developer settings pages. MCP file config is no longer retained in the Warp app.

- [ ] **Step 4: Run compile check**

Run:

```bash
cargo check -p warp --lib
```

Expected: compile passes or reports remaining direct references to removed sidebar variants. For each reported reference, remove only the render branch for the hidden page and keep enum variants until no code references them.

- [ ] **Step 5: Commit**

```bash
git add app/src/root_view.rs app/src/settings_view/mod.rs
git commit -m "feat: hide login and cloud navigation"
```

---

### Task 11: Stop Registering Old Agent/Cloud Runtime for Local Builds

**Files:**
- Modify: `app/src/lib.rs`
- Modify: `app/src/ai/harness_availability.rs`
- Modify: `app/src/ai/request_usage_model.rs`

- [ ] **Step 1: Find runtime registrations**

Run:

```bash
rg -n "AgentDriver|AIRequestUsageModel|HarnessAvailabilityModel|CloudPreferencesSyncer|TeamUpdateManager|ApiKeyManager|UserWorkspaces|AgentConversationsModel" app/src/lib.rs
```

Expected: output identifies old server-backed models registered during app boot.

- [ ] **Step 2: Remove server-backed Agent registration**

In `app/src/lib.rs`, remove registrations for:

```rust
AgentDriver
AgentConversationsModel
AIRequestUsageModel
HarnessAvailabilityModel
ApiKeyManager
TeamUpdateManager
CloudPreferencesSyncer
```

Keep `UserWorkspaces` only if compile errors show non-AI terminal features still read it. If it remains, do not let Local AI getters depend on it.

- [ ] **Step 3: Replace usage model call sites in local AI flow**

Search:

```bash
rg -n "AIRequestUsageModel::|has_requests_remaining|has_any_ai_remaining|enable_buy_credits_banner" app/src
```

Remove only the call sites that are on the natural-language submit path now owned by ACP. Keep unrelated code until its UI entry is removed and compile errors prove it is unused.

- [ ] **Step 4: Run compile check**

Run:

```bash
cargo check -p warp --lib
```

Expected: compile passes after old AI usage checks are removed from local flow.

- [ ] **Step 5: Commit**

```bash
git add app/src/lib.rs app/src/ai/harness_availability.rs app/src/ai/request_usage_model.rs
git commit -m "feat: stop registering server agent runtime"
```

---

### Task 12: End-to-End Local Verification

**Files:**
- No source edits unless a previous task left compile failures.

- [ ] **Step 1: Verify adapter commands are discoverable**

Run:

```bash
command -v codex-acp
command -v claude-agent-acp
```

Expected when user installed both adapters: each command prints an absolute path. If one is missing, Warp should show the install command from `AcpAgentBackend::install_command()`.

- [ ] **Step 2: Run focused tests**

Run:

```bash
cargo test -p warp ai::local_suggestions::tests ai::acp::tests settings::ai_tests::test_local_ai_settings_defaults settings::ai_tests::test_local_ai_getters_do_not_require_auth
```

Expected: all selected tests pass.

- [ ] **Step 3: Run compile check**

Run:

```bash
cargo check -p warp --lib
```

Expected: compile passes.

- [ ] **Step 4: Manual app verification**

Run the app using the repo's existing desktop run command:

```bash
cargo run -p warp --bin warp-oss
```

Expected:
- App opens without login.
- Settings shows Local AI controls.
- Natural language input submits to selected ACP backend.
- Missing adapter shows install command.
- Next Command and Prompt Suggestions do not make Warp server requests.

- [ ] **Step 5: Commit verification fixes**

If Step 4 required source changes, commit only those fixes:

```bash
git add app/src
git commit -m "fix: complete ACP verification"
```

If Step 4 required no changes, do not create an empty commit.

---

## Self-Review

**Spec coverage:** ACP backend without command path is covered by Tasks 6-8. User-installed npm adapter path is covered by Tasks 6 and 12. OpenAI-compatible Next Command and Prompt Suggestions are covered by Tasks 2-5. Settings UI is covered by Task 9. Login, Teams, Drive, Billing, Usage, and Warp Agent surface removal is covered by Tasks 10-11.

**Placeholder scan:** The plan avoids vague future-work markers, unnamed deferred work, and unowned generic edge-case steps. Where compile-time API names may differ in the external ACP crate, the plan gives the exact registry path to inspect and limits the adjustment to field names.

**Type consistency:** `AcpAgentBackend`, `LocalAiEffort`, `LocalSuggestionsConfig`, `LocalSuggestionProvider`, `SuggestionProvider`, `AcpEvent`, `AcpAgentModel`, and `AcpAgentState` are introduced before they are referenced by later tasks.
