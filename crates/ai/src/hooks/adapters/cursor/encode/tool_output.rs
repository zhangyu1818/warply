//! A `kind`-tagged JSON view of every `AIAgentActionResultType`, carried as a string in
//! `tool_output`.

use serde_json::{Map, Value, json};
use warp_terminal::model::BlockId;

use super::{insert_some, range_view, serialized};
use crate::agent::action_result::{
    AIAgentActionResultType, AnyFileContent, AskUserQuestionAnswerItem, AskUserQuestionResult,
    CreateDocumentsResult, DocumentContext, EditDocumentsResult, FileContext, FileGlobV2Result,
    GrepResult, InsertReviewCommentsResult, LrcActivity, ReadDocumentsResult, ReadFilesResult,
    ReadShellCommandOutputResult, RequestCommandOutputResult, RequestComputerUseResult,
    RequestFileEditsResult, SearchCodebaseResult, TransferShellCommandControlToUserResult,
    UseComputerResult, WriteToLongRunningShellCommandResult,
};
use crate::hooks::adapters::cursor::parts::{ToolPart, names_requested_file};

/// The result as one part of the action sees it: for one file of a split read or edit batch,
/// the entries of the result that are about that file; otherwise the whole result. A result that
/// is not the per-file kind (an error, a cancellation) is the same for every part.
pub(super) fn part_result_view(part: &ToolPart<'_>, result: &AIAgentActionResultType) -> Value {
    match (part, result) {
        (
            ToolPart::ReadFile(location),
            AIAgentActionResultType::ReadFiles(ReadFilesResult::Success { files }),
        ) => {
            let files = files
                .iter()
                .filter(|file| names_requested_file(&file.file_name, &location.name))
                .map(file_context_view)
                .collect::<Vec<_>>();
            json!({
                "kind": "success",
                "files": files,
            })
        }
        (
            ToolPart::EditFile {
                file: Some(file), ..
            },
            AIAgentActionResultType::RequestFileEdits(RequestFileEditsResult::Success {
                updated_files,
                deleted_files,
                ..
            }),
        ) => {
            let updated_files = updated_files
                .iter()
                .filter(|updated| names_requested_file(&updated.file_context.file_name, file))
                .map(|updated| {
                    json!({
                        "was_edited_by_user": updated.was_edited_by_user,
                        "file": file_context_view(&updated.file_context),
                    })
                })
                .collect::<Vec<_>>();
            let deleted_files = deleted_files
                .iter()
                .filter(|deleted| names_requested_file(deleted, file))
                .collect::<Vec<_>>();
            json!({
                "kind": "success",
                "updated_files": updated_files,
                "deleted_files": deleted_files,
            })
        }
        (ToolPart::Whole(_) | ToolPart::ReadFile(_) | ToolPart::EditFile { .. }, result) => {
            result_view(result)
        }
    }
}

