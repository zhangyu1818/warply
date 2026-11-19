//! The `tool_input` object for the tool events: the action's arguments as the model gave them, one
//! hand-written JSON view per `AIAgentActionType`. Where Warp's action maps onto one of Cursor's
//! tools, the view uses Cursor's field names (`file_path`, `old_string`, `new_string`) so a
//! script written for Cursor reads it unchanged.

use serde_json::{Map, Value, json};

use super::{insert_some, range_view, serialized};
use crate::agent::action::{
    AIAgentActionType, AIAgentPtyWriteMode, CommentSide, FileEdit, InsertReviewComment,
    ShellCommandDelay,
};
use crate::diff_validation::ParsedDiff;
use crate::hooks::model::HookEnvelope;

/// The `tool_input` object for `preToolUse` and the post-tool events: the action's arguments as
/// the model gave them. Shell commands carry the working directory beside the command, as
/// Cursor's example does.
pub(super) fn tool_input(action: &AIAgentActionType, envelope: &HookEnvelope) -> Value {
    match action {
        AIAgentActionType::RequestCommandOutput {
            command,
            is_read_only,
            is_risky,
            wait_until_completion,
            uses_pager,
            rationale,
            citations: _,
        } => {
            let mut fields = Map::new();
            fields.insert("command".to_owned(), json!(command));
            if let Some(cwd) = &envelope.cwd {
                fields.insert("cwd".to_owned(), json!(cwd.to_string_lossy()));
            }
            insert_some(&mut fields, "is_read_only", *is_read_only);
            insert_some(&mut fields, "is_risky", *is_risky);
            fields.insert(
                "wait_until_completion".to_owned(),
                json!(wait_until_completion),
            );
            insert_some(&mut fields, "uses_pager", *uses_pager);
            insert_some(&mut fields, "rationale", rationale.as_deref());
            Value::Object(fields)
        }
        AIAgentActionType::WriteToLongRunningShellCommand {
            block_id,
            input,
            mode,
        } => json!({
            "block_id": serialized("block_id", block_id),
            "input": String::from_utf8_lossy(input),
            "mode": match mode {
                AIAgentPtyWriteMode::Raw => "raw",
                AIAgentPtyWriteMode::Line => "line",
                AIAgentPtyWriteMode::Block => "block",
            },
        }),
        AIAgentActionType::ReadFiles(request) => json!({
            "files": request
                .locations
                .iter()
                .map(|location| json!({
                    "path": location.name,
                    "lines": location.lines.iter().map(range_view).collect::<Vec<_>>(),
                }))
                .collect::<Vec<_>>(),
        }),
        AIAgentActionType::SearchCodebase(request) => {
            let mut fields = Map::new();
            fields.insert("query".to_owned(), json!(request.query));
            insert_some(
                &mut fields,
                "partial_paths",
                request.partial_paths.as_deref(),
            );
            insert_some(
                &mut fields,
                "codebase_path",
                request.codebase_path.as_deref(),
            );
            Value::Object(fields)
        }
        AIAgentActionType::RequestFileEdits { file_edits, title } => {
            let mut fields = Map::new();
            insert_some(&mut fields, "file_path", single_file(file_edits));
            insert_some(&mut fields, "title", title.as_deref());
            fields.insert(
                "edits".to_owned(),
                Value::Array(file_edits.iter().map(file_edit_view).collect()),
            );
            Value::Object(fields)
        }
        AIAgentActionType::Grep { queries, path } => json!({ "queries": queries, "path": path }),
        AIAgentActionType::FileGlobV2 {
            patterns,
            search_dir,
        } => {
            let mut fields = Map::new();
            fields.insert("patterns".to_owned(), json!(patterns));
            insert_some(&mut fields, "search_dir", search_dir.as_deref());
            Value::Object(fields)
        }
        AIAgentActionType::InitProject | AIAgentActionType::OpenCodeReview => json!({}),
        AIAgentActionType::ReadDocuments(request) => json!({
            "document_ids": request
                .document_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>(),
        }),
        AIAgentActionType::EditDocuments(request) => json!({
            "diffs": request
                .diffs
                .iter()
                .map(|diff| json!({
                    "document_id": diff.document_id.to_string(),
                    "search": diff.search,
                    "replace": diff.replace,
                }))
                .collect::<Vec<_>>(),
        }),
        AIAgentActionType::CreateDocuments(request) => json!({
            "documents": request
                .documents
                .iter()
                .map(|document| json!({ "title": document.title, "content": document.content }))
                .collect::<Vec<_>>(),
        }),
        AIAgentActionType::ReadShellCommandOutput { block_id, delay } => {
            let mut fields = Map::new();
            fields.insert("block_id".to_owned(), serialized("block_id", block_id));
            let delay = delay.as_ref().map(|delay| match delay {
                ShellCommandDelay::Duration(duration) => {
                    json!({ "seconds": duration.as_secs_f64() })
                }
                ShellCommandDelay::OnCompletion => json!("on_completion"),
            });
            insert_some(&mut fields, "delay", delay);
            Value::Object(fields)
        }
        AIAgentActionType::UseComputer(request) => {
            let mut fields = Map::new();
            fields.insert("action_summary".to_owned(), json!(request.action_summary));
            fields.insert(
                "actions".to_owned(),
                serialized("actions", &request.actions),
            );
            insert_some(
                &mut fields,
                "screenshot_params",
                request.screenshot_params.as_ref(),
            );
            Value::Object(fields)
        }
        AIAgentActionType::InsertCodeReviewComments {
            repo_path,
            comments,
            base_branch,
        } => {
            let mut fields = Map::new();
            fields.insert("repo_path".to_owned(), json!(repo_path.to_string_lossy()));
            insert_some(&mut fields, "base_branch", base_branch.as_deref());
            fields.insert(
                "comments".to_owned(),
                Value::Array(comments.iter().map(review_comment_view).collect()),
            );
            Value::Object(fields)
        }
        AIAgentActionType::RequestComputerUse(request) => {
            let mut fields = Map::new();
            fields.insert("task_summary".to_owned(), json!(request.task_summary));
            insert_some(
                &mut fields,
                "screenshot_params",
                request.screenshot_params.as_ref(),
            );
            Value::Object(fields)
        }
        AIAgentActionType::TransferShellCommandControlToUser { reason } => {
            json!({ "reason": reason })
        }
        AIAgentActionType::AskUserQuestion { questions } => {
            json!({ "questions": serialized("questions", questions) })
        }
    }
}

