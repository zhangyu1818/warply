use warp_terminal::model::BlockId;

use super::*;
use crate::agent::action::{AIAgentActionType, FileEdit, ReadFilesRequest, SearchCodebaseRequest};
use crate::agent::action_result::{
    AIAgentActionResultType, ReadFilesResult, ReadShellCommandOutputResult,
    RequestCommandOutputResult, RequestFileEditsResult,
};
use crate::hooks::adapters::cursor::events::CursorEvent;
use crate::hooks::adapters::cursor::test_support::{
    binding_for, completed_shell, fixture_envelope, shell_action, subagent_start, tool_failure,
    tool_pre, tool_result,
};
use crate::hooks::model::{HookPayload, PromptProvenance, TurnStopStatus};

fn binding_with_matcher(event: CursorEvent, matcher: Option<&str>) -> DeclaredBinding {
    let mut binding = binding_for(event, false);
    binding.matcher = matcher.map(str::to_owned);
    binding
}

fn matches(event: CursorEvent, matcher: Option<&str>, payload: HookPayload) -> bool {
    let sources = SourceCache::default();
    binding_matches(
        &sources,
        event.spec(),
        &binding_with_matcher(event, matcher),
        &fixture_envelope(payload),
    )
}

fn shell_pre(command: &str) -> HookPayload {
    tool_pre(shell_action(command))
}

#[test]
fn empty_star_and_missing_matchers_match_everything() {
    for matcher in [None, Some(""), Some("*")] {
        assert!(matches(
            CursorEvent::BeforeShellExecution,
            matcher,
            shell_pre("anything at all")
        ));
        assert!(matches(
            CursorEvent::Stop,
            matcher,
            HookPayload::TurnStop {
                status: TurnStopStatus::Completed,
                continuation_count: 0,
            }
        ));
    }
    assert!(matches_everything(""));
    assert!(matches_everything("*"));
    assert!(!matches_everything(".*"));
}

#[test]
fn a_matcher_is_an_unanchored_case_sensitive_regex() {
    let event = CursorEvent::BeforeShellExecution;
    assert!(matches(
        event,
        Some("rm -rf"),
        shell_pre("sudo rm -rf /tmp/x")
    ));
    assert!(!matches(event, Some("rm -rf"), shell_pre("RM -RF /tmp/x")));
    assert!(matches(
        event,
        Some("^git (push|commit)"),
        shell_pre("git push origin main")
    ));
    assert!(!matches(
        event,
        Some("^git (push|commit)"),
        shell_pre("echo git push")
    ));
    assert!(matches(event, Some("push$"), shell_pre("git push")));
    assert!(!matches(
        event,
        Some("push$"),
        shell_pre("git push --force")
    ));
}

#[test]
fn pre_tool_use_matches_the_tool_name() {
    let event = CursorEvent::PreToolUse;
    let cases: Vec<(&str, AIAgentActionType)> = vec![
        ("Shell", shell_action("ls")),
        (
            "Read",
            AIAgentActionType::ReadFiles(ReadFilesRequest {
                locations: Vec::new(),
            }),
        ),
        (
            "Write",
            AIAgentActionType::RequestFileEdits {
                file_edits: vec![FileEdit::Create {
                    file: Some("a.rs".to_owned()),
                    content: Some(String::new()),
                }],
                title: None,
            },
        ),
        (
            "Grep",
            AIAgentActionType::Grep {
                queries: vec!["x".to_owned()],
                path: ".".to_owned(),
            },
        ),
        (
            "SearchCodebase",
            AIAgentActionType::SearchCodebase(SearchCodebaseRequest {
                query: "q".to_owned(),
                partial_paths: None,
                codebase_path: None,
            }),
        ),
        ("InitProject", AIAgentActionType::InitProject),
    ];
    for (name, action) in cases {
        let exact = format!("^{}$", regex::escape(name));
        assert!(
            matches(event, Some(&exact), tool_pre(action.clone())),
            "{name} should match {exact}"
        );
        assert!(
            !matches(event, Some("^NoSuchTool$"), tool_pre(action.clone())),
            "{name} should not match another name"
        );
        assert!(matches(event, Some("*"), tool_pre(action)));
    }
    assert!(matches(
        event,
        Some("Shell|Write"),
        tool_pre(shell_action("ls"))
    ));
}

