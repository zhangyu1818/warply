use async_trait::async_trait;
use serde_json::Value;

use crate::ai::predict::terminal_input_suggestions::{
    TerminalInputSuggestionsRequest, TerminalInputSuggestionsResponse,
};
use crate::ai::predict::terminal_prompt_suggestions::{
    TerminalPromptSuggestionsRequest, TerminalPromptSuggestionsResponse,
};
use crate::http_api::AIApiError;

use super::TerminalSuggestionsConfig;
use super::client::OpenAICompatibleClient;

#[async_trait]
pub trait SuggestionProvider: Send + Sync {
    async fn generate_input_suggestions(
        &self,
        config: TerminalSuggestionsConfig,
        request: TerminalInputSuggestionsRequest,
    ) -> Result<TerminalInputSuggestionsResponse, AIApiError>;

    async fn generate_prompt_suggestions(
        &self,
        config: TerminalSuggestionsConfig,
        request: TerminalPromptSuggestionsRequest,
    ) -> Result<TerminalPromptSuggestionsResponse, AIApiError>;
}

#[derive(Default)]
pub struct TerminalSuggestionProvider {
    client: OpenAICompatibleClient,
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
        "Return {\"id\":\"suggestions\",\"suggestion\":null} if no useful prompt exists.",
    ]
    .join("\n")
}

pub fn map_input_suggestions_value(
    value: Value,
) -> Result<TerminalInputSuggestionsResponse, AIApiError> {
    serde_json::from_value(value).map_err(AIApiError::from)
}

pub fn map_prompt_suggestions_value(
    value: Value,
) -> Result<TerminalPromptSuggestionsResponse, AIApiError> {
    serde_json::from_value(value).map_err(AIApiError::from)
}

#[async_trait]
impl SuggestionProvider for TerminalSuggestionProvider {
    async fn generate_input_suggestions(
        &self,
        config: TerminalSuggestionsConfig,
        request: TerminalInputSuggestionsRequest,
    ) -> Result<TerminalInputSuggestionsResponse, AIApiError> {
        let user = serde_json::to_string(&request)?;
        log::debug!(
            "[terminal-suggestions] generating next command prefix_present={} context_messages={} history_bytes={}",
            request.prefix.is_some(),
            request.context_messages.len(),
            request.history_context.len(),
        );
        let value = self
            .client
            .complete_json(&config, input_suggestions_system_prompt(), user)
            .await?;
        log::debug!("[terminal-suggestions] mapping next command response");
        map_input_suggestions_value(value)
    }

    async fn generate_prompt_suggestions(
        &self,
        config: TerminalSuggestionsConfig,
        request: TerminalPromptSuggestionsRequest,
    ) -> Result<TerminalPromptSuggestionsResponse, AIApiError> {
        let user = serde_json::to_string(&request)?;
        log::debug!(
            "[terminal-suggestions] generating prompt suggestion context_messages={} exit_code={}",
            request.context_messages.len(),
            request.exit_code,
        );
        let value = self
            .client
            .complete_json(&config, prompt_suggestions_system_prompt(), user)
            .await?;
        log::debug!("[terminal-suggestions] mapping prompt suggestion response");
        map_prompt_suggestions_value(value)
    }
}
