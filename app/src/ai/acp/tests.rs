use std::collections::HashMap;

use agent_client_protocol::schema::{
    AvailableCommand, ConfigOptionUpdate, ContentBlock, ContentChunk, CurrentModeUpdate, Plan,
    PlanEntry, PlanEntryPriority, PlanEntryStatus, SessionConfigOption,
    SessionConfigOptionCategory, SessionConfigSelectGroup, SessionConfigSelectOption,
    SessionInfoUpdate, SessionUpdate, TextContent, ToolCall, ToolCallUpdate, ToolCallUpdateFields,
};

use crate::settings::AcpAgentBackend;

use super::config_options::flatten_config_options;
use super::events::AcpEvent;
use super::mapping::map_session_update;
use super::model::default_config_options_to_apply;
use super::AcpToolCall;

#[test]
fn test_backend_commands_are_fixed() {
    assert_eq!(AcpAgentBackend::Codex.adapter_command(), "codex-acp");
    assert_eq!(
        AcpAgentBackend::Claude.adapter_command(),
        "claude-agent-acp"
    );
    assert_eq!(
        AcpAgentBackend::Codex.install_command(),
        "npm i -g @zed-industries/codex-acp"
    );
    assert_eq!(
        AcpAgentBackend::Claude.install_command(),
        "npm i -g @agentclientprotocol/claude-agent-acp"
    );
}

#[test]
fn test_map_agent_message_chunk() {
    let update = SessionUpdate::AgentMessageChunk(ContentChunk::new(ContentBlock::Text(
        TextContent::new("hello".to_string()),
    )));

    assert_eq!(
        map_session_update(update),
        Some(AcpEvent::AssistantTextDelta {
            text: "hello".to_string()
        })
    );
}

#[test]
fn test_map_agent_thought_chunk() {
    let update = SessionUpdate::AgentThoughtChunk(ContentChunk::new(ContentBlock::Text(
        TextContent::new("thinking".to_string()),
    )));

    assert_eq!(
        map_session_update(update),
        Some(AcpEvent::AssistantThoughtDelta {
            text: "thinking".to_string()
        })
    );
}

#[test]
fn test_map_user_message_chunk() {
    let update = SessionUpdate::UserMessageChunk(ContentChunk::new(ContentBlock::Text(
        TextContent::new("hello".to_string()),
    )));

    assert_eq!(
        map_session_update(update),
        Some(AcpEvent::UserTextDelta {
            text: "hello".to_string()
        })
    );
}

#[test]
fn test_map_tool_call() {
    let update = SessionUpdate::ToolCall(ToolCall::new("tool-1", "Read SKILL.md"));

    assert!(matches!(
        map_session_update(update),
        Some(AcpEvent::ToolCallStarted { tool_call }) if tool_call.id == "tool-1"
    ));
}

#[test]
fn test_map_tool_call_update() {
    let update =
        SessionUpdate::ToolCallUpdate(ToolCallUpdate::new("tool-1", ToolCallUpdateFields::new()));

    assert!(matches!(
        map_session_update(update),
        Some(AcpEvent::ToolCallUpdated { update }) if update.tool_call_id.0.as_ref() == "tool-1"
    ));
}

#[test]
fn test_map_plan_update_preserves_plan() {
    let update = SessionUpdate::Plan(Plan::new(vec![PlanEntry::new(
        "Read files",
        PlanEntryPriority::Medium,
        PlanEntryStatus::InProgress,
    )]));

    assert!(matches!(
        map_session_update(update),
        Some(AcpEvent::PlanUpdated { plan }) if plan.entries.len() == 1
    ));
}

#[test]
fn test_map_available_commands_update() {
    let update = SessionUpdate::AvailableCommandsUpdate(
        agent_client_protocol::schema::AvailableCommandsUpdate::new(vec![AvailableCommand::new(
            "review",
            "Review changes",
        )]),
    );

    assert!(matches!(
        map_session_update(update),
        Some(AcpEvent::AvailableCommandsUpdated { commands }) if commands.len() == 1
    ));
}

#[test]
fn test_map_current_mode_update() {
    let update = SessionUpdate::CurrentModeUpdate(CurrentModeUpdate::new("default"));

    assert!(matches!(
        map_session_update(update),
        Some(AcpEvent::CurrentModeUpdated { update }) if update.current_mode_id.0.as_ref() == "default"
    ));
}

#[test]
fn test_map_config_option_update() {
    let update = SessionUpdate::ConfigOptionUpdate(ConfigOptionUpdate::new(vec![]));

    assert!(matches!(
        map_session_update(update),
        Some(AcpEvent::ConfigOptionsUpdated { update }) if update.config_options.is_empty()
    ));
}

#[test]
fn test_map_session_info_update() {
    let update = SessionUpdate::SessionInfoUpdate(SessionInfoUpdate::new().title("ACP Thread"));

    assert!(matches!(
        map_session_update(update),
        Some(AcpEvent::SessionInfoUpdated { .. })
    ));
}

#[test]
fn test_acp_tool_call_update_merges_existing_call() {
    use agent_client_protocol::schema::{
        ToolCall, ToolCallStatus, ToolCallUpdate, ToolCallUpdateFields, ToolKind,
    };

    let mut call = AcpToolCall::from_acp(
        ToolCall::new("read-1", "Read SKILL.md")
            .kind(ToolKind::Read)
            .status(ToolCallStatus::InProgress),
    );

    call.apply_update(ToolCallUpdate::new(
        "read-1",
        ToolCallUpdateFields::new().status(ToolCallStatus::Completed),
    ));

    assert_eq!(call.id.as_str(), "read-1");
    assert_eq!(call.title, "Read SKILL.md");
    assert_eq!(call.kind, ToolKind::Read);
    assert_eq!(call.status, ToolCallStatus::Completed);
}