#[test]
fn post_tool_events_match_the_tool_name_too() {
    let result = tool_result(shell_action("ls"), completed_shell("ls", "", 0), None);
    assert!(matches(
        CursorEvent::PostToolUse,
        Some("^Shell$"),
        result.clone()
    ));
    assert!(!matches(CursorEvent::PostToolUse, Some("^Read$"), result));

    let failure = tool_failure(
        AIAgentActionType::ReadFiles(ReadFilesRequest {
            locations: Vec::new(),
        }),
        AIAgentActionResultType::ReadFiles(ReadFilesResult::Error("boom".to_owned())),
        None,
    );
    assert!(matches(
        CursorEvent::PostToolUseFailure,
        Some("^Read$"),
        failure
    ));
}

#[test]
fn shell_events_match_the_command_text_and_only_shell_commands() {
    let pre = CursorEvent::BeforeShellExecution;
    assert!(matches(pre, Some("^rm "), shell_pre("rm -rf x")));
    assert!(!matches(pre, Some("^rm "), shell_pre("ls")));
    assert!(!matches(
        pre,
        None,
        tool_pre(AIAgentActionType::ReadFiles(ReadFilesRequest {
            locations: Vec::new(),
        }))
    ));

    let after = CursorEvent::AfterShellExecution;
    let finished = tool_result(
        shell_action("cargo test"),
        completed_shell("cargo test", "ok", 0),
        None,
    );
    assert!(matches(after, Some("^cargo"), finished.clone()));
    assert!(!matches(after, Some("^git"), finished));

    let snapshot = tool_result(
        shell_action("tail -f log"),
        AIAgentActionResultType::RequestCommandOutput(
            RequestCommandOutputResult::LongRunningCommandSnapshot {
                block_id: BlockId::from("b".to_owned()),
                command: "tail -f log".to_owned(),
                grid_contents: String::new(),
                cursor: String::new(),
                is_alt_screen_active: false,
                activity: None,
            },
        ),
        None,
    );
    assert!(
        !matches(after, None, snapshot),
        "a still-running command has not finished"
    );
    let denied = tool_failure(
        shell_action("rm -rf /"),
        AIAgentActionResultType::DeniedByHook {
            reason: "no".to_owned(),
        },
        None,
    );
    assert!(
        !matches(after, None, denied),
        "a command that never ran did not finish"
    );
    // The real exit of an agent-started command is reported once, on the command's own action;
    // the poll that happened to see it is a `postToolUse` of `ShellRead`.
    let poll = tool_result(
        AIAgentActionType::ReadShellCommandOutput {
            block_id: BlockId::from("b".to_owned()),
            delay: None,
        },
        AIAgentActionResultType::ReadShellCommandOutput(
            ReadShellCommandOutputResult::CommandFinished {
                command: "cargo test".to_owned(),
                block_id: BlockId::from("b".to_owned()),
                output: "ok".to_owned(),
                exit_code: 0.into(),
            },
        ),
        None,
    );
    assert!(!matches(after, None, poll.clone()));
    assert!(matches(CursorEvent::PostToolUse, Some("^ShellRead$"), poll));
}

#[test]
fn mcp_events_never_match_without_mcp_tool_calls() {
    // The fork has no app-side MCP tool calls, so Cursor's MCP events match nothing here.
    assert!(!matches(
        CursorEvent::BeforeMcpExecution,
        None,
        shell_pre("ls")
    ));
    assert!(!matches(
        CursorEvent::AfterMcpExecution,
        None,
        shell_pre("ls")
    ));
}

