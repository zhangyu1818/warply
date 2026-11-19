//! The completion funnel for tool boundaries. Every terminal result of an action passes through
//! here once. A tool that ran and returned is a `tool.result`, whatever it returned; a tool that
//! never ran (denied, cancelled, or stopped before it could start) is a `tool.failure`. The
//! result is never held; the dispatch runs detached.
//!
//! A command the agent runs in interact mode first produces a snapshot of the still-running
//! command, and its real exit is seen later by whichever poll, write, or handoff happens to be
//! waiting on it. The funnel remembers the command when the snapshot passes and, when the exit
//! is seen, reports it once more as a completed result of the command's own action, so a hook
//! watching for finished shell commands sees every command exactly once, with its exit code.

use std::collections::HashMap;
use std::time::Duration;

use ai::agent::action_result::{
    ReadShellCommandOutputResult, RequestCommandOutputResult,
    TransferShellCommandControlToUserResult, WriteToLongRunningShellCommandResult,
};
#[cfg(test)]
use ai::hooks::HookEnvelope;
use warp_core::command::ExitCode;
use warp_terminal::model::BlockId;
use warpui::{AppContext, ModelHandle, SingletonEntity};

use super::envelope::{EnvelopeSite, envelope_for_tool_failure, envelope_for_tool_result};
use super::{HookRuntime, executes_tool_calls_locally};
use crate::ai::agent::conversation::AIConversationId;
use crate::ai::agent::{
    AIAgentAction, AIAgentActionResultType, AIAgentActionType, CancellationReason,
};
use crate::terminal::model::session::active_session::ActiveSession;

/// What the funnel recorded about one terminal result, kept so tests can count calls per path.
#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObservedTerminalResult {
    pub action_id: crate::ai::agent::AIAgentActionId,
    pub boundary: ai::hooks::HookBoundary,
}

/// A long-running command whose snapshot was reported and whose exit has not been seen yet.
struct PendingShellCompletion {
    conversation_id: AIConversationId,
    action: AIAgentAction,
}

/// Long-running commands waiting for their real exit, by the block they run in.
#[derive(Default)]
pub(crate) struct ShellCompletions {
    pending: HashMap<BlockId, PendingShellCompletion>,
}

impl ShellCompletions {
    pub(super) fn forget(&mut self, conversation_id: AIConversationId) {
        self.pending
            .retain(|_, pending| pending.conversation_id != conversation_id);
    }