#[test]
fn test_acp_tool_call_preserves_meta_without_terminal_trace_inference() {
    use agent_client_protocol::schema::{
        Terminal, ToolCall, ToolCallContent, ToolCallUpdate, ToolCallUpdateFields, ToolKind,
    };
    use serde_json::{json, Map};

    let mut initial_meta = Map::new();
    initial_meta.insert(
        "terminal_info".to_string(),
        json!({
            "command": "cargo test -p warp --lib ai::acp",
            "cwd": "/repo"
        }),
    );
    let mut call = AcpToolCall::from_acp(
        ToolCall::new("exec-1", "Run tests")
            .kind(ToolKind::Execute)
            .content(vec![ToolCallContent::Terminal(Terminal::new("term-1"))])
            .meta(initial_meta),
    );

    let mut stdout_meta = Map::new();
    stdout_meta.insert(
        "terminal_output".to_string(),
        json!({
            "stream": "stdout",
            "chunk": "running 1 test\n"
        }),
    );
    call.apply_update(ToolCallUpdate::new("exec-1", ToolCallUpdateFields::new()).meta(stdout_meta));

    let mut stderr_meta = Map::new();
    stderr_meta.insert(
        "terminal_output".to_string(),
        json!({
            "stream": "stderr",
            "chunk": "warning: existing warning\n"
        }),
    );
    call.apply_update(ToolCallUpdate::new("exec-1", ToolCallUpdateFields::new()).meta(stderr_meta));

    let mut exit_meta = Map::new();
    exit_meta.insert("terminal_exit".to_string(), json!({ "exit_code": 0 }));
    call.apply_update(ToolCallUpdate::new("exec-1", ToolCallUpdateFields::new()).meta(exit_meta));

    assert!(call.meta.is_some());
    assert!(call.terminal_traces.is_empty());
}

#[test]
fn test_default_config_options_only_applies_valid_select_values() {
    let options = vec![
        SessionConfigOption::select(
            "model",
            "Model",
            "gpt-5.5",
            vec![
                SessionConfigSelectOption::new("gpt-5.5", "GPT-5.5"),
                SessionConfigSelectOption::new("gpt-5.4", "GPT-5.4"),
            ],
        )
        .category(SessionConfigOptionCategory::Model),
        SessionConfigOption::select(
            "reasoning_effort",
            "Reasoning Effort",
            "xhigh",
            vec![SessionConfigSelectGroup::new(
                "reasoning",
                "Reasoning",
                vec![
                    SessionConfigSelectOption::new("medium", "Medium"),
                    SessionConfigSelectOption::new("xhigh", "XHigh"),
                ],
            )],
        )
        .category(SessionConfigOptionCategory::ThoughtLevel),
    ];
    let defaults = HashMap::from([
        ("model".to_string(), "gpt-5.4".to_string()),
        ("reasoning_effort".to_string(), "invalid".to_string()),
        ("missing".to_string(), "ignored".to_string()),
    ]);

    let actual = default_config_options_to_apply(&options, &defaults);
    let actual: Vec<_> = actual
        .iter()
        .map(|(id, value)| (id.0.to_string(), value.0.to_string()))
        .collect();

    assert_eq!(actual, vec![("model".to_string(), "gpt-5.4".to_string())]);
}

#[test]
fn test_flatten_config_options_preserves_select_values() {
    let options = vec![
        SessionConfigOption::select(
            "model",
            "Model",
            "gpt-5.5",
            vec![
                SessionConfigSelectOption::new("gpt-5.5", "GPT-5.5"),
                SessionConfigSelectOption::new("gpt-5.4", "GPT-5.4"),
            ],
        )
        .category(SessionConfigOptionCategory::Model),
        SessionConfigOption::select(
            "reasoning_effort",
            "Reasoning Effort",
            "xhigh",
            vec![SessionConfigSelectGroup::new(
                "reasoning",
                "Reasoning",
                vec![
                    SessionConfigSelectOption::new("medium", "Medium"),
                    SessionConfigSelectOption::new("xhigh", "XHigh"),
                ],
            )],
        )
        .category(SessionConfigOptionCategory::ThoughtLevel),
    ];

    let actual = flatten_config_options(&options);

    assert_eq!(actual.len(), 2);
    assert_eq!(actual[0].id, "model");
    assert_eq!(actual[0].name, "Model");
    assert_eq!(actual[0].category, Some(SessionConfigOptionCategory::Model));
    assert_eq!(actual[0].current_value, "gpt-5.5");
    assert_eq!(
        actual[0]
            .values
            .iter()
            .map(|value| (value.id.as_str(), value.name.as_str()))
            .collect::<Vec<_>>(),
        vec![("gpt-5.5", "GPT-5.5"), ("gpt-5.4", "GPT-5.4")]
    );
    assert_eq!(actual[1].id, "reasoning_effort");
    assert_eq!(
        actual[1].category,
        Some(SessionConfigOptionCategory::ThoughtLevel)
    );
    assert_eq!(
        actual[1]
            .values
            .iter()
            .map(|value| (value.id.as_str(), value.name.as_str()))
            .collect::<Vec<_>>(),
        vec![("medium", "Medium"), ("xhigh", "XHigh")]
    );
}