#[test]
fn literal_events_match_their_fixed_word() {
    let stop = HookPayload::TurnStop {
        status: TurnStopStatus::Completed,
        continuation_count: 0,
    };
    assert!(matches(CursorEvent::Stop, Some("^Stop$"), stop.clone()));
    assert!(!matches(CursorEvent::Stop, Some("^stop$"), stop.clone()));
    assert!(!matches(CursorEvent::Stop, Some("Shell"), stop));

    let prompt = HookPayload::PromptSubmit {
        text: "hi".to_owned(),
        attachments: Vec::new(),
        provenance: PromptProvenance::User,
    };
    assert!(matches(
        CursorEvent::BeforeSubmitPrompt,
        Some("UserPromptSubmit"),
        prompt
    ));
    let response = HookPayload::ResponseComplete {
        message_id: "m".to_owned(),
        text: "t".to_owned(),
    };
    assert!(matches(
        CursorEvent::AfterAgentResponse,
        Some("^AgentResponse$"),
        response
    ));
    let thought = HookPayload::ReasoningComplete {
        message_id: "m".to_owned(),
        text: "t".to_owned(),
        duration: None,
    };
    assert!(matches(
        CursorEvent::AfterAgentThought,
        Some("AgentThought"),
        thought
    ));
}

#[test]
fn subagent_start_matches_on_cursors_general_purpose_type() {
    let launch = subagent_start();
    assert!(matches(CursorEvent::SubagentStart, None, launch.clone()));
    assert!(matches(
        CursorEvent::SubagentStart,
        Some("^generalPurpose$"),
        launch.clone()
    ));
    assert!(
        !matches(CursorEvent::SubagentStart, Some("explore"), launch.clone()),
        "every child Warp launches is a general-purpose worker"
    );
    assert!(
        !matches(CursorEvent::SubagentStop, None, launch),
        "subagentStop is not fired yet"
    );
    assert!(!matches(CursorEvent::SubagentStart, None, shell_pre("ls")));
}

#[test]
fn events_without_a_matcher_target_match_every_occurrence() {
    let start = HookPayload::SessionStart {
        is_background_agent: false,
    };
    assert!(matches(CursorEvent::SessionStart, None, start.clone()));
    assert!(matches(CursorEvent::SessionStart, Some("*"), start));
    assert_eq!(
        matched_value(
            CursorEvent::SessionStart.spec(),
            &fixture_envelope(HookPayload::SessionStart {
                is_background_agent: true,
            })
        )
        .as_deref(),
        Some("")
    );
}

#[test]
fn a_binding_never_matches_an_envelope_of_another_boundary() {
    let result = tool_result(shell_action("ls"), completed_shell("ls", "", 0), None);
    assert!(!matches(
        CursorEvent::BeforeShellExecution,
        None,
        result.clone()
    ));
    assert!(!matches(CursorEvent::PreToolUse, None, result));
    assert!(!matches(
        CursorEvent::AfterShellExecution,
        None,
        shell_pre("ls")
    ));
    assert!(!matches(CursorEvent::PostToolUse, None, shell_pre("ls")));
    assert!(!matches(
        CursorEvent::Stop,
        None,
        HookPayload::SessionStart {
            is_background_agent: false,
        }
    ));
}

#[test]
fn events_warp_does_not_fire_yet_never_match_even_on_their_boundary() {
    let edit = tool_result(
        AIAgentActionType::RequestFileEdits {
            file_edits: Vec::new(),
            title: None,
        },
        AIAgentActionResultType::RequestFileEdits(RequestFileEditsResult::Cancelled),
        None,
    );
    assert!(matches(CursorEvent::PostToolUse, None, edit.clone()));
    assert!(
        !matches(CursorEvent::AfterFileEdit, None, edit),
        "afterFileEdit is a Phase 2 facet of tool.result"
    );
}

#[test]
fn grep_is_a_tool_name_not_a_command() {
    let grep = tool_pre(AIAgentActionType::Grep {
        queries: vec!["rm".to_owned()],
        path: ".".to_owned(),
    });
    assert!(matches(
        CursorEvent::PreToolUse,
        Some("^Grep$"),
        grep.clone()
    ));
    assert!(!matches(
        CursorEvent::BeforeShellExecution,
        Some("rm"),
        grep
    ));
}
