use std::ops::Range;
use std::path::PathBuf;
use std::time::Duration;

use serde_json::{Value, json};
use warp_terminal::model::BlockId;

use super::*;
use crate::agent::FileLocations;
use crate::agent::action::{AIAgentActionType, FileEdit, ReadFilesRequest};
use crate::agent::action_result::{
    AIAgentActionResultType, AnyFileContent, FileContext, ReadFilesResult,
    RequestCommandOutputResult,
};
use crate::diff_validation::ParsedDiff;
use crate::hooks::adapters::cursor::test_support::{
    EVENT_FIXTURES, FIXTURE_CONVERSATION_ID, completed_shell, fixture_envelope, json, shell_action,
    tool_failure, tool_pre, tool_result,
};
use crate::hooks::model::{
    HookTurnId, PromptAttachment, PromptAttachmentKind, PromptProvenance, TurnStopStatus,
};

/// The payload each `stdin.json` fixture was written for.
fn fixture_payload(event: CursorEvent) -> HookPayload {
    match event {
        CursorEvent::SessionStart => HookPayload::SessionStart {
            is_background_agent: false,
        },
        CursorEvent::PreToolUse => tool_pre(AIAgentActionType::RequestCommandOutput {
            command: "rm -rf build".to_owned(),
            is_read_only: Some(false),
            is_risky: Some(true),
            wait_until_completion: true,
            uses_pager: None,
            rationale: Some("Clean the build directory".to_owned()),
            citations: Vec::new(),
        }),
        CursorEvent::PostToolUse => tool_result(
            shell_action("cargo test"),
            completed_shell("cargo test", "test result: ok", 0),
            Some(Duration::from_millis(1500)),
        ),
        CursorEvent::PostToolUseFailure => tool_failure(
            shell_action("rm -rf /"),
            AIAgentActionResultType::DeniedByHook {
                reason: "not allowed here".to_owned(),
            },
            None,
        ),
        CursorEvent::BeforeShellExecution => tool_pre(shell_action("git push --force")),
        CursorEvent::AfterShellExecution => tool_result(
            shell_action("cargo test"),
            completed_shell("cargo test", "test result: ok", 0),
            Some(Duration::from_millis(1500)),
        ),
        // The fork has no app-side MCP tool calls, so the MCP stdin fixtures stay on disk
        // without a payload this build can construct for them.
        CursorEvent::BeforeMcpExecution | CursorEvent::AfterMcpExecution => {
            panic!("{} has no fork-buildable payload", event.name())
        }
        CursorEvent::BeforeSubmitPrompt => HookPayload::PromptSubmit {
            text: "Fix the failing test".to_owned(),
            attachments: vec![
                PromptAttachment {
                    kind: PromptAttachmentKind::File,
                    path: PathBuf::from("/work/repo/src/lib.rs"),
                },
                PromptAttachment {
                    kind: PromptAttachmentKind::Rule,
                    path: PathBuf::from("/work/repo/AGENTS.md"),
                },
            ],
            provenance: PromptProvenance::User,
        },
        CursorEvent::Stop => HookPayload::TurnStop {
            status: TurnStopStatus::Completed,
            continuation_count: 0,
        },
        CursorEvent::AfterAgentResponse => HookPayload::ResponseComplete {
            message_id: "m1".to_owned(),
            text: "Done. The tests pass.".to_owned(),
        },
        CursorEvent::AfterAgentThought => HookPayload::ReasoningComplete {
            message_id: "m0".to_owned(),
            text: "Let me check the tests first.".to_owned(),
            duration: Some(Duration::from_secs(2)),
        },
        CursorEvent::SessionEnd
        | CursorEvent::SubagentStart
        | CursorEvent::SubagentStop
        | CursorEvent::BeforeReadFile
        | CursorEvent::AfterFileEdit
        | CursorEvent::PreCompact
        | CursorEvent::BeforeTabFileRead
        | CursorEvent::AfterTabFileEdit
        | CursorEvent::WorkspaceOpen => panic!("{} has no fixture", event.name()),
    }
}

fn encode(event: CursorEvent, payload: HookPayload) -> Value {
    encode_input(event.spec(), &fixture_envelope(payload))
}

/// Every input for the firing, for actions a Cursor script sees as several.
fn encode_all(event: CursorEvent, payload: HookPayload) -> Vec<HookInput> {
    encode_inputs(event.spec(), &fixture_envelope(payload), |_| true)
}

/// Cursor sends some fields as JSON text inside a string. Their key order is a serializer
/// detail, so they are compared as parsed JSON.
fn with_nested_json_parsed(mut value: Value) -> Value {
    if let Some(object) = value.as_object_mut() {
        for key in ["tool_input", "tool_output", "result_json"] {
            let parsed = object
                .get(key)
                .and_then(Value::as_str)
                .and_then(|text| serde_json::from_str::<Value>(text).ok());
            if let Some(parsed) = parsed {
                object.insert(key.to_owned(), parsed);
            }
        }
    }
    value
}

