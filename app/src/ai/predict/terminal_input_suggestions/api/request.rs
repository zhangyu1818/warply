use serde::{Deserialize, Serialize};

use crate::ai::block_context::BlockContext;
use crate::terminal::input::IntelligentAutosuggestionResult;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalInputSuggestionsRequest {
    pub context_messages: Vec<String>,
    pub history_context: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_context: Option<String>,
    pub rejected_suggestions: Vec<String>,
    pub prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_context: Option<Box<BlockContext>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_result: Option<IntelligentAutosuggestionResult>,
}
