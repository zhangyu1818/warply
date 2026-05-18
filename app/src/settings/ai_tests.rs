use super::*;
use crate::ai::acp::registry::DEFAULT_AGENT_ID;
use crate::test_util::settings::initialize_settings_for_tests;
use warpui::{App, SingletonEntity};

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
