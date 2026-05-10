pub mod client;
pub mod provider;

#[cfg(test)]
mod tests;

use crate::settings::TerminalSuggestionEffort;

#[derive(Clone, Debug, PartialEq)]
pub struct TerminalSuggestionsConfig {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub effort: TerminalSuggestionEffort,
}