/// A JSON object describing a result, tagged with the variant it came from. Binary payloads are
/// summarized by size; everything else is carried as the result holds it.
pub(super) fn result_view(result: &AIAgentActionResultType) -> Value {
    match result {
        AIAgentActionResultType::RequestCommandOutput(result) => match result {
            RequestCommandOutputResult::Completed {
                block_id,
                command,
                output,
                exit_code,
            } => {
                // The fork's command results carry no timestamps.
                let mut fields = Map::new();
                fields.insert("kind".to_owned(), json!("completed"));
                fields.insert("command".to_owned(), json!(command));
                fields.insert("output".to_owned(), json!(output));
                fields.insert("exit_code".to_owned(), json!(exit_code.value()));
                fields.insert("block_id".to_owned(), serialized("block_id", block_id));
                Value::Object(fields)
            }
            RequestCommandOutputResult::LongRunningCommandSnapshot {
                block_id,
                command,
                grid_contents,
                cursor,
                is_alt_screen_active,
                activity,
            } => json!({
                "kind": "long_running_snapshot",
                "command": command,
                "grid_contents": grid_contents,
                "cursor": cursor,
                "is_alt_screen_active": is_alt_screen_active,
                "block_id": serialized("block_id", block_id),
                "activity": serialized("activity", activity),
            }),
            RequestCommandOutputResult::CancelledBeforeExecution => {
                json!({ "kind": "cancelled_before_execution" })
            }
            RequestCommandOutputResult::Denylisted { command } => {
                json!({ "kind": "denylisted", "command": command })
            }
            RequestCommandOutputResult::TerminalBusy { command, block_id } => json!({
                "kind": "terminal_busy",
                "command": command,
                "block_id": serialized("block_id", block_id),
            }),
        },
        AIAgentActionResultType::WriteToLongRunningShellCommand(result) => match result {
            WriteToLongRunningShellCommandResult::Snapshot {
                block_id,
                grid_contents,
                cursor,
                is_alt_screen_active,
                is_preempted,
                activity,
            } => pty_snapshot_view(
                block_id,
                grid_contents,
                cursor,
                *is_alt_screen_active,
                *is_preempted,
                activity,
            ),
            WriteToLongRunningShellCommandResult::CommandFinished {
                block_id,
                output,
                exit_code,
            } => finished_command_view(block_id, None, output, exit_code.value()),
            WriteToLongRunningShellCommandResult::Cancelled => json!({ "kind": "cancelled" }),
            WriteToLongRunningShellCommandResult::Error(error) => {
                json!({ "kind": "error", "error": serialized("error", error) })
            }
        },
        AIAgentActionResultType::RequestFileEdits(result) => match result {
            RequestFileEditsResult::Success {
                diff,
                updated_files,
                deleted_files,
                lines_added,
                lines_removed,
            } => json!({
                "kind": "success",
                "diff": diff,
                "updated_files": updated_files
                    .iter()
                    .map(|updated| json!({
                        "was_edited_by_user": updated.was_edited_by_user,
                        "file": file_context_view(&updated.file_context),
                    }))
                    .collect::<Vec<_>>(),
                "deleted_files": deleted_files,
                "lines_added": lines_added,
                "lines_removed": lines_removed,
            }),
            RequestFileEditsResult::Cancelled => json!({ "kind": "cancelled" }),
            RequestFileEditsResult::DiffApplicationFailed { error } => {
                json!({ "kind": "diff_application_failed", "error": error })
            }
        },
        AIAgentActionResultType::ReadFiles(result) => match result {
            ReadFilesResult::Success { files } => json!({
                "kind": "success",
                "files": file_context_views(files),
            }),
            ReadFilesResult::Error(error) => error_view(error),
            ReadFilesResult::Cancelled => json!({ "kind": "cancelled" }),
        },
        AIAgentActionResultType::SearchCodebase(result) => match result {
            SearchCodebaseResult::Success { files } => {
                json!({ "kind": "success", "files": file_context_views(files) })
            }
            SearchCodebaseResult::Failed { reason, message } => json!({
                "kind": "failed",
                "reason": format!("{reason:?}"),
                "message": message,
            }),
            SearchCodebaseResult::Cancelled => json!({ "kind": "cancelled" }),
        },
        AIAgentActionResultType::Grep(result) => match result {
            GrepResult::Success { matched_files } => {
                json!({ "kind": "success", "matched_files": serialized("matched_files", matched_files) })
            }
            GrepResult::Error(error) => error_view(error),
            GrepResult::Cancelled => json!({ "kind": "cancelled" }),
        },
        AIAgentActionResultType::FileGlobV2(result) => match result {
            FileGlobV2Result::Success {
                matched_files,
                warnings,
            } => json!({
                "kind": "success",
                "matched_files": serialized("matched_files", matched_files),
                "warnings": warnings,
            }),
            FileGlobV2Result::Error(error) => error_view(error),
            FileGlobV2Result::Cancelled => json!({ "kind": "cancelled" }),
        },
        AIAgentActionResultType::OpenCodeReview | AIAgentActionResultType::InitProject => {
            json!({ "kind": "completed" })
        }
        AIAgentActionResultType::ReadDocuments(result) => match result {
            ReadDocumentsResult::Success { documents } => {
                json!({ "kind": "success", "documents": document_views(documents) })
            }
            ReadDocumentsResult::Error(error) => error_view(error),
            ReadDocumentsResult::Cancelled => json!({ "kind": "cancelled" }),
        },
        AIAgentActionResultType::EditDocuments(result) => match result {
            EditDocumentsResult::Success { updated_documents } => json!({
                "kind": "success",
                "updated_documents": document_views(updated_documents),
            }),
            EditDocumentsResult::Error(error) => error_view(error),
            EditDocumentsResult::Cancelled => json!({ "kind": "cancelled" }),
        },
        AIAgentActionResultType::CreateDocuments(result) => match result {
            CreateDocumentsResult::Success { created_documents } => json!({
                "kind": "success",
                "created_documents": document_views(created_documents),
            }),
            CreateDocumentsResult::Error(error) => error_view(error),
            CreateDocumentsResult::Cancelled => json!({ "kind": "cancelled" }),
        },
        AIAgentActionResultType::ReadShellCommandOutput(result) => match result {
            ReadShellCommandOutputResult::CommandFinished {
                command,
                block_id,
                output,
                exit_code,
            } => finished_command_view(block_id, Some(command), output, exit_code.value()),
            ReadShellCommandOutputResult::LongRunningCommandSnapshot {
                command,
                block_id,
                grid_contents,
                cursor,
                is_alt_screen_active,
                is_preempted,
                activity,
            } => json!({
                "kind": "long_running_snapshot",
                "command": command,
                "block_id": serialized("block_id", block_id),
                "grid_contents": grid_contents,
                "cursor": cursor,
                "is_alt_screen_active": is_alt_screen_active,
                "is_preempted": is_preempted,
                "activity": serialized("activity", activity),
            }),
            ReadShellCommandOutputResult::Cancelled => json!({ "kind": "cancelled" }),
            ReadShellCommandOutputResult::Error(error) => {
                json!({ "kind": "error", "error": serialized("error", error) })
            }
        },
        AIAgentActionResultType::UseComputer(result) => match result {
            UseComputerResult::Success(action_result) => {
                // The fork's trimmed computer_use reports a screenshot and cursor position only.
                let mut fields = Map::new();
                fields.insert("kind".to_owned(), json!("success"));
                insert_some(
                    &mut fields,
                    "screenshot",
                    action_result.screenshot.as_ref().map(screenshot_view),
                );
                insert_some(
                    &mut fields,
                    "cursor_position",
                    action_result
                        .cursor_position
                        .map(|position| json!([position.x(), position.y()])),
                );
                Value::Object(fields)
            }
            UseComputerResult::Error(error) => error_view(error),
            UseComputerResult::Cancelled => json!({ "kind": "cancelled" }),
        },
        AIAgentActionResultType::InsertReviewComments(result) => match result {
            InsertReviewCommentsResult::Success { repo_path } => {
                json!({ "kind": "success", "repo_path": repo_path })
            }
            InsertReviewCommentsResult::Error { repo_path, message } => {
                json!({ "kind": "error", "repo_path": repo_path, "error": message })
            }
            InsertReviewCommentsResult::Cancelled => json!({ "kind": "cancelled" }),
        },
        AIAgentActionResultType::RequestComputerUse(result) => match result {
            RequestComputerUseResult::Approved {
                screenshot,
                platform,
            } => json!({
                "kind": "approved",
                "screenshot": screenshot_view(screenshot),
                "platform": format!("{platform:?}"),
            }),
            RequestComputerUseResult::Error(error) => error_view(error),
            RequestComputerUseResult::Cancelled => json!({ "kind": "cancelled" }),
        },
        AIAgentActionResultType::TransferShellCommandControlToUser(result) => match result {
            TransferShellCommandControlToUserResult::Snapshot {
                block_id,
                grid_contents,
                cursor,
                is_alt_screen_active,
                is_preempted,
                activity,
            } => pty_snapshot_view(
                block_id,
                grid_contents,
                cursor,
                *is_alt_screen_active,
                *is_preempted,
                activity,
            ),
            TransferShellCommandControlToUserResult::CommandFinished {
                block_id,
                output,
                exit_code,
            } => finished_command_view(block_id, None, output, exit_code.value()),
            TransferShellCommandControlToUserResult::Cancelled => json!({ "kind": "cancelled" }),
            TransferShellCommandControlToUserResult::Error(error) => {
                json!({ "kind": "error", "error": serialized("error", error) })
            }
        },
        AIAgentActionResultType::AskUserQuestion(result) => match result {
            AskUserQuestionResult::Success { answers } => json!({
                "kind": "success",
                "answers": answers
                    .iter()
                    .map(|answer| match answer {
                        AskUserQuestionAnswerItem::Answered {
                            question_id,
                            selected_options,
                            other_text,
                        } => json!({
                            "question_id": question_id,
                            "selected_options": selected_options,
                            "other_text": other_text,
                        }),
                        AskUserQuestionAnswerItem::Skipped { question_id } => {
                            json!({ "question_id": question_id, "skipped": true })
                        }
                    })
                    .collect::<Vec<_>>(),
            }),
            AskUserQuestionResult::Error(error) => error_view(error),
            AskUserQuestionResult::Cancelled => json!({ "kind": "cancelled" }),
            AskUserQuestionResult::SkippedByAutoApprove { question_ids } => json!({
                "kind": "skipped_by_auto_approve",
                "question_ids": question_ids,
            }),
        },
        AIAgentActionResultType::DeniedByHook { reason } => {
            json!({ "kind": "denied_by_hook", "reason": reason })
        }
    }
}