#[test]
fn every_phase_1_event_encodes_exactly_its_fixture() {
    for fixture in &EVENT_FIXTURES {
        // The MCP fixtures describe app-side MCP tool calls, which the fork never produces;
        // their stdin.json files stay on disk unexercised.
        if matches!(
            fixture.event,
            CursorEvent::BeforeMcpExecution | CursorEvent::AfterMcpExecution
        ) {
            continue;
        }
        let encoded = encode(fixture.event, fixture_payload(fixture.event));
        assert_eq!(
            with_nested_json_parsed(encoded),
            with_nested_json_parsed(json(fixture.stdin)),
            "{} stdin differs from fixtures/{}/stdin.json",
            fixture.event.name(),
            fixture.event.name()
        );
    }
}

#[test]
fn generation_id_is_the_turn_id_once_one_exists() {
    let mut envelope = fixture_envelope(tool_pre(shell_action("ls")));
    let turn = HookTurnId::new();
    envelope.turn_id = Some(turn);
    let input = encode_input(CursorEvent::BeforeShellExecution.spec(), &envelope);
    assert_eq!(input["generation_id"], turn.as_uuid().to_string());
    assert_eq!(input["conversation_id"], FIXTURE_CONVERSATION_ID);
}

#[test]
fn generation_id_falls_back_to_the_conversation_id_before_a_turn_exists() {
    let mut envelope = fixture_envelope(HookPayload::SessionStart {
        is_background_agent: false,
    });
    envelope.turn_id = None;
    for event in [
        CursorEvent::SessionStart,
        CursorEvent::BeforeShellExecution,
        CursorEvent::Stop,
    ] {
        let input = encode_input(event.spec(), &envelope);
        assert_eq!(
            input["generation_id"],
            FIXTURE_CONVERSATION_ID,
            "{}: the key is always present and never a made-up value",
            event.name()
        );
    }
}

#[test]
fn unknown_common_fields_are_absent_or_null_rather_than_invented() {
    let mut envelope = fixture_envelope(tool_pre(shell_action("ls")));
    envelope.model_id = None;
    envelope.user_email = None;
    envelope.action_id = None;
    envelope.roots = Vec::new();
    let input = encode_input(CursorEvent::BeforeShellExecution.spec(), &envelope);
    let object = input.as_object().expect("an object");
    assert!(!object.contains_key("model"));
    assert!(!object.contains_key("model_id"));
    assert!(!object.contains_key("tool_use_id"));
    assert_eq!(input["user_email"], Value::Null);
    assert_eq!(input["transcript_path"], Value::Null);
    assert_eq!(input["model_params"], json!([]));
    assert_eq!(
        input["workspace_roots"],
        json!(["/work/repo/src"]),
        "with no detected root the working directory is the workspace"
    );

    envelope.cwd = None;
    let input = encode_input(CursorEvent::BeforeShellExecution.spec(), &envelope);
    let object = input.as_object().expect("an object");
    assert_eq!(input["workspace_roots"], json!([]));
    assert!(!object.contains_key("cwd"));
    assert_eq!(input["command"], "ls");
}

#[test]
fn cursor_version_and_warp_version_both_carry_warps_version() {
    let input = encode(
        CursorEvent::Stop,
        HookPayload::TurnStop {
            status: TurnStopStatus::Error,
            continuation_count: 2,
        },
    );
    assert_eq!(input["cursor_version"], "v0.2026.01.01");
    assert_eq!(input["warp_version"], "v0.2026.01.01");
    assert_eq!(input["status"], "error");
    assert_eq!(input["loop_count"], 2);
    assert_eq!(
        encode(
            CursorEvent::Stop,
            HookPayload::TurnStop {
                status: TurnStopStatus::Cancelled,
                continuation_count: 0,
            },
        )["status"],
        "aborted"
    );
}

#[test]
fn after_shell_execution_reports_the_exit_code_when_nothing_measured_a_duration() {
    // The fork's command results carry no timestamps, so no duration is ever derived from them.
    let result =
        AIAgentActionResultType::RequestCommandOutput(RequestCommandOutputResult::Completed {
            block_id: BlockId::from("b".to_owned()),
            command: "make".to_owned(),
            output: "built".to_owned(),
            exit_code: 1.into(),
        });
    let input = encode(
        CursorEvent::AfterShellExecution,
        tool_result(shell_action("make"), result, None),
    );
    let object = input.as_object().expect("an object");
    assert_eq!(input["exit_code"], 1);
    assert_eq!(input["output"], "built");
    assert!(!object.contains_key("duration"));
}

