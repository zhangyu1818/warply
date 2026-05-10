mod static_prompt_suggestions;
mod terminal;

use warpui::ModelHandle;

pub use terminal::{
    PassiveSuggestionsEvent as TerminalPassiveSuggestionsEvent,
    PassiveSuggestionsModel as TerminalPassiveSuggestionsModel,
};

#[derive(Clone)]
pub struct PassiveSuggestionsModels {
    pub terminal: ModelHandle<TerminalPassiveSuggestionsModel>,
}
