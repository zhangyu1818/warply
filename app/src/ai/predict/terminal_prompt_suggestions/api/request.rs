use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalPromptSuggestionsRequest {
    pub context_messages: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_context: Option<String>,
    pub exit_code: i32,
}
