use mockito::{Matcher, Server};

use crate::ai::predict::terminal_input_suggestions::{
    TerminalAgentPromptSuggestion, TerminalInputSuggestionsResponse,
};
use crate::ai::predict::terminal_prompt_suggestions::{
    SimplePromptSuggestion, TerminalPromptSuggestion, TerminalPromptSuggestionsResponse,
};
use crate::settings::TerminalSuggestionEffort;

use super::client::{
    completion_url, parse_json_object, strip_json_fence, ChatCompletionRequest, ChatMessage,
    OpenAICompatibleClient,
};
use super::provider::{map_input_suggestions_value, map_prompt_suggestions_value};
use super::TerminalSuggestionsConfig;

#[test]
fn test_completion_url_appends_chat_completions() {
    assert_eq!(
        completion_url("https://example.com/v1").unwrap().as_str(),
        "https://example.com/v1/chat/completions"
    );
}

#[test]
fn test_completion_url_keeps_full_chat_completions_url() {
    assert_eq!(
        completion_url("https://example.com/openai/chat/completions")
            .unwrap()
            .as_str(),
        "https://example.com/openai/chat/completions"
    );
}

#[test]
fn test_strip_json_fence() {
    let raw =
        "```json\n{\"commands\":[\"ls\"],\"ai_queries\":[],\"most_likely_action\":\"ls\"}\n```";
    assert_eq!(
        strip_json_fence(raw),
        "{\"commands\":[\"ls\"],\"ai_queries\":[],\"most_likely_action\":\"ls\"}"
    );
}

#[test]
fn test_parse_json_object_from_fenced_response() {
    let raw = "```json\n{\"id\":\"one\",\"suggestion\":{\"simple\":{\"query\":\"explain failure\",\"should_plan_task\":false}}}\n```";
    let value = parse_json_object(raw).unwrap();
    assert_eq!(value["id"], "one");
    assert!(value["suggestion"]["simple"]["query"].is_string());
}

#[test]
fn test_parse_json_object_rejects_non_object_response() {
    let err = parse_json_object("[\"ls\"]").unwrap_err();
    assert!(err.to_string().contains("not a JSON object"));
}

#[test]
fn test_chat_request_skips_default_effort() {
    let request = ChatCompletionRequest {
        model: "local-model".to_string(),
        messages: vec![ChatMessage {
            role: "user",
            content: "prompt".to_string(),
        }],
        temperature: 0.0,
        reasoning_effort: TerminalSuggestionEffort::Default
            .config_value()
            .map(str::to_string),
    };

    let value = serde_json::to_value(request).unwrap();
    assert_eq!(value["model"], "local-model");
    assert!(value.get("reasoning_effort").is_none());
}

#[tokio::test]
async fn test_complete_json_posts_chat_request_and_parses_content() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/chat/completions")
        .match_header("authorization", "Bearer test-key")
        .match_body(Matcher::PartialJson(serde_json::json!({
            "model": "local-model",
            "messages": [
                {"role": "system", "content": "system prompt"},
                {"role": "user", "content": "user prompt"},
            ],
        })))
        .with_status(200)
        .with_body(
            serde_json::json!({
                "choices": [{
                    "message": {
                        "content": "{\"commands\":[\"ls\"],\"ai_queries\":[],\"most_likely_action\":\"ls\"}"
                    }
                }]
            })
            .to_string(),
        )
        .expect(1)
        .create_async()
        .await;

    let client = OpenAICompatibleClient::default();
    let config = TerminalSuggestionsConfig {
        endpoint: server.url(),
        api_key: "test-key".to_string(),
        model: "local-model".to_string(),
        effort: TerminalSuggestionEffort::Default,
    };

    let value = client
        .complete_json(
            &config,
            "system prompt".to_string(),
            "user prompt".to_string(),
        )
        .await
        .unwrap();

    assert_eq!(value["commands"][0], "ls");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_complete_json_reports_http_errors() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/chat/completions")
        .with_status(500)
        .with_body("upstream failure")
        .expect(1)
        .create_async()
        .await;

    let client = OpenAICompatibleClient::default();
    let config = TerminalSuggestionsConfig {
        endpoint: server.url(),
        api_key: String::new(),
        model: "local-model".to_string(),
        effort: TerminalSuggestionEffort::Default,
    };

    let err = client
        .complete_json(
            &config,
            "system prompt".to_string(),
            "user prompt".to_string(),
        )
        .await
        .unwrap_err();

    assert!(err.to_string().contains(
        "OpenAI-compatible endpoint returned 500 Internal Server Error: upstream failure"
    ));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_complete_json_posts_reasoning_effort_when_configured() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/chat/completions")
        .match_body(Matcher::PartialJson(serde_json::json!({
            "model": "local-model",
            "reasoning_effort": "high",
        })))
        .with_status(200)
        .with_body(
            serde_json::json!({
                "choices": [{
                    "message": {
                        "content": "{\"commands\":[\"ls\"],\"ai_queries\":[],\"most_likely_action\":\"ls\"}"
                    }
                }]
            })
            .to_string(),
        )
        .expect(1)
        .create_async()
        .await;

    let client = OpenAICompatibleClient::default();
    let config = TerminalSuggestionsConfig {
        endpoint: server.url(),
        api_key: String::new(),
        model: "local-model".to_string(),
        effort: TerminalSuggestionEffort::High,
    };

    let value = client
        .complete_json(
            &config,
            "system prompt".to_string(),
            "user prompt".to_string(),
        )
        .await
        .unwrap();

    assert_eq!(value["commands"][0], "ls");
    mock.assert_async().await;
}

#[test]
fn test_map_input_suggestions_value() {
    let value = serde_json::json!({
        "commands": ["cargo test"],
        "ai_queries": [{"query": "explain the failure", "context_block_ids": ["block-1"]}],
        "most_likely_action": "cargo test"
    });

    let actual = map_input_suggestions_value(value).unwrap();
    assert_eq!(
        actual,
        TerminalInputSuggestionsResponse {
            commands: vec!["cargo test".to_string()],
            ai_queries: vec![TerminalAgentPromptSuggestion {
                query: "explain the failure".to_string(),
                context_block_ids: vec!["block-1".to_string()],
            }],
            most_likely_action: "cargo test".to_string(),
        }
    );
}

#[test]
fn test_map_prompt_suggestions_value() {
    let value = serde_json::json!({
        "id": "suggestions-one",
        "suggestion": {
            "simple": {
                "query": "explain why the command failed",
                "should_plan_task": false
            }
        }
    });

    let actual = map_prompt_suggestions_value(value).unwrap();
    assert_eq!(
        actual,
        TerminalPromptSuggestionsResponse {
            id: "suggestions-one".to_string(),
            suggestion: Some(TerminalPromptSuggestion::Simple(SimplePromptSuggestion {
                query: "explain why the command failed".to_string(),
                should_plan_task: false,
            })),
        }
    );
}
