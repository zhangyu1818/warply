//! The tool name a Cursor-format hook matches on and sees as `tool_name`. Cursor's own names are
//! used where a Warp action maps onto one of Cursor's tools; Warp-only tools get Warp-specific
//! names, which a `"*"` matcher still matches.

use std::borrow::Cow;

use crate::agent::action::{AIAgentActionType, FileEdit};

pub(super) fn tool_name(action: &AIAgentActionType) -> Cow<'static, str> {
    let name = match action {
        AIAgentActionType::RequestCommandOutput { .. } => "Shell",
        AIAgentActionType::ReadFiles(_) => "Read",
        AIAgentActionType::RequestFileEdits { file_edits, .. } => {
            let all_deletes = !file_edits.is_empty()
                && file_edits
                    .iter()
                    .all(|edit| matches!(edit, FileEdit::Delete { .. }));
            if all_deletes { "Delete" } else { "Write" }
        }
        AIAgentActionType::Grep { .. } => "Grep",
        AIAgentActionType::FileGlobV2 { .. } => "Glob",
        AIAgentActionType::SearchCodebase(_) => "SearchCodebase",
        AIAgentActionType::WriteToLongRunningShellCommand { .. } => "ShellWrite",
        AIAgentActionType::ReadShellCommandOutput { .. } => "ShellRead",
        AIAgentActionType::TransferShellCommandControlToUser { .. } => "ShellHandoff",
        AIAgentActionType::UseComputer(_) => "UseComputer",
        AIAgentActionType::RequestComputerUse(_) => "RequestComputerUse",
        AIAgentActionType::ReadDocuments(_) => "ReadDocuments",
        AIAgentActionType::EditDocuments(_) => "EditDocuments",
        AIAgentActionType::CreateDocuments(_) => "CreateDocuments",
        AIAgentActionType::AskUserQuestion { .. } => "AskUserQuestion",
        AIAgentActionType::InitProject => "InitProject",
        AIAgentActionType::OpenCodeReview => "OpenCodeReview",
        AIAgentActionType::InsertCodeReviewComments { .. } => "InsertCodeReviewComments",
    };
    Cow::Borrowed(name)
}

#[cfg(test)]
#[path = "tool_names_tests.rs"]
mod tests;
