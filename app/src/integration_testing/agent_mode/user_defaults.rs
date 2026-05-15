use std::collections::HashMap;

// User default keys
const IS_ACTIVE_AI_ENABLED: &str = "IsActiveAIEnabled";
const INTELLIGENT_AUTOSUGGESTIONS_ENABLED: &str = "IntelligentAutosuggestionsEnabled";
const CODE_SUGGESTIONS_ENABLED: &str = "CodeSuggestionsEnabled";

pub fn user_defaults_map_with_active_ai(enabled: bool) -> HashMap<String, String> {
    HashMap::from_iter([
        (
            INTELLIGENT_AUTOSUGGESTIONS_ENABLED.to_owned(),
            enabled.to_string(),
        ),
        (CODE_SUGGESTIONS_ENABLED.to_owned(), enabled.to_string()),
        (IS_ACTIVE_AI_ENABLED.to_owned(), enabled.to_string()),
    ])
}

pub fn user_defaults_map_for_ai_input() -> HashMap<String, String> {
    HashMap::from_iter([(
        "InputBoxTypeSetting".to_owned(),
        serde_json::to_string("Universal").unwrap(),
    )])
}
