//! Views of an envelope payload for the shell and MCP facets of the tool boundaries. A facet is
//! one vendor event that projects onto a shared boundary and sees only part of its payload
//! (architecture.md §4.1): Cursor's shell and MCP events sit beside `preToolUse` and
//! `postToolUse` on `tool.pre` and `tool.result`.

use chrono::{DateTime, Local};
use uuid::Uuid;

use crate::agent::action::AIAgentActionType;
use crate::agent::action_result::{AIAgentActionResultType, RequestCommandOutputResult};
use crate::hooks::model::HookPayload;

pub(super) fn pending_shell_command(payload: &HookPayload) -> Option<&str> {
    match payload {
        HookPayload::ToolPre {
            action: AIAgentActionType::RequestCommandOutput { command, .. },
            ..
        } => Some(command),
        HookPayload::ToolPre { .. }
        | HookPayload::SessionStart { .. }
        | HookPayload::PromptSubmit { .. }
        | HookPayload::ToolResult { .. }
        | HookPayload::ToolFailure { .. }
        | HookPayload::TurnStop { .. }
        | HookPayload::ResponseComplete { .. }
        | HookPayload::ReasoningComplete { .. } => None,
    }
}

/// A shell command that ran to completion, as a tool result reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FinishedShellCommand<'a> {
    pub command: &'a str,
    pub output: &'a str,
    /// Absent when the shell itself ended before reporting a code.
    pub exit_code: Option<i32>,
    pub start_ts: Option<DateTime<Local>>,
    pub completed_ts: Option<DateTime<Local>>,
}

/// The finished shell command in a result payload. A snapshot of a still-running command and a
/// command that never started are not finished commands. Neither is a poll, write, or handoff
/// that saw a long-running command end: the completion funnel reports that exit once more, on
/// the command's own action, so the command is counted once.
pub(super) fn finished_shell_command(payload: &HookPayload) -> Option<FinishedShellCommand<'_>> {
    let result = match payload {
        HookPayload::ToolResult { result, .. } | HookPayload::ToolFailure { result, .. } => result,
        HookPayload::ToolPre { .. }
        | HookPayload::SessionStart { .. }
        | HookPayload::PromptSubmit { .. }
        | HookPayload::TurnStop { .. }
        | HookPayload::ResponseComplete { .. }
        | HookPayload::ReasoningComplete { .. } => return None,
    };
    match result {
        AIAgentActionResultType::RequestCommandOutput(result) => match result {
            RequestCommandOutputResult::Completed {
                command,
                output,
                exit_code,
                block_id: _,
            } => Some(FinishedShellCommand {
                command,
                output,
                exit_code: Some(exit_code.value()),
                // The fork's command results carry no timestamps.
                start_ts: None,
                completed_ts: None,
            }),
            RequestCommandOutputResult::LongRunningCommandSnapshot { .. }
            | RequestCommandOutputResult::CancelledBeforeExecution
            | RequestCommandOutputResult::Denylisted { .. }
            | RequestCommandOutputResult::TerminalBusy { .. } => None,
        },
        AIAgentActionResultType::ReadShellCommandOutput(_)
        | AIAgentActionResultType::WriteToLongRunningShellCommand(_)
        | AIAgentActionResultType::TransferShellCommandControlToUser(_) => None,
        AIAgentActionResultType::RequestFileEdits(_)
        | AIAgentActionResultType::ReadFiles(_)
        | AIAgentActionResultType::SearchCodebase(_)
        | AIAgentActionResultType::Grep(_)
        | AIAgentActionResultType::FileGlobV2(_)
        | AIAgentActionResultType::OpenCodeReview
        | AIAgentActionResultType::InitProject
        | AIAgentActionResultType::ReadDocuments(_)
        | AIAgentActionResultType::EditDocuments(_)
        | AIAgentActionResultType::CreateDocuments(_)
        | AIAgentActionResultType::UseComputer(_)
        | AIAgentActionResultType::InsertReviewComments(_)
        | AIAgentActionResultType::RequestComputerUse(_)
        | AIAgentActionResultType::AskUserQuestion(_)
        | AIAgentActionResultType::DeniedByHook { .. } => None,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct McpToolCall<'a> {
    pub server_id: Option<Uuid>,
    pub name: &'a str,
    pub input: &'a serde_json::Value,
    /// Present on the result boundaries.
    pub outcome: Option<McpCallOutcome<'a>>,
}

/// How an MCP tool call that ran ended. A cancelled call never ran, so the MCP events do not see
/// it and there is no variant for it.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum McpCallOutcome<'a> {
    Returned(&'a serde_json::Value),
    Failed(&'a str),
}

/// The fork has no app-side MCP tool calls (MCP belongs to the ACP agent process), so no payload
/// is ever an MCP call: Cursor's MCP events parse and report but never fire here.
pub(super) fn mcp_tool_call(_payload: &HookPayload) -> Option<McpToolCall<'_>> {
    None
}