fn error_view(error: &str) -> Value {
    json!({ "kind": "error", "error": error })
}

fn pty_snapshot_view(
    block_id: &BlockId,
    grid_contents: &str,
    cursor: &str,
    is_alt_screen_active: bool,
    is_preempted: bool,
    activity: &Option<LrcActivity>,
) -> Value {
    json!({
        "kind": "snapshot",
        "block_id": serialized("block_id", block_id),
        "grid_contents": grid_contents,
        "cursor": cursor,
        "is_alt_screen_active": is_alt_screen_active,
        "is_preempted": is_preempted,
        "activity": serialized("activity", activity),
    })
}

fn finished_command_view(
    block_id: &BlockId,
    command: Option<&str>,
    output: &str,
    exit_code: i32,
) -> Value {
    // The fork's command results carry no timestamps, so the finished view reports none.
    let mut fields = Map::new();
    fields.insert("kind".to_owned(), json!("command_finished"));
    insert_some(&mut fields, "command", command);
    fields.insert("block_id".to_owned(), serialized("block_id", block_id));
    fields.insert("output".to_owned(), json!(output));
    fields.insert("exit_code".to_owned(), json!(exit_code));
    Value::Object(fields)
}

fn file_context_view(context: &FileContext) -> Value {
    let mut fields = Map::new();
    fields.insert("file_name".to_owned(), json!(context.file_name));
    match &context.content {
        AnyFileContent::StringContent(content) => {
            fields.insert("content".to_owned(), json!(content));
        }
        AnyFileContent::BinaryContent(bytes) => {
            fields.insert("is_binary".to_owned(), json!(true));
            fields.insert("size_bytes".to_owned(), json!(bytes.len()));
        }
    }
    insert_some(
        &mut fields,
        "line_range",
        context.line_range.as_ref().map(range_view),
    );
    fields.insert("line_count".to_owned(), json!(context.line_count));
    Value::Object(fields)
}

fn file_context_views(contexts: &[FileContext]) -> Value {
    Value::Array(contexts.iter().map(file_context_view).collect())
}

fn document_view(document: &DocumentContext) -> Value {
    json!({
        "document_id": document.document_id.to_string(),
        "version": document.document_version.0,
        "content": document.content,
        "line_ranges": document.line_ranges.iter().map(range_view).collect::<Vec<_>>(),
    })
}

fn document_views(documents: &[DocumentContext]) -> Value {
    Value::Array(documents.iter().map(document_view).collect())
}

fn screenshot_view(screenshot: &computer_use::Screenshot) -> Value {
    json!({
        "width": screenshot.width,
        "height": screenshot.height,
        "original_width": screenshot.original_width,
        "original_height": screenshot.original_height,
        "mime_type": screenshot.mime_type,
        "size_bytes": screenshot.data.len(),
    })
}
