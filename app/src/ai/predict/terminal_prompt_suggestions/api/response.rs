use serde::{Deserialize, Serialize};

use crate::ai::agent::FileLocations;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalPromptSuggestionsResponse {
    pub id: String,
    pub suggestion: Option<TerminalPromptSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TerminalPromptSuggestion {
    Simple(SimplePromptSuggestion),
    Coding(CodingPromptSuggestion),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SimplePromptSuggestion {
    pub query: String,
    pub should_plan_task: bool,
}

impl TerminalPromptSuggestionsResponse {
    pub fn is_valid_code_delegation(&self) -> bool {
        matches!(&self.suggestion, Some(TerminalPromptSuggestion::Coding(coding_query)) if !coding_query.files.is_empty())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodingPromptSuggestion {
    pub files: Vec<GeneratedFileLocation>,
    pub query: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneratedFileLocation {
    pub file_name: String,
    pub line_numbers: Option<Vec<usize>>,
}

impl From<GeneratedFileLocation> for FileLocations {
    fn from(value: GeneratedFileLocation) -> Self {
        Self {
            name: value.file_name,
            lines: Vec::new(),
        }
    }
}