/// The one file a batch of edits touches, when every edit names the same file. Cursor's `Write`
/// and `Delete` edit one file, so a script reads the path from `file_path`.
fn single_file(file_edits: &[FileEdit]) -> Option<&str> {
    let mut files = file_edits.iter().map(FileEdit::file);
    let first = files.next()??;
    files.all(|file| file == Some(first)).then_some(first)
}

fn file_edit_view(edit: &FileEdit) -> Value {
    match edit {
        FileEdit::Edit(ParsedDiff::StrReplaceEdit {
            file,
            search,
            replace,
        }) => json!({
            "kind": "edit",
            "file_path": file,
            "old_string": search,
            "new_string": replace,
        }),
        FileEdit::Edit(ParsedDiff::V4AEdit {
            file,
            move_to,
            hunks,
        }) => json!({
            "kind": "edit",
            "file_path": file,
            "move_to": move_to,
            "hunks": serialized("hunks", hunks),
        }),
        FileEdit::Create { file, content } => json!({
            "kind": "create",
            "file_path": file,
            "content": content,
        }),
        FileEdit::Delete { file } => json!({ "kind": "delete", "file_path": file }),
    }
}

fn review_comment_view(comment: &InsertReviewComment) -> Value {
    let mut fields = Map::new();
    fields.insert("comment_id".to_owned(), json!(comment.comment_id));
    fields.insert("author".to_owned(), json!(comment.author));
    fields.insert(
        "last_modified_timestamp".to_owned(),
        json!(comment.last_modified_timestamp),
    );
    fields.insert("comment_body".to_owned(), json!(comment.comment_body));
    insert_some(
        &mut fields,
        "parent_comment_id",
        comment.parent_comment_id.as_deref(),
    );
    insert_some(&mut fields, "html_url", comment.html_url.as_deref());
    if let Some(location) = &comment.comment_location {
        let mut location_fields = Map::new();
        location_fields.insert(
            "relative_file_path".to_owned(),
            json!(location.relative_file_path),
        );
        if let Some(line) = &location.line {
            let side = line.side.map(|side| match side {
                CommentSide::Right => "right",
                CommentSide::Left => "left",
            });
            let mut line_fields = Map::new();
            line_fields.insert(
                "comment_line_range".to_owned(),
                range_view(&line.comment_line_range),
            );
            line_fields.insert(
                "diff_hunk_line_range".to_owned(),
                range_view(&line.diff_hunk_line_range),
            );
            line_fields.insert("diff_hunk_text".to_owned(), json!(line.diff_hunk_text));
            insert_some(&mut line_fields, "side", side);
            location_fields.insert("line".to_owned(), Value::Object(line_fields));
        }
        fields.insert("location".to_owned(), Value::Object(location_fields));
    }
    Value::Object(fields)
}