#[test]
fn a_cancelled_tool_is_an_interrupt_with_its_reason() {
    let input = encode(
        CursorEvent::PostToolUseFailure,
        tool_failure(
            shell_action("sleep 100"),
            AIAgentActionResultType::RequestCommandOutput(
                RequestCommandOutputResult::CancelledBeforeExecution,
            ),
            Some("user_cancelled"),
        ),
    );
    assert_eq!(input["failure_type"], "error");
    assert_eq!(input["is_interrupt"], true);
    assert_eq!(input["cancellation_reason"], "user_cancelled");
    assert_eq!(input["error_message"], "Command output cancelled");
}

#[test]
fn tool_input_describes_non_shell_actions() {
    let read = tool_pre(AIAgentActionType::ReadFiles(ReadFilesRequest {
        locations: vec![FileLocations {
            name: "src/lib.rs".to_owned(),
            lines: vec![Range { start: 1, end: 20 }],
        }],
    }));
    let input = encode(CursorEvent::PreToolUse, read);
    assert_eq!(input["tool_name"], "Read");
    assert_eq!(
        input["tool_input"],
        json!({ "file_path": "src/lib.rs", "lines": [[1, 20]] }),
        "a read has Cursor's single-file shape"
    );

    let edit = tool_pre(AIAgentActionType::RequestFileEdits {
        file_edits: vec![
            FileEdit::Create {
                file: Some("new.rs".to_owned()),
                content: Some("fn main() {}".to_owned()),
            },
            FileEdit::Delete {
                file: Some("old.rs".to_owned()),
            },
        ],
        title: Some("Replace old.rs".to_owned()),
    });
    let inputs = encode_all(CursorEvent::PreToolUse, edit);
    assert_eq!(
        inputs.len(),
        2,
        "a batch over two files is one event per file"
    );
    assert_eq!(inputs[0].label.as_deref(), Some("new.rs"));
    assert_eq!(inputs[0].payload["tool_name"], "Write");
    assert_eq!(
        inputs[0].payload["tool_input"],
        json!({
            "file_path": "new.rs",
            "title": "Replace old.rs",
            "edits": [
                { "kind": "create", "file_path": "new.rs", "content": "fn main() {}" }
            ]
        })
    );
    assert_eq!(inputs[1].label.as_deref(), Some("old.rs"));
    assert_eq!(
        inputs[1].payload["tool_name"], "Delete",
        "the part made only of a delete is a Delete"
    );
    assert_eq!(
        inputs[1].payload["tool_input"],
        json!({
            "file_path": "old.rs",
            "title": "Replace old.rs",
            "edits": [ { "kind": "delete", "file_path": "old.rs" } ]
        })
    );
    assert_eq!(
        inputs[0].payload["tool_use_id"], inputs[1].payload["tool_use_id"],
        "both parts belong to the one tool call"
    );
}

#[test]
fn edits_to_one_file_name_it_with_cursors_field_names() {
    let edit = tool_pre(AIAgentActionType::RequestFileEdits {
        file_edits: vec![
            FileEdit::Edit(ParsedDiff::StrReplaceEdit {
                file: Some("src/lib.rs".to_owned()),
                search: Some("a".to_owned()),
                replace: Some("b".to_owned()),
            }),
            FileEdit::Edit(ParsedDiff::StrReplaceEdit {
                file: Some("src/lib.rs".to_owned()),
                search: Some("c".to_owned()),
                replace: Some("d".to_owned()),
            }),
        ],
        title: None,
    });
    let input = encode(CursorEvent::PreToolUse, edit);
    assert_eq!(input["tool_name"], "Write");
    assert_eq!(input["tool_input"]["file_path"], "src/lib.rs");
    assert_eq!(
        input["tool_input"]["edits"][1],
        json!({ "kind": "edit", "file_path": "src/lib.rs", "old_string": "c", "new_string": "d" })
    );

    let delete = tool_pre(AIAgentActionType::RequestFileEdits {
        file_edits: vec![FileEdit::Delete {
            file: Some("old.rs".to_owned()),
        }],
        title: None,
    });
    let input = encode(CursorEvent::PreToolUse, delete);
    assert_eq!(input["tool_name"], "Delete");
    assert_eq!(input["tool_input"]["file_path"], "old.rs");

    let nameless = tool_pre(AIAgentActionType::RequestFileEdits {
        file_edits: vec![FileEdit::Create {
            file: None,
            content: None,
        }],
        title: None,
    });
    let input = encode(CursorEvent::PreToolUse, nameless);
    assert!(
        input["tool_input"].get("file_path").is_none(),
        "an edit with no file names none"
    );
}