    #[cfg(test)]
    pub(crate) fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

/// Fires `tool.result` or `tool.failure` for a terminal result. Only conversations whose tool
/// calls this client runs fire hooks; a result mirrored from another client does not.
/// `duration` is how long the tool ran, from the moment the executor started it (after every
/// check and confirmation) to this result; `None` when the action never started.
pub(crate) fn observe_terminal_result(
    action: &AIAgentAction,
    result: &AIAgentActionResultType,
    cancellation_reason: Option<CancellationReason>,
    duration: Option<Duration>,
    conversation_id: AIConversationId,
    active_session: &ModelHandle<ActiveSession>,
    ctx: &mut AppContext,
) {
    if !HookRuntime::is_active(ctx) || !executes_tool_calls_locally(conversation_id, ctx) {
        return;
    }
    let site = EnvelopeSite::new(conversation_id, active_session);
    let envelope = if ran_and_returned(result) && cancellation_reason.is_none() {
        envelope_for_tool_result(action, result, duration, &site, ctx)
    } else {
        envelope_for_tool_failure(action, result, cancellation_reason, duration, &site, ctx)
    };
    #[cfg(test)]
    record_observation(action, &envelope, ctx);
    HookRuntime::as_ref(ctx).dispatch_detached(envelope, ctx);

    match shell_progress(result) {
        ShellProgress::Snapshot { block_id } => {
            HookRuntime::handle(ctx).update(ctx, |runtime, _| {
                runtime.shell_completions.pending.insert(
                    block_id,
                    PendingShellCompletion {
                        conversation_id,
                        action: action.clone(),
                    },
                );
            });
        }
        ShellProgress::Finished(finished) => {
            let pending = HookRuntime::handle(ctx).update(ctx, |runtime, _| {
                runtime.shell_completions.pending.remove(&finished.block_id)
            });
            if let Some(pending) = pending {
                report_shell_completion(pending, finished, active_session, ctx);
            }
        }
        ShellProgress::None => {}
    }
}

/// Fires the `tool.result` for the real exit of a long-running command, on the command's own
/// action and in the command's own conversation.
fn report_shell_completion(
    pending: PendingShellCompletion,
    finished: FinishedCommand,
    active_session: &ModelHandle<ActiveSession>,
    ctx: &mut AppContext,
) {
    let AIAgentActionType::RequestCommandOutput { command, .. } = &pending.action.action else {
        return;
    };
    // The fork's command results carry no timestamps, so the envelope's duration stays empty.
    let duration = None;
    let result =
        AIAgentActionResultType::RequestCommandOutput(RequestCommandOutputResult::Completed {
            block_id: finished.block_id,
            command: command.clone(),
            output: finished.output,
            exit_code: finished.exit_code,
        });
    let site = EnvelopeSite::new(pending.conversation_id, active_session);
    let envelope = envelope_for_tool_result(&pending.action, &result, duration, &site, ctx);
    #[cfg(test)]
    record_observation(&pending.action, &envelope, ctx);
    HookRuntime::as_ref(ctx).dispatch_detached(envelope, ctx);
}

/// A long-running command's exit as one of the shell actions reported it.
struct FinishedCommand {
    block_id: BlockId,
    output: String,
    exit_code: ExitCode,
}

/// What a result says about a long-running command.
enum ShellProgress {
    /// The command is still running; its exit will be reported later.
    Snapshot {
        block_id: BlockId,
    },
    /// The command exited.
    Finished(FinishedCommand),
    None,
}

fn shell_progress(result: &AIAgentActionResultType) -> ShellProgress {
    match result {
        AIAgentActionResultType::RequestCommandOutput(
            RequestCommandOutputResult::LongRunningCommandSnapshot { block_id, .. },
        ) => ShellProgress::Snapshot {
            block_id: block_id.clone(),
        },
        AIAgentActionResultType::ReadShellCommandOutput(
            ReadShellCommandOutputResult::CommandFinished {
                block_id,
                output,
                exit_code,
                ..
            },
        )
        | AIAgentActionResultType::WriteToLongRunningShellCommand(
            WriteToLongRunningShellCommandResult::CommandFinished {
                block_id,
                output,
                exit_code,
            },
        )
        | AIAgentActionResultType::TransferShellCommandControlToUser(
            TransferShellCommandControlToUserResult::CommandFinished {
                block_id,
                output,
                exit_code,
            },
        ) => ShellProgress::Finished(FinishedCommand {
            block_id: block_id.clone(),
            output: output.clone(),
            exit_code: *exit_code,
        }),
        AIAgentActionResultType::RequestCommandOutput(
            RequestCommandOutputResult::Completed { .. }
            | RequestCommandOutputResult::CancelledBeforeExecution
            | RequestCommandOutputResult::Denylisted { .. }
            | RequestCommandOutputResult::TerminalBusy { .. },
        )
        | AIAgentActionResultType::ReadShellCommandOutput(
            ReadShellCommandOutputResult::LongRunningCommandSnapshot { .. }
            | ReadShellCommandOutputResult::Cancelled
            | ReadShellCommandOutputResult::Error(_),
        )
        | AIAgentActionResultType::WriteToLongRunningShellCommand(
            WriteToLongRunningShellCommandResult::Snapshot { .. }
            | WriteToLongRunningShellCommandResult::Cancelled
            | WriteToLongRunningShellCommandResult::Error(_),
        )
        | AIAgentActionResultType::TransferShellCommandControlToUser(
            TransferShellCommandControlToUserResult::Snapshot { .. }
            | TransferShellCommandControlToUserResult::Cancelled
            | TransferShellCommandControlToUserResult::Error(_),
        )
        | AIAgentActionResultType::RequestFileEdits(_)
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
        | AIAgentActionResultType::DeniedByHook { .. } => ShellProgress::None,
    }
}

/// Whether the tool ran and this is what it returned. A failing exit code, an error the tool
/// reported, or a shell that ended under the command are all answers from a tool that ran. A
/// cancelled result, a result Warp produced because the tool was not allowed to start, and a
/// denial by an agent hook are not.
pub(crate) fn ran_and_returned(result: &AIAgentActionResultType) -> bool {
    if result.is_cancelled() && !result.is_cancelled_during_requested_command_execution() {
        return false;
    }
    match result {
        AIAgentActionResultType::RequestCommandOutput(result) => match result {
            RequestCommandOutputResult::Completed { .. }
            | RequestCommandOutputResult::LongRunningCommandSnapshot { .. } => true,
            RequestCommandOutputResult::CancelledBeforeExecution
            | RequestCommandOutputResult::Denylisted { .. }
            | RequestCommandOutputResult::TerminalBusy { .. } => false,
        },
        AIAgentActionResultType::DeniedByHook { .. } => false,
        AIAgentActionResultType::ReadShellCommandOutput(_)
        | AIAgentActionResultType::WriteToLongRunningShellCommand(_)
        | AIAgentActionResultType::TransferShellCommandControlToUser(_)
        | AIAgentActionResultType::RequestFileEdits(_)
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
        | AIAgentActionResultType::AskUserQuestion(_) => true,
    }
}

#[cfg(test)]
fn record_observation(action: &AIAgentAction, envelope: &HookEnvelope, ctx: &mut AppContext) {
    let observed = ObservedTerminalResult {
        action_id: action.id.clone(),
        boundary: envelope.boundary(),
    };
    HookRuntime::handle(ctx).update(ctx, |runtime, _| {
        runtime.observed_terminal_results.push(observed);
    });
}

#[cfg(test)]
pub(crate) fn observed_terminal_results(ctx: &AppContext) -> Vec<ObservedTerminalResult> {
    HookRuntime::as_ref(ctx).observed_terminal_results.clone()
}

#[cfg(test)]
#[path = "tool_result_tests.rs"]
mod tests;
