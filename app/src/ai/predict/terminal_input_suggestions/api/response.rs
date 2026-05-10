use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalAgentPromptSuggestion {
    pub query: String,
    pub context_block_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalInputSuggestionsResponse {
    pub commands: Vec<String>,
    pub ai_queries: Vec<TerminalAgentPromptSuggestion>,
    pub most_likely_action: String,
}