#[test]
fn a_read_of_several_files_is_one_event_per_file_with_that_files_output() {
    let request = ReadFilesRequest {
        locations: vec![
            FileLocations {
                name: "src/a.rs".to_owned(),
                lines: Vec::new(),
            },
            FileLocations {
                name: "missing.txt".to_owned(),
                lines: Vec::new(),
            },
        ],
    };
    let result = AIAgentActionResultType::ReadFiles(ReadFilesResult::Success {
        files: vec![FileContext::new(
            "/work/repo/src/a.rs".to_owned(),
            AnyFileContent::StringContent("fn a() {}".to_owned()),
            None,
            None,
        )],
    });

    let pre = encode_all(
        CursorEvent::PreToolUse,
        tool_pre(AIAgentActionType::ReadFiles(request.clone())),
    );
    assert_eq!(pre.len(), 2);
    assert_eq!(pre[0].label.as_deref(), Some("src/a.rs"));
    assert_eq!(pre[0].payload["tool_name"], "Read");
    assert_eq!(
        pre[0].payload["tool_input"],
        json!({ "file_path": "src/a.rs", "lines": [] })
    );
    assert_eq!(pre[1].label.as_deref(), Some("missing.txt"));

    let post = encode_all(
        CursorEvent::PostToolUse,
        tool_result(AIAgentActionType::ReadFiles(request), result, None),
    );
    assert_eq!(post.len(), 2);
    let first: Value =
        serde_json::from_str(post[0].payload["tool_output"].as_str().expect("a string"))
            .expect("JSON");
    assert_eq!(first["kind"], "success");
    assert_eq!(first["files"][0]["content"], "fn a() {}");
    assert_eq!(first["files"].as_array().map(Vec::len), Some(1));
    let second: Value =
        serde_json::from_str(post[1].payload["tool_output"].as_str().expect("a string"))
            .expect("JSON");
    assert_eq!(second["files"].as_array().map(Vec::len), Some(0));
}

#[test]
fn a_single_file_read_is_one_unlabeled_event() {
    let inputs = encode_all(
        CursorEvent::PreToolUse,
        tool_pre(AIAgentActionType::ReadFiles(ReadFilesRequest {
            locations: vec![FileLocations {
                name: "src/a.rs".to_owned(),
                lines: Vec::new(),
            }],
        })),
    );
    assert_eq!(inputs.len(), 1);
    assert_eq!(
        inputs[0].label, None,
        "nothing was split, so nothing is labeled"
    );
    assert_eq!(inputs[0].payload["tool_input"]["file_path"], "src/a.rs");
}

#[test]
fn tool_output_summarizes_binary_content_and_keeps_text() {
    let result = AIAgentActionResultType::ReadFiles(ReadFilesResult::Success {
        files: vec![
            FileContext::new(
                "a.txt".to_owned(),
                AnyFileContent::StringContent("hello\nworld".to_owned()),
                None,
                None,
            ),
            FileContext::new(
                "logo.png".to_owned(),
                AnyFileContent::BinaryContent(vec![0, 1, 2]),
                None,
                None,
            ),
        ],
    });
    let input = encode(
        CursorEvent::PostToolUse,
        tool_result(
            AIAgentActionType::ReadFiles(ReadFilesRequest {
                locations: Vec::new(),
            }),
            result,
            None,
        ),
    );
    let output: Value =
        serde_json::from_str(input["tool_output"].as_str().expect("a string")).expect("JSON");
    assert_eq!(output["kind"], "success");
    assert_eq!(output["files"][0]["content"], "hello\nworld");
    assert_eq!(output["files"][0]["line_count"], 2);
    assert_eq!(output["files"][1]["is_binary"], true);
    assert_eq!(output["files"][1]["size_bytes"], 3);
    assert!(output["files"][1].get("content").is_none());
    let object = input.as_object().expect("an object");
    assert!(!object.contains_key("duration"));
}

#[test]
fn events_warp_does_not_fire_encode_only_the_common_fields() {
    let input = encode(
        CursorEvent::SessionEnd,
        HookPayload::SessionStart {
            is_background_agent: true,
        },
    );
    let mut keys = input
        .as_object()
        .expect("an object")
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    keys.sort();
    assert_eq!(
        keys,
        vec![
            "conversation_id",
            "cursor_version",
            "generation_id",
            "hook_event_name",
            "model",
            "model_id",
            "model_params",
            "transcript_path",
            "user_email",
            "warp_version",
            "workspace_roots",
        ]
    );
    assert_eq!(input["hook_event_name"], "sessionEnd");
}

#[test]
fn an_event_given_a_payload_it_is_not_about_adds_nothing() {
    let input = encode(
        CursorEvent::BeforeShellExecution,
        tool_pre(AIAgentActionType::ReadFiles(ReadFilesRequest {
            locations: Vec::new(),
        })),
    );
    let object = input.as_object().expect("an object");
    assert!(!object.contains_key("command"));
    assert_eq!(input["hook_event_name"], "beforeShellExecution");
}
