use super::*;
use crate::ai::acp::registry::DEFAULT_AGENT_ID;
use crate::test_util::settings::initialize_settings_for_tests;
use warpui::{App, SingletonEntity};

#[test]
fn test_toolbar_command_map_deserialize_from_map() {
    let json = serde_json::json!({
        "^claude": "Claude",
        "^gemini": "Gemini",
        "^codex": ""
    });
    let map: ToolbarCommandMap = serde_json::from_value(json).unwrap();
    assert_eq!(map.0.len(), 3);
    assert_eq!(map.0["^claude"], "Claude");
    assert_eq!(map.0["^gemini"], "Gemini");
    assert_eq!(map.0["^codex"], "");
}

#[test]
fn test_toolbar_command_map_from_file_value_map_format() {
    use settings_value::SettingsValue;

    let value = serde_json::json!({
        "^claude": "Claude",
        "^amp": "Amp"
    });
    let map = ToolbarCommandMap::from_file_value(&value).unwrap();
    assert_eq!(map.0.len(), 2);
    assert_eq!(map.0["^claude"], "Claude");
    assert_eq!(map.0["^amp"], "Amp");
}

#[test]
fn test_toolbar_command_map_from_file_value_invalid() {
    use settings_value::SettingsValue;

    let value = serde_json::json!(42);
    assert!(ToolbarCommandMap::from_file_value(&value).is_none());
}

#[test]
fn test_toolbar_command_map_roundtrip() {
    use settings_value::SettingsValue;

    let mut inner = IndexMap::new();
    inner.insert("^claude".to_string(), "Claude".to_string());
    inner.insert("^custom".to_string(), String::new());
    let original = ToolbarCommandMap::new(inner);

    let file_value = original.to_file_value();
    let restored = ToolbarCommandMap::from_file_value(&file_value).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_terminal_suggestions_settings_defaults() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);

        AISettings::handle(&app).read(&app, |settings, _ctx| {
            assert_eq!(settings.acp_agent_backend.as_str(), DEFAULT_AGENT_ID);
            assert!(settings.acp_default_config_options.is_empty());
            assert_eq!(settings.terminal_suggestions_endpoint.as_str(), "");
            assert_eq!(settings.terminal_suggestions_api_key.as_str(), "");
            assert_eq!(settings.terminal_suggestions_model.as_str(), "");
            assert_eq!(
                *settings.terminal_suggestions_effort,
                TerminalSuggestionEffort::Default
            );
            assert!(*settings.terminal_next_command_enabled);
            assert!(*settings.terminal_prompt_suggestions_enabled);
        });
    });
}

#[test]
fn test_terminal_suggestions_getters_do_not_require_auth() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);

        AISettings::handle(&app).read(&app, |settings, _ctx| {
            assert!(settings.is_terminal_next_command_enabled());
            assert!(settings.is_terminal_prompt_suggestions_enabled());
            assert_eq!(settings.acp_agent_backend.as_str(), DEFAULT_AGENT_ID);
        });
    });
}

#[test]
fn test_terminal_suggestions_config_trims_required_fields() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);

        AISettings::handle(&app).update(&mut app, |settings, ctx| {
            settings
                .terminal_suggestions_endpoint
                .set_value(" https://example.com/v1 ".to_string(), ctx)
                .unwrap();
            settings
                .terminal_suggestions_api_key
                .set_value(" token ".to_string(), ctx)
                .unwrap();
            settings
                .terminal_suggestions_model
                .set_value(" gpt-local ".to_string(), ctx)
                .unwrap();
            settings
                .terminal_suggestions_effort
                .set_value(TerminalSuggestionEffort::High, ctx)
                .unwrap();
        });

        AISettings::handle(&app).read(&app, |settings, _ctx| {
            let config = settings.terminal_suggestions_config().unwrap();
            assert_eq!(config.endpoint, "https://example.com/v1");
            assert_eq!(config.api_key, "token");
            assert_eq!(config.model, "gpt-local");
            assert_eq!(config.effort, TerminalSuggestionEffort::High);
        });
    });
}
