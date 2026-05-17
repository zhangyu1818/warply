pub(crate) mod conversation;
pub(crate) mod todos;

pub(crate) mod comment;
pub(crate) mod icons;
pub(crate) mod identifiers;
pub(crate) mod redaction;
pub(crate) mod task;
mod task_store;
pub(crate) mod util;

// Re-export types that were moved to the ai crate.
pub use ai::agent::{action::*, action_result::*, AIAgentCitation, FileLocations};

use crate::ai::block_context::BlockContext;
use crate::ai::blocklist::block::view_impl::output::are_all_text_sections_empty;
use crate::code::editor_management::CodeSource;
use crate::code_review::comments::{
    AttachedReviewComment as CodeReviewComment, ReviewCommentBatch,
};
use crate::search::slash_command_menu::static_commands::commands;
use chrono::{DateTime, Local, TimeDelta};
use comment::ReviewComment;
pub use identifiers::AIIdentifiers;
use task::TaskId;

use warp_editor::render::model::LineCount;

use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::ops::{Deref, DerefMut, Range};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::ai::acp::{AcpPermissionRequest, AcpPlan, AcpToolCall};
use crate::ai::execution_context::AiExecutionContext;
use crate::terminal::model::block::BlockId;
use crate::terminal::shell::ShellType;
use derivative::Derivative;
use markdown_parser::{parse_markdown, FormattedTable, FormattedText, FormattedTextInline};
use serde::{Deserialize, Serialize};

use super::llms::LLMId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CancellationReason {
    /// The user explicitly cancelled without providing a follow-up.
    ManuallyCancelled,

    /// The user submitted a follow-up query during streaming which implicitly cancelled the current one.
    FollowUpSubmitted {
        is_for_same_conversation: bool,
    },

    /// The user executed a shell command in the middle of the response stream.
    UserCommandExecuted,

    /// The user reverted the conversation to a previous state, deleting exchanges.
    Reverted,

    // The user deleted the conversation while it was in progress.
    Deleted,

    /// The long-running command completed while the agent was still streaming.
    /// This should be treated as a successful completion, not a cancellation.
    OptimisticCLISubagentCompletion,
}

impl Display for CancellationReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CancellationReason::ManuallyCancelled => write!(f, "manual cancellation"),
            CancellationReason::FollowUpSubmitted { .. } => write!(f, "follow-up submission"),
            CancellationReason::UserCommandExecuted => write!(f, "user command execution"),
            CancellationReason::Reverted => write!(f, "revert"),
            CancellationReason::Deleted => write!(f, "deleted"),
            CancellationReason::OptimisticCLISubagentCompletion => {
                write!(f, "LRC command completed")
            }
        }
    }
}

impl CancellationReason {
    pub fn is_follow_up_for_same_conversation(&self) -> bool {
        matches!(
            self,
            CancellationReason::FollowUpSubmitted {
                is_for_same_conversation: true
            }
        )
    }
}

impl CancellationReason {
    pub fn is_manually_cancelled(&self) -> bool {
        matches!(self, CancellationReason::ManuallyCancelled)
    }

    pub fn is_reverted(&self) -> bool {
        matches!(self, CancellationReason::Reverted)
    }

    pub fn is_lrc_command_completed(&self) -> bool {
        matches!(self, CancellationReason::OptimisticCLISubagentCompletion)
    }
}

#[derive(Clone, Debug)]
pub enum FinishedAIAgentOutput {
    /// The user manually cancelled output streaming.
    Cancelled {
        // The output received up til the point of cancellation, if any.
        output: Option<Shared<AIAgentOutput>>,
        /// Why the stream was cancelled.
        reason: CancellationReason,
    },
    /// Output streaming failed.
    Error {
        // The output received up til the error was encountered, if any.
        output: Option<Shared<AIAgentOutput>>,
        error: RenderableAIError,
    },
    /// Output streaming completed successfully.
    Success { output: Shared<AIAgentOutput> },
}

impl Display for FinishedAIAgentOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinishedAIAgentOutput::Cancelled { .. } => write!(f, "Cancelled"),
            FinishedAIAgentOutput::Error { error, .. } => write!(f, "Error: {error}"),
            FinishedAIAgentOutput::Success { output } => write!(f, "\n{output}"),
        }
    }
}

impl FinishedAIAgentOutput {
    pub fn model_id(&self) -> Option<LLMId> {
        self.output()
            .and_then(|output| output.get().model_info.as_ref().map(|m| m.model_id.clone()))
    }

    pub fn output(&self) -> Option<&Shared<AIAgentOutput>> {
        match self {
            Self::Cancelled { output, .. } => output.as_ref(),
            Self::Error { .. } => None,
            Self::Success { output } => Some(output),
        }
    }
}

#[derive(Debug)]
pub struct Shared<T> {
    value: Arc<RwLock<T>>,
}

impl<T> Clone for Shared<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            value: Arc::new(RwLock::new(self.value.read().clone())),
        }
    }
}

impl<T: Clone + std::fmt::Debug> Shared<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Arc::new(RwLock::new(value)),
        }
    }

    pub fn get(&self) -> impl Deref<Target = T> + '_ {
        self.value.read()
    }

    /// Returns an owned `Shared` pointing to the same underlying `T`.
    ///
    /// While `Clone` performs a deep copy on the other value, this ultimately points to the same
    /// value `T`.
    pub fn get_owned(&self) -> Shared<T> {
        Self {
            value: self.value.clone(),
        }
    }

    fn get_mut(&self) -> impl DerefMut<Target = T> + '_ {
        self.value.write()
    }
}

impl<T: Display> Display for Shared<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.read().fmt(f)
    }
}

/// Status of output streaming from the AI API.
#[derive(Clone, Debug)]
pub enum AIAgentOutputStatus {
    Streaming {
        output: Option<Shared<AIAgentOutput>>,
    },
    Finished {
        finished_output: FinishedAIAgentOutput,
    },
}

impl Display for AIAgentOutputStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIAgentOutputStatus::Streaming { .. } => write!(f, "Streaming..."),
            AIAgentOutputStatus::Finished { finished_output } => write!(f, "{finished_output}"),
        }
    }
}

impl AIAgentOutputStatus {
    pub fn cancel_reason(&self) -> Option<&CancellationReason> {
        match self {
            Self::Finished {
                finished_output: FinishedAIAgentOutput::Cancelled { reason, .. },
            } => Some(reason),
            _ => None,
        }
    }

    pub fn model_id(&self) -> Option<LLMId> {
        self.output()
            .and_then(|output| output.get().model_info.as_ref().map(|m| m.model_id.clone()))
    }

    pub fn output(&self) -> Option<&Shared<AIAgentOutput>> {
        match self {
            Self::Streaming { output, .. } => output.as_ref(),
            Self::Finished {
                finished_output, ..
            } => finished_output.output(),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(
            self,
            Self::Finished {
                finished_output: FinishedAIAgentOutput::Cancelled { .. },
                ..
            }
        )
    }

    pub fn is_finished(&self) -> bool {
        match self {
            Self::Streaming { .. } => false,
            Self::Finished { .. } => true,
        }
    }

    pub fn is_finished_and_successful(&self) -> bool {
        matches!(
            self,
            Self::Finished {
                finished_output: FinishedAIAgentOutput::Success { .. }
            }
        )
    }

    pub fn is_streaming(&self) -> bool {
        matches!(self, Self::Streaming { .. })
    }
}

// This value is the cost of a single request.
// It is returned as part of the final response chunk from the agent.
/// The AI output received in response to a user prompt/query.
#[derive(Clone, Default, Derivative)]
#[derivative(Debug, PartialEq, Eq)]
pub struct AIAgentOutput {
    pub messages: Vec<AIAgentOutputMessage>,

    /// The set of documents that were referenced in the LLM's response.
    pub citations: Vec<AIAgentCitation>,

    /// Information about the model that generated this output.
    pub model_info: Option<OutputModelInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputModelInfo {
    pub model_id: LLMId,
    pub display_name: String,
}

impl Display for AIAgentOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, message) in self.messages.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "Message {}: {}", i + 1, message)?;
        }
        Ok(())
    }
}

impl AIAgentOutput {
    /// Returns only the text from agent output messages in the output.
    pub fn text_from_agent_output(&self) -> impl Iterator<Item = &AIAgentText> {
        self.messages
            .iter()
            .filter_map(|message| match &message.message {
                AIAgentOutputMessageType::Text(text) => Some(text),
                _ => None,
            })
    }

    /// Returns only the text from reasoning messages in the output.
    pub fn text_from_agent_reasoning(&self) -> impl Iterator<Item = &AIAgentText> {
        self.messages
            .iter()
            .filter_map(|message| match &message.message {
                AIAgentOutputMessageType::Reasoning { text, .. } => Some(text),
                _ => None,
            })
    }

    /// Returns all of the text contained in the output, including agent output, reasoning,
    /// and conversation summaries.
    ///
    /// IMPORTANT: This must stay in sync with the rendering code in `output.rs` — every
    /// message type whose sections increment `text_section_index` during rendering must
    /// also be yielded here, otherwise link detection indices will be offset.
    pub fn all_text(&self) -> impl Iterator<Item = &AIAgentText> {
        self.messages
            .iter()
            .filter_map(|message| match &message.message {
                AIAgentOutputMessageType::Text(text) => Some(text),
                AIAgentOutputMessageType::Reasoning { text, .. } => Some(text),
                AIAgentOutputMessageType::Summarization {
                    text,
                    summarization_type: SummarizationType::ConversationSummary,
                    ..
                } => Some(text),
                _ => None,
            })
            // It's important to filter these out, because we filter these out when rendering the output
            // and the text_section_index must match for detected links to work.
            .filter(|text| !are_all_text_sections_empty(&text.sections))
    }

    /// Returns all of the text contained in the output with their message IDs, including agent output,
    /// reasoning, and conversation summaries.
    ///
    /// IMPORTANT: This must stay in sync with the rendering code in `output.rs` — see [`all_text`].
    pub fn all_text_with_message_id(&self) -> impl Iterator<Item = (&MessageId, &AIAgentText)> {
        self.messages
            .iter()
            .filter_map(|message| match &message.message {
                AIAgentOutputMessageType::Text(text) => Some((&message.id, text)),
                AIAgentOutputMessageType::Reasoning { text, .. } => Some((&message.id, text)),
                AIAgentOutputMessageType::Summarization {
                    text,
                    summarization_type: SummarizationType::ConversationSummary,
                    ..
                } => Some((&message.id, text)),
                _ => None,
            })
            // It's important to filter these out, because we filter these out when rendering the output
            // and the text_section_index must match for detected links to work.
            .filter(|(_, text)| !are_all_text_sections_empty(&text.sections))
    }

    pub fn actions(&self) -> impl Iterator<Item = &AIAgentAction> {
        self.messages
            .iter()
            .filter_map(|message| match &message.message {
                AIAgentOutputMessageType::Action(action) => Some(action),
                _ => None,
            })
    }

    pub fn todo_operations(&self) -> impl Iterator<Item = &TodoOperation> {
        self.messages
            .iter()
            .filter_map(|message| match &message.message {
                AIAgentOutputMessageType::TodoOperation(operation) => Some(operation),
                _ => None,
            })
    }

    /// Format this output for copying to clipboard.
    /// This extracts all content (text, code, and action results) with proper formatting.
    pub fn format_for_copy(
        &self,
        action_model: Option<&crate::ai::blocklist::BlocklistAIActionModel>,
    ) -> String {
        let mut result = Vec::new();
        let mut last_was_action = false;

        // Process all messages in order, collecting all content
        for message in &self.messages {
            match &message.message {
                AIAgentOutputMessageType::Text(text) => {
                    // If the last message was an action and this is text, add some separation
                    if last_was_action {
                        result.push(String::new()); // Add blank line for readability
                    }

                    // Collect all text and code sections from this text message
                    for section in &text.sections {
                        match section {
                            AIAgentTextSection::PlainText { text } => {
                                result.push(text.text().to_string());
                            }
                            AIAgentTextSection::Code { .. }
                            | AIAgentTextSection::Table { .. }
                            | AIAgentTextSection::Image { .. }
                            | AIAgentTextSection::MermaidDiagram { .. } => {
                                result.push(format!("{}", MarkdownTextSection(section)));
                            }
                        }
                    }
                    last_was_action = false;
                }
                AIAgentOutputMessageType::Action(action) => {
                    // Include action results from the action model if available
                    if let Some(action_model) = action_model {
                        if let Some(action_result) = action_model.get_action_result(&action.id) {
                            result.push(format!("{}", MarkdownActionResult(&action_result.result)));
                            // Add an extra newline after tool call results for readability
                            result.push(String::new());
                            last_was_action = true;
                        }
                    }
                }
                AIAgentOutputMessageType::TodoOperation(operation) => {
                    result.push(format!("{operation}"));
                    last_was_action = false;
                }
                AIAgentOutputMessageType::Subagent(subagent) => {
                    result.push(format!("{subagent}"));
                    last_was_action = false;
                }
                AIAgentOutputMessageType::CommentsAddressed {
                    comments: comment_ids,
                } => {
                    result.push(format!("Addressed {} comments", comment_ids.len()));
                    last_was_action = false;
                }
                AIAgentOutputMessageType::Reasoning { .. } => continue,
                AIAgentOutputMessageType::Summarization { .. } => continue,
                AIAgentOutputMessageType::AcpToolCall(tool_call) => {
                    result.push(format!("ACP Tool Call: {}", tool_call.title));
                    last_was_action = false;
                }
                AIAgentOutputMessageType::AcpPlan(plan) => {
                    result.push(format!("ACP Plan: {} entries", plan.plan.entries.len()));
                    last_was_action = false;
                }
                AIAgentOutputMessageType::AcpPermission(request) => {
                    result.push(format!("ACP Permission Request: {}", request.request_id));
                    last_was_action = false;
                }
                AIAgentOutputMessageType::DebugOutput { text } => {
                    result.push(format!("[DEBUG] {text}"));
                    last_was_action = false;
                }
                AIAgentOutputMessageType::ArtifactCreated(_) => continue,
            }
        }

        // Remove trailing empty lines
        while result.last() == Some(&String::new()) {
            result.pop();
        }

        result.join("\n")
    }

    pub fn extend_citations(&mut self, citations: Vec<AIAgentCitation>) {
        let new_citations: Vec<_> = citations
            .into_iter()
            .filter(|c| !self.citations.contains(c))
            .collect();
        self.citations.extend(new_citations);
    }

    /// Calculate the action index for a given action_id by counting preceding actions in the output.
    /// Returns the 0-based index of the action, or None if the action is not found.
    pub fn calculate_action_index(&self, target_action_id: &AIAgentActionId) -> Option<usize> {
        let mut action_index = 0;
        for output_message in &self.messages {
            if let AIAgentOutputMessageType::Action(AIAgentAction { id, .. }) =
                &output_message.message
            {
                if id == target_action_id {
                    return Some(action_index);
                }
                action_index += 1;
            }
        }
        None // Fallback if action_id not found
    }
}

/// Represents user visible errors.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RenderableAIError {
    Other { error_message: String },
}

impl Display for RenderableAIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other { error_message } => write!(f, "{error_message}"),
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgrammingLanguage {
    Shell(ShellType),
    Other(String),
}

impl ProgrammingLanguage {
    pub fn display_name(&self) -> String {
        match self {
            Self::Shell(shell_type) => shell_type.name().to_owned(),
            Self::Other(language) => language.to_lowercase(),
        }
    }

    /// Returns the file extension for the given programming language.
    // TODO(INT-605): Refactor so we don't have to edit this function and the `languages` crate.
    pub fn to_extension(&self) -> Option<&str> {
        match self {
            // The arms below cover both canonical language names emitted by the agent (e.g.
            // "rust", "kotlin") and common markdown code-fence aliases (e.g. "rs", "kt") to keep
            // syntax highlighting working when the model uses either. The set of recognized
            // languages here is kept in sync with `SUPPORTED_LANGUAGES` in the `languages` crate.
            Self::Other(language) => match language.to_lowercase().as_str() {
                "rust" | "rs" => Some("rs"),
                "go" | "golang" => Some("go"),
                "python" | "py" => Some("py"),
                "javascript" | "js" => Some("js"),
                "typescript" | "ts" => Some("ts"),
                "jsx" => Some("jsx"),
                "tsx" => Some("tsx"),
                "yaml" | "yml" => Some("yaml"),
                "cpp" | "c++" => Some("cpp"),
                "java" => Some("java"),
                "groovy" => Some("java"),
                "shell" => Some("sh"),
                "c#" | "csharp" => Some("cs"),
                "html" => Some("html"),
                "css" => Some("css"),
                "c" => Some("c"),
                "json" => Some("json"),
                "jq" => Some("jq"),
                "hcl" | "terraform" | "tf" => Some("hcl"),
                "lua" => Some("lua"),
                "ruby" | "rb" => Some("rb"),
                "php" => Some("php"),
                "toml" => Some("toml"),
                "swift" => Some("swift"),
                "kotlin" | "kt" => Some("kt"),
                "powershell" => Some("ps1"),
                "elixir" => Some("exs"),
                "scala" => Some("scala"),
                "sql" => Some("sql"),
                "objective-c" | "objc" => Some("m"),
                "starlark" => Some("bzl"),
                "xml" => Some("xml"),
                "vue" => Some("vue"),
                "dockerfile" | "docker" | "containerfile" => Some("dockerfile"),
                _ => None,
            },
            Self::Shell(ShellType::PowerShell) => Some("ps1"),
            _ => None,
        }
    }

    /// Return whether this language is a shell language.
    /// This is used to determine whether to show the "execute in terminal" button.
    pub fn is_shell(&self) -> bool {
        matches!(self, Self::Shell(_))
    }
}

impl Display for ProgrammingLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgrammingLanguage::Shell(shell_type) => write!(f, "{}", shell_type.name()),
            ProgrammingLanguage::Other(language) => write!(f, "{}", language.to_lowercase()),
        }
    }
}

impl From<String> for ProgrammingLanguage {
    // Returns a programming language for a markdown language specifier
    fn from(value: String) -> Self {
        if let Some(shell_type) = ShellType::from_markdown_language_spec(value.as_str()) {
            ProgrammingLanguage::Shell(shell_type)
        } else {
            ProgrammingLanguage::Other(value)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AgentOutputImageLayout {
    Block,
    Inline,
}

/// A ID for an AI action generated as part of an [`AIAgentOutput`].
///
/// The internal ID itself should be opaque to all callers. This ID may be relayed back to the AI with
/// the `AIAgentActionResult` from the action.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AIAgentActionId(String);

impl From<String> for AIAgentActionId {
    fn from(value: String) -> Self {
        AIAgentActionId(value)
    }
}

impl From<AIAgentActionId> for String {
    fn from(value: AIAgentActionId) -> Self {
        value.0
    }
}

impl Display for AIAgentActionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<crate::persistence::model::AIAgentActionId> for AIAgentActionId {
    fn from(value: crate::persistence::model::AIAgentActionId) -> Self {
        Self(value.0)
    }
}

impl From<AIAgentActionId> for crate::persistence::model::AIAgentActionId {
    fn from(value: AIAgentActionId) -> Self {
        crate::persistence::model::AIAgentActionId(value.0)
    }
}

/// An "action" included in an AI output.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AIAgentAction {
    /// Unique ID for the action.
    pub id: AIAgentActionId,

    /// The ID of the task to which this action belongs.
    pub task_id: TaskId,

    /// The action itself.
    pub action: AIAgentActionType,

    /// `true` if this action requires a corresponding `AIAgentActionResult` to be sent back to the
    /// AI API.
    ///
    /// If this is `true`, a corresponding result _must_ be included in the next query to the AI.
    pub requires_result: bool,
}

impl Display for AIAgentAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.action)
    }
}

impl AIAgentAction {
    pub fn is_request_file_edit(&self) -> bool {
        matches!(self.action, AIAgentActionType::RequestFileEdits { .. })
    }

    pub fn is_agent_monitored_request_command_output(&self) -> bool {
        matches!(
            self.action,
            AIAgentActionType::RequestCommandOutput {
                wait_until_completion: false,
                ..
            }
        )
    }

    pub fn is_get_specific_files(&self) -> bool {
        self.action.is_read_files()
    }

    pub fn is_get_relevant_files(&self) -> bool {
        self.action.is_search_codebase()
    }

    pub fn is_grep(&self) -> bool {
        self.action.is_grep()
    }

    pub fn is_file_glob(&self) -> bool {
        self.action.is_file_glob()
    }

    pub fn executable_command(&self) -> Option<String> {
        match &self.action {
            AIAgentActionType::RequestCommandOutput { command, .. } => Some(command.clone()),
            _ => None,
        }
    }

    pub fn is_write_to_shell_command(&self) -> bool {
        self.action.is_write_to_shell_command()
    }

    pub fn matches_command(&self, command: &String) -> bool {
        Some(command) == self.executable_command().as_ref()
    }
}

pub struct MarkdownTextSection<'a>(pub &'a AIAgentTextSection);

impl<'a> std::fmt::Display for MarkdownTextSection<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            AIAgentTextSection::PlainText { text } => {
                write!(f, "{}", text.text())
            }
            AIAgentTextSection::Code {
                code,
                language,
                source,
            } => {
                write!(f, "```")?;
                if let Some(lang) = language {
                    write!(f, "{}", lang.display_name())?;
                }
                if let Some(CodeSource::Link {
                    path,
                    range_start,
                    range_end,
                }) = source
                {
                    write!(f, " path={path:?}")?;
                    if let (Some(range_start), Some(range_end)) = (range_start, range_end) {
                        write!(
                            f,
                            " start={} end={}",
                            range_start.line_num, range_end.line_num
                        )?;
                    }
                }
                writeln!(f)?;
                writeln!(f, "{code}")?;
                write!(f, "```")
            }
            AIAgentTextSection::Table { table } => {
                write!(f, "{}", table.markdown_source)
            }
            AIAgentTextSection::Image { image } => {
                write!(f, "{}", image.markdown_source)
            }
            AIAgentTextSection::MermaidDiagram { diagram } => {
                write!(f, "{}", diagram.markdown_source)
            }
        }
    }
}

pub struct MarkdownActionResult<'a>(pub &'a AIAgentActionResultType);

impl<'a> std::fmt::Display for MarkdownActionResult<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            AIAgentActionResultType::RequestCommandOutput(result) => match result {
                RequestCommandOutputResult::Completed {
                    command,
                    output,
                    exit_code: _,
                    ..
                } => {
                    write!(
                        f,
                        "\n**Command Executed:**\n```bash\n{command}\n```\n\n**Output:**\n```\n{output}\n```"
                    )
                }
                RequestCommandOutputResult::LongRunningCommandSnapshot {
                    command,
                    grid_contents,
                    ..
                } => {
                    write!(
                        f,
                        "\n```bash\n{command}\n```\n\n**Current Output:**\n```\n{grid_contents}\n```"
                    )
                }
                RequestCommandOutputResult::CancelledBeforeExecution => {
                    write!(f, "\n_Command cancelled_")
                }
                RequestCommandOutputResult::Denylisted { command } => {
                    write!(
                        f,
                        "\nCommand ({command}) was on denylist and so was not allowed to run"
                    )
                }
            },
            AIAgentActionResultType::WriteToLongRunningShellCommand(result) => match result {
                WriteToLongRunningShellCommandResult::CommandFinished { output, .. } => {
                    write!(f, "\n```\n{output}\n```")
                }
                WriteToLongRunningShellCommandResult::Snapshot { grid_contents, .. } => {
                    write!(f, "\n```\n{grid_contents}\n```")
                }
                WriteToLongRunningShellCommandResult::Cancelled => {
                    write!(f, "\n_Command cancelled_")
                }
                WriteToLongRunningShellCommandResult::Error(e) => {
                    write!(f, "\n_Write to command failed: {e:?}")
                }
            },
            AIAgentActionResultType::RequestFileEdits(result) => match result {
                RequestFileEditsResult::Success { diff, .. } => {
                    write!(f, "\n\n**Diff:**\n```diff\n{diff}\n```\n\n")
                }
                RequestFileEditsResult::Cancelled => write!(f, "\n_File edits cancelled_"),
                RequestFileEditsResult::DiffApplicationFailed { error } => {
                    write!(f, "\n_File edits failed: {error} _")
                }
            },
            AIAgentActionResultType::ReadFiles(result) => match result {
                ReadFilesResult::Success { files } => {
                    write!(f, "\n\n**Files Read:**\n\n")?;
                    for file in files {
                        writeln!(f, "**{}**", file.file_name)?;
                        let content = &file.content;
                        if let AnyFileContent::StringContent(text) = content {
                            if !text.trim().is_empty() {
                                writeln!(f, "```\n{text}\n```\n")?;
                            }
                        }
                    }
                    Ok(())
                }
                ReadFilesResult::Error(error) => write!(f, "\n_Read files error: {error} _"),
                ReadFilesResult::Cancelled => write!(f, "\n_Read files cancelled_"),
            },
            AIAgentActionResultType::SearchCodebase(result) => match result {
                SearchCodebaseResult::Success { files } => {
                    write!(f, "\n\n**Codebase Search Results:**\n\n")?;
                    for file in files {
                        writeln!(f, "- **{}**", file.file_name)?;
                        let content = &file.content;
                        if let AnyFileContent::StringContent(text) = content {
                            if !text.trim().is_empty() {
                                writeln!(f, "```\n{text}\n```\n")?;
                            }
                        }
                    }
                    Ok(())
                }
                SearchCodebaseResult::Failed { message, .. } => {
                    write!(f, "\n_Codebase search failed: {message} _")
                }
                SearchCodebaseResult::Cancelled => write!(f, "\n_Codebase search cancelled_"),
            },
            AIAgentActionResultType::FileGlobV2(result) => match result {
                FileGlobV2Result::Success { matched_files, .. } => {
                    write!(f, "\n\n**File Glob Results:**\n\n")?;
                    for file in matched_files {
                        writeln!(f, "- **{}**", file.file_path)?;
                    }
                    Ok(())
                }
                FileGlobV2Result::Error(message) => {
                    write!(f, "\n_File glob error: {message} _")
                }
                FileGlobV2Result::Cancelled => write!(f, "\n_File glob cancelled_"),
            },
            AIAgentActionResultType::Grep(result) => match result {
                GrepResult::Success { matched_files } => {
                    write!(f, "\n\n**Grep Results:**\n\n")?;
                    for file in matched_files {
                        writeln!(f, "- **{}**", file.file_path)?;
                    }
                    Ok(())
                }
                GrepResult::Error(message) => {
                    write!(f, "\n_Grep error: {message} _")
                }
                GrepResult::Cancelled => write!(f, "\n_Grep cancelled_"),
            },
            AIAgentActionResultType::ReadDocuments(result) => match result {
                ReadDocumentsResult::Success { documents } => {
                    write!(f, "\n\n**Documents Read:**\n\n")?;
                    for document in documents {
                        writeln!(f, "**Document {}**", document.document_id)?;
                        if !document.content.trim().is_empty() {
                            writeln!(f, "```\n{}\n```\n", document.content)?;
                        }
                    }
                    Ok(())
                }
                ReadDocumentsResult::Error(error) => {
                    write!(f, "\n_Read documents error: {error} _")
                }
                ReadDocumentsResult::Cancelled => write!(f, "\n_Read documents cancelled_"),
            },
            AIAgentActionResultType::EditDocuments(result) => match result {
                EditDocumentsResult::Success { updated_documents } => {
                    write!(f, "\n\n**Documents Edited:**\n\n")?;
                    for document in updated_documents {
                        writeln!(f, "**Document {}**", document.document_id)?;
                        if !document.content.trim().is_empty() {
                            writeln!(f, "```\n{}\n```\n", document.content)?;
                        }
                    }
                    Ok(())
                }
                EditDocumentsResult::Error(error) => {
                    write!(f, "\n_Edit documents error: {error} _")
                }
                EditDocumentsResult::Cancelled => write!(f, "\n_Edit documents cancelled_"),
            },
            AIAgentActionResultType::CreateDocuments(result) => match result {
                CreateDocumentsResult::Success { created_documents } => {
                    write!(f, "\n\n**Documents Created:**\n\n")?;
                    for document in created_documents {
                        writeln!(f, "**Document {}**", document.document_id)?;
                        if !document.content.trim().is_empty() {
                            writeln!(f, "```\n{}\n```\n", document.content)?;
                        }
                    }
                    Ok(())
                }
                CreateDocumentsResult::Error(error) => {
                    write!(f, "\n_Create documents error: {error} _")
                }
                CreateDocumentsResult::Cancelled => write!(f, "\n_Create documents cancelled_"),
            },
            AIAgentActionResultType::ReadShellCommandOutput(result) => match result {
                ReadShellCommandOutputResult::CommandFinished { output, .. } => {
                    write!(f, "\n```\n{output}\n```")
                }
                ReadShellCommandOutputResult::LongRunningCommandSnapshot {
                    command,
                    grid_contents,
                    ..
                } => {
                    write!(
                        f,
                        "\n```bash\n{command}\n```\n\n**Current Output:**\n```\n{grid_contents}\n```"
                    )
                }
                ReadShellCommandOutputResult::Cancelled => {
                    write!(f, "\n_Command cancelled_")
                }
                ReadShellCommandOutputResult::Error(e) => {
                    write!(f, "\n_Read shell command output failed: {e:?}_")
                }
            },
            other => {
                write!(f, "{other}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AIAgentActionResult {
    pub id: AIAgentActionId,
    pub task_id: TaskId,
    pub result: AIAgentActionResultType,
}

impl Display for AIAgentActionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.result)
    }
}

impl AIAgentActionResult {
    /// Returns `true` if this action was explicitly rejected by the user.
    pub fn is_rejected(&self) -> bool {
        matches!(
            self.result,
            AIAgentActionResultType::RequestFileEdits(RequestFileEditsResult::Cancelled)
                | AIAgentActionResultType::RequestCommandOutput(
                    RequestCommandOutputResult::CancelledBeforeExecution
                )
                | AIAgentActionResultType::ReadFiles(ReadFilesResult::Cancelled)
                | AIAgentActionResultType::SearchCodebase(SearchCodebaseResult::Cancelled)
                | AIAgentActionResultType::Grep(GrepResult::Cancelled)
                | AIAgentActionResultType::FileGlobV2(FileGlobV2Result::Cancelled),
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FormattedTextLineWrapper {
    /// The raw text with the Markdown formatting syntax stripped.
    /// This is needed for find & link/secret detection.
    stripped_text: String,
    /// Pre-extracted URL hyperlinks from this line.
    /// The AI formatted text wrapper only supports URL hyperlinks (since it's constructed via markdown).
    hyperlinks: Vec<(Range<usize>, String)>,
}

impl FormattedTextLineWrapper {
    /// Returns the raw text with the Markdown formatting syntax stripped.
    pub fn raw_text(&self) -> &str {
        &self.stripped_text
    }

    pub fn hyperlinks(&self) -> Vec<(Range<usize>, String)> {
        self.hyperlinks.clone()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FormattedTextWrapper {
    /// Private to prevent direct mutation that would desync the cached `formatted_text` Arc.
    lines: Vec<FormattedTextLineWrapper>,
    formatted_text: Arc<FormattedText>,
}

impl PartialEq for FormattedTextWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.lines == other.lines
    }
}

impl Eq for FormattedTextWrapper {}

impl FormattedTextWrapper {
    pub fn lines(&self) -> &[FormattedTextLineWrapper] {
        &self.lines
    }

    /// Returns a cheap clone of the cached [`FormattedText`], avoiding a per-call deep copy.
    pub fn formatted_text_arc(&self) -> Arc<FormattedText> {
        Arc::clone(&self.formatted_text)
    }
}

impl From<FormattedText> for FormattedTextWrapper {
    fn from(value: FormattedText) -> Self {
        let formatted_text = Arc::new(value);
        let lines = formatted_text
            .lines
            .iter()
            .map(|line| FormattedTextLineWrapper {
                stripped_text: line.raw_text(),
                hyperlinks: line
                    .hyperlinks(true)
                    .into_iter()
                    .filter_map(|(r, u)| Some((r, u.url()?)))
                    .collect(),
            })
            .collect();
        Self {
            lines,
            formatted_text,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AgentOutputText {
    pub(crate) formatted_lines: Option<FormattedTextWrapper>,
    /// The raw text with the Markdown formatting syntax. This is needed for restoring the
    /// Markdown formatting when reopening warp.
    markdown_text: String,
}

impl AgentOutputText {
    /// Returns the original responded text with the Markdown format syntax.
    pub fn text(&self) -> &str {
        self.markdown_text.as_str()
    }

    /// Note that mutating the returned string will not automatically reparse the text and update `formatted_lines`.
    pub fn mut_text(&mut self) -> &mut String {
        &mut self.markdown_text
    }

    pub fn reparse_markdown(&mut self) {
        let parsed_result = parse_markdown(self.markdown_text.as_str());
        self.formatted_lines = parsed_result.map(|formatted| formatted.into()).ok();
    }
}

impl From<String> for AgentOutputText {
    fn from(value: String) -> Self {
        let parsed_result = parse_markdown(value.as_str());
        Self {
            formatted_lines: parsed_result.map(|formatted| formatted.into()).ok(),
            markdown_text: value,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AgentOutputTable {
    pub markdown_source: String,
    table: FormattedTable,
}

impl AgentOutputTable {
    pub fn structured(markdown_source: String, table: FormattedTable) -> Self {
        Self {
            markdown_source,
            table,
        }
    }

    fn plain_text_for_cell(cell: &FormattedTextInline) -> String {
        cell.iter().map(|fragment| fragment.text.as_str()).collect()
    }

    fn plain_text_for_row(cells: &[FormattedTextInline]) -> String {
        cells
            .iter()
            .map(Self::plain_text_for_cell)
            .collect::<Vec<_>>()
            .join("\t")
    }

    pub fn rendered_lines(&self) -> Vec<String> {
        let mut lines = Vec::with_capacity(1 + self.table.rows.len());
        lines.push(Self::plain_text_for_row(&self.table.headers));
        lines.extend(
            self.table
                .rows
                .iter()
                .map(|row| Self::plain_text_for_row(row)),
        );
        lines
    }

    pub fn table(&self) -> &FormattedTable {
        &self.table
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AgentOutputImage {
    pub alt_text: String,
    pub source: String,
    /// Optional CommonMark image title preserved from `![alt](src "title")`.
    /// Empty titles are normalized to `None` by the shared markdown parser.
    pub title: Option<String>,
    pub markdown_source: String,
    pub layout: AgentOutputImageLayout,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AgentOutputMermaidDiagram {
    pub source: String,
    pub markdown_source: String,
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AIAgentTextSection {
    /// Plain textual output from the AI.
    PlainText { text: AgentOutputText },
    /// A snippet of code included as part of the AI's output.
    Code {
        code: String,
        language: Option<ProgrammingLanguage>,
        source: Option<CodeSource>,
    },
    /// A formatted markdown table rendered in a text block.
    Table { table: AgentOutputTable },
    /// A markdown image rendered as a visual block.
    Image { image: AgentOutputImage },
    /// A Mermaid diagram rendered as a visual block.
    MermaidDiagram { diagram: AgentOutputMermaidDiagram },
}

impl AIAgentTextSection {
    pub fn is_empty(&self) -> bool {
        match self {
            AIAgentTextSection::PlainText { text } => text.text().is_empty(),
            AIAgentTextSection::Code { code, .. } => code.is_empty(),
            AIAgentTextSection::Table { table } => table.markdown_source.is_empty(),
            AIAgentTextSection::Image { image } => image.markdown_source.is_empty(),
            AIAgentTextSection::MermaidDiagram { diagram } => diagram.markdown_source.is_empty(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AIAgentText {
    pub sections: Vec<AIAgentTextSection>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AIAgentTodoId(String);

impl AsRef<str> for AIAgentTodoId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for AIAgentTodoId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<AIAgentTodoId> for String {
    fn from(value: AIAgentTodoId) -> Self {
        value.0
    }
}

impl Display for AIAgentTodoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AIAgentTodo {
    pub id: AIAgentTodoId,
    pub title: String,
    pub description: String,
}

impl AIAgentTodo {
    pub fn new(id: AIAgentTodoId, title: String, description: String) -> Self {
        Self {
            id,
            title,
            description,
        }
    }
}

impl Display for AIAgentTodo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.id, self.title)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TodoOperation {
    UpdateTodos { todos: Vec<AIAgentTodo> },
    MarkAsCompleted { completed_todos: Vec<AIAgentTodo> },
}

impl Display for TodoOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TodoOperation::UpdateTodos { todos } => {
                write!(f, "UpdateTodos: {} items", todos.len())
            }
            TodoOperation::MarkAsCompleted { completed_todos } => {
                write!(f, "MarkAsCompleted: {} items", completed_todos.len())
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SubagentType {
    Cli,
    Research,
    Advice,
    ComputerUse,
    Summarization,
    ConversationSearch {
        query: Option<String>,
        /// The ID of the conversation being searched. None when searching the
        /// current conversation.
        conversation_id: Option<String>,
    },
    Unknown,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SubagentCall {
    pub task_id: String,
    pub subagent_type: SubagentType,
}

impl Display for SubagentCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Subagent: {}", self.task_id)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AIAgentOutputMessageType {
    Text(AIAgentText),
    Reasoning {
        text: AIAgentText,
        /// How long the Agent reasoned for.
        /// Only populated when the Agent is done reasoning.
        finished_duration: Option<Duration>,
    },
    Summarization {
        /// The summarization text sections.
        text: AIAgentText,
        /// How long the Agent spent summarizing.
        /// Only populated when the summarization is done.
        finished_duration: Option<Duration>,
        summarization_type: SummarizationType,
        /// Number of tokens in the summarization.
        /// Only populated for ConversationSummary during/after summarization.
        token_count: Option<u32>,
    },
    Subagent(SubagentCall),
    Action(AIAgentAction),
    TodoOperation(TodoOperation),
    AcpToolCall(AcpToolCall),
    AcpPlan(AcpPlan),
    AcpPermission(AcpPermissionRequest),
    CommentsAddressed {
        comments: Vec<ReviewComment>,
    },
    /// Debug-only output message for staging/dev builds.
    DebugOutput {
        text: String,
    },
    /// Notification that an artifact was created (e.g. a PR).
    ArtifactCreated(ArtifactCreatedData),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ArtifactCreatedData {
    PullRequest {
        url: String,
        branch: String,
    },
    Screenshot {
        artifact_uid: String,
        mime_type: String,
        description: Option<String>,
    },
    File {
        artifact_uid: String,
        filepath: String,
        filename: String,
        mime_type: String,
        description: Option<String>,
        size_bytes: i64,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SummarizationType {
    ConversationSummary,
    ToolCallResultSummary,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MessageId(String);

impl MessageId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
}

impl Deref for MessageId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A single output message received in an AI's response to some [`AIAgentInput`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AIAgentOutputMessage {
    pub id: MessageId,
    pub message: AIAgentOutputMessageType,
    pub citations: Vec<AIAgentCitation>,
}

impl Display for AIAgentOutputMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.message {
            AIAgentOutputMessageType::Text(text)
            | AIAgentOutputMessageType::Reasoning { text, .. }
            | AIAgentOutputMessageType::Summarization { text, .. } => {
                if matches!(self.message, AIAgentOutputMessageType::Reasoning { .. }) {
                    write!(f, "LLM Reasoning: ")?;
                } else if matches!(self.message, AIAgentOutputMessageType::Summarization { .. }) {
                    write!(f, "Conversation Summary: ")?;
                }
                for (i, section) in text.sections.iter().enumerate() {
                    if i > 0 {
                        writeln!(f)?;
                    }
                    match section {
                        AIAgentTextSection::PlainText { text } => write!(f, "{}", text.text())?,
                        AIAgentTextSection::Code {
                            code,
                            language,
                            source,
                        } => {
                            write!(f, "```")?;
                            if let Some(lang) = language {
                                write!(f, "{lang}")?;
                            }
                            if let Some(CodeSource::Link {
                                path,
                                range_start,
                                range_end,
                            }) = source
                            {
                                write!(f, " path={path:?}")?;
                                if let (Some(range_start), Some(range_end)) =
                                    (range_start, range_end)
                                {
                                    write!(
                                        f,
                                        " start={} end={}",
                                        range_start.line_num, range_end.line_num
                                    )?;
                                }
                            }
                            writeln!(f)?;
                            writeln!(f, "{code}")?;
                            write!(f, "```")
                        }?,
                        AIAgentTextSection::Table { table } => {
                            { write!(f, "{}", table.markdown_source) }?
                        }
                        AIAgentTextSection::Image { image } => {
                            write!(f, "{}", image.markdown_source)?
                        }
                        AIAgentTextSection::MermaidDiagram { diagram } => {
                            write!(f, "{}", diagram.markdown_source)?
                        }
                    }
                }
            }
            AIAgentOutputMessageType::Action(action) => write!(f, "Action: {action}")?,
            AIAgentOutputMessageType::TodoOperation(todo) => write!(f, "Todo: {todo}")?,
            AIAgentOutputMessageType::Subagent(subagent) => write!(f, "Subagent: {subagent}")?,
            AIAgentOutputMessageType::AcpToolCall(tool_call) => {
                write!(f, "ACP Tool Call: {}", tool_call.title)?
            }
            AIAgentOutputMessageType::AcpPlan(plan) => {
                write!(f, "ACP Plan: {} entries", plan.plan.entries.len())?
            }
            AIAgentOutputMessageType::AcpPermission(request) => {
                write!(f, "ACP Permission Request: {}", request.request_id)?
            }
            AIAgentOutputMessageType::CommentsAddressed {
                comments: comment_ids,
            } => write!(f, "Addressed {} comments", comment_ids.len())?,
            AIAgentOutputMessageType::DebugOutput { text } => write!(f, "[DEBUG] {text}")?,
            AIAgentOutputMessageType::ArtifactCreated(data) => match data {
                ArtifactCreatedData::PullRequest { url, branch } => {
                    write!(f, "Created PR: {url} (branch: {branch})")?
                }
                ArtifactCreatedData::Screenshot { artifact_uid, .. } => {
                    write!(f, "Screenshot captured (artifact: {artifact_uid})")?
                }
                ArtifactCreatedData::File {
                    artifact_uid,
                    filepath,
                    ..
                } => write!(
                    f,
                    "File artifact uploaded: {filepath} (artifact: {artifact_uid})"
                )?,
            },
        }

        if !self.citations.is_empty() {
            writeln!(f)?;
            writeln!(f, "Citations:")?;
            for citation in &self.citations {
                writeln!(f, "  - {citation}")?
            }
        }
        Ok(())
    }
}

impl AIAgentOutputMessage {
    pub fn action(id: MessageId, action: AIAgentAction) -> Self {
        Self {
            id,
            message: AIAgentOutputMessageType::Action(action),
            citations: vec![],
        }
    }

    pub fn text(id: MessageId, text: AIAgentText) -> Self {
        Self {
            id,
            message: AIAgentOutputMessageType::Text(text),
            citations: vec![],
        }
    }

    pub fn subagent(id: MessageId, subagent: SubagentCall) -> Self {
        Self {
            id,
            message: AIAgentOutputMessageType::Subagent(subagent),
            citations: vec![],
        }
    }

    pub fn reasoning(id: MessageId, text: AIAgentText, duration: Option<Duration>) -> Self {
        Self {
            id,
            message: AIAgentOutputMessageType::Reasoning {
                text,
                finished_duration: duration,
            },
            citations: vec![],
        }
    }

    pub fn todo_operation(id: MessageId, operation: TodoOperation) -> Self {
        Self {
            id,
            message: AIAgentOutputMessageType::TodoOperation(operation),
            citations: vec![],
        }
    }

    pub fn comments_addressed(id: MessageId, comments: Vec<ReviewComment>) -> Self {
        Self {
            id,
            message: AIAgentOutputMessageType::CommentsAddressed { comments },
            citations: vec![],
        }
    }

    pub fn debug_output(id: MessageId, text: String) -> Self {
        Self {
            id,
            message: AIAgentOutputMessageType::DebugOutput { text },
            citations: vec![],
        }
    }

    pub fn summarization(
        id: MessageId,
        text: AIAgentText,
        duration: Option<Duration>,
        summarization_type: SummarizationType,
        token_count: Option<u32>,
    ) -> Self {
        Self {
            id,
            message: AIAgentOutputMessageType::Summarization {
                text,
                finished_duration: duration,
                summarization_type,
                token_count,
            },
            citations: vec![],
        }
    }

    pub fn acp_tool_call(id: MessageId, tool_call: AcpToolCall) -> Self {
        Self {
            id,
            message: AIAgentOutputMessageType::AcpToolCall(tool_call),
            citations: vec![],
        }
    }

    pub fn acp_plan(id: MessageId, plan: AcpPlan) -> Self {
        Self {
            id,
            message: AIAgentOutputMessageType::AcpPlan(plan),
            citations: vec![],
        }
    }

    pub fn acp_permission(id: MessageId, request: AcpPermissionRequest) -> Self {
        Self {
            id,
            message: AIAgentOutputMessageType::AcpPermission(request),
            citations: vec![],
        }
    }

    pub fn artifact_created(id: MessageId, data: ArtifactCreatedData) -> Self {
        Self {
            id,
            message: AIAgentOutputMessageType::ArtifactCreated(data),
            citations: vec![],
        }
    }

    pub fn with_citations(self, citations: Vec<AIAgentCitation>) -> Self {
        Self { citations, ..self }
    }
}

/// Contains context that may be attached to a user query.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AIAgentContext {
    Directory {
        pwd: Option<String>,
        home_dir: Option<String>,
        are_file_symbols_indexed: bool,
    },

    /// Text selected via the cursor within the block list.
    SelectedText(String),

    /// Information about the execution environment (OS, shell type and version) is included in the
    /// query.
    ExecutionEnvironment(AiExecutionContext),

    /// The current date and time.
    CurrentTime {
        current_time: DateTime<Local>,
    },

    /// An image attached to the query.
    Image(ImageContext),

    /// Indexed codebase possibly relevant to the query.
    Codebase {
        /// Absolute path to the indexed codebase.
        path: String,
        /// Repository name.
        name: String,
    },

    ProjectRules {
        root_path: String,
        active_rules: Vec<FileContext>,
        additional_rule_paths: Vec<String>,
    },

    File(FileContext),

    Git {
        head: String,
        branch: Option<String>,
    },

    #[serde(untagged)]
    Block(Box<BlockContext>),
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ImageContext {
    /// Base64-encoded image data.
    pub data: String,

    /// MIME type of the media content (e.g., "image/jpeg", "image/png")
    pub mime_type: String,

    pub file_name: String,

    /// Whether this image was exported from Figma, detected via
    /// the `Software: Figma` PNG metadata field.
    #[serde(default)]
    pub is_figma: bool,
}

impl std::fmt::Debug for ImageContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // We log dispatching typed actions (with `ImageContext` as an argument) and we don't want
        // to log any UGC in prod.
        f.debug_struct("ImageContext")
            .field("data", &"REDACTED_B64_IMAGE_DATA_UGC")
            .field("mime_type", &self.mime_type)
            .field("file_name", &"REDACTED_FILE_NAME_UGC")
            .finish()
    }
}

/// Source of a document content attachment.
/// Used to identify user-attached plans to track in the UI.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentContentAttachmentSource {
    UserAttached,
    PlanEdited,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AIAgentAttachment {
    PlainText(String),
    DocumentContent {
        document_id: String,
        content: String,
        source: DocumentContentAttachmentSource,
        line_range: Option<Range<LineCount>>,
    },
    DiffHunk {
        file_path: String,
        line_range: Range<LineCount>,
        diff_content: String,
        lines_added: u32,
        lines_removed: u32,
        current: Option<CurrentHead>,
        base: DiffBase,
    },
    DiffSet {
        /// Map from file path to list of diff hunks for that file
        file_diffs: HashMap<String, Vec<DiffSetHunk>>,
        /// Git branch information for the diff
        current: Option<CurrentHead>,
        base: DiffBase,
    },
    /// Reference to a local file attachment.
    FilePathReference {
        file_id: String,
        /// The original filename.
        file_name: String,
        /// The full resolved path on disk where the file was downloaded.
        file_path: String,
    },
    #[serde(untagged)]
    Block(BlockContext),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurrentHead {
    BranchName(String),
    HeadlessCommitSha(String),
}

impl CurrentHead {
    pub fn title(&self) -> String {
        match self {
            CurrentHead::BranchName(name) => name.clone(),
            CurrentHead::HeadlessCommitSha(sha) => {
                let short = sha.chars().take(7).collect::<String>();
                format!("Commit {short}")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffBase {
    BranchName(String),
    HeadlessCommitSha(String),
    UncommittedChanges,
}

/// A simplified diff hunk for use in DiffSet attachments
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffSetHunk {
    pub line_range: Range<LineCount>,
    pub diff_content: String,
    pub lines_added: u32,
    pub lines_removed: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaticQueryType {
    Install,
    Code,
    Deploy,
    SomethingElse,
    EvaluationSuite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShellCommandCompletedTrigger {
    // We heap-allocate this because it's large and bloats the size of the
    // `ShellCommandCompleted` enum variant relative to other variants.
    pub executed_shell_command: Box<BlockContext>,
    pub relevant_files: Vec<FileContext>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[allow(clippy::enum_variant_names)]
pub enum PassiveSuggestionTrigger {
    ShellCommandCompleted(ShellCommandCompletedTrigger),
}

impl PassiveSuggestionTrigger {
    /// Returns the block ID that triggered this passive suggestion
    /// iff the trigger type was [Self::ShellCommandCompleted].
    pub fn block_id(&self) -> Option<BlockId> {
        match self {
            Self::ShellCommandCompleted(c) => Some(c.executed_shell_command.id.clone()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UserQueryMode {
    #[default]
    Normal,
    Plan,
}

pub fn extract_user_query_mode(query: String) -> (String, UserQueryMode) {
    if let Some(query) = commands::strip_command_prefix(&query, commands::PLAN_NAME) {
        (query, UserQueryMode::Plan)
    } else {
        (query, UserQueryMode::Normal)
    }
}

/// Reconstructs the display form of a user query that has been stripped via
/// [`extract_user_query_mode`], by re-prepending the slash-command prefix
/// associated with [`UserQueryMode`].
///
/// This is the inverse of [`extract_user_query_mode`] and the canonical way
/// for UI to render a stored `(mode, query)` pair so the displayed prompt
/// always matches what the user originally submitted.
pub fn display_user_query_with_mode(mode: UserQueryMode, query: &str) -> String {
    match mode {
        UserQueryMode::Normal => query.to_owned(),
        UserQueryMode::Plan => format!("{} {query}", commands::PLAN_NAME),
    }
}

// TODO(zachbai): Refactor this to consolidate with `LongRunningCommandSnapshot` and `Snapshot`
// variants of `ReadShellCommandOutputResult` and `WriteToLongRunningShellCommandResult`.
#[derive(Clone, Debug, PartialEq)]
pub struct RunningCommand {
    pub command: String,
    pub block_id: BlockId,
    pub grid_contents: String,
    pub cursor: String,
    pub requested_command_id: Option<AIAgentActionId>,
    pub is_alt_screen_active: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AIAgentInput {
    /// A user's query to the AI.
    UserQuery {
        query: String,
        context: Arc<[AIAgentContext]>,
        static_query_type: Option<StaticQueryType>,
        referenced_attachments: HashMap<String, AIAgentAttachment>,
        user_query_mode: UserQueryMode,
        running_command: Option<RunningCommand>,
    },

    AutoCodeDiffQuery {
        query: String,
        context: Arc<[AIAgentContext]>,
    },

    ResumeConversation {
        context: Arc<[AIAgentContext]>,
    },

    InitProjectRules {
        context: Arc<[AIAgentContext]>,
        display_query: Option<String>,
    },

    CreateNewProject {
        query: String,
        context: Arc<[AIAgentContext]>,
    },

    CloneRepository {
        clone_repo_url: CloneRepositoryURL,
        context: Arc<[AIAgentContext]>,
    },

    /// A batch of inline code review comments for the agent to address.
    CodeReview {
        context: Arc<[AIAgentContext]>,
        review_comments: AgentReviewCommentBatch,
    },

    FetchReviewComments {
        repo_path: String,
        context: Arc<[AIAgentContext]>,
    },

    /// The result of an `AIAgentAction`, relayed back to the LLM for it to continue answering a
    /// user query.
    ActionResult {
        result: AIAgentActionResult,
        context: Arc<[AIAgentContext]>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentReviewCommentBatch {
    /// The review comments in this batch. Uses `code_review::comments::ReviewComment`
    /// because it contains full target information needed for API conversion and UI rendering.
    pub comments: Vec<CodeReviewComment>,
    /// All diff hunks that have comments in this batch attached to them, grouped by file name.
    pub diff_set: HashMap<String, Vec<DiffSetHunk>>,
}

impl AgentReviewCommentBatch {
    pub fn review_comments(&self) -> ReviewCommentBatch {
        ReviewCommentBatch::from_comments(self.comments.clone())
    }
}

/// A simple struct that holds a URL to be used for the CloneRepository input.
///
/// Needed because we want to display a query that's more than just the URL to the user
/// and the code is setup such that the query string must be preallocated.
#[derive(Clone, Debug, PartialEq)]
pub struct CloneRepositoryURL {
    /// The query displayed to the user when a user clones a repository.
    query: String,

    /// The URL of the repository to clone.
    url: String,
}

impl CloneRepositoryURL {
    pub fn new(url: String) -> Self {
        Self {
            query: format!("Clone {url}"),
            url,
        }
    }

    pub fn into_url(self) -> String {
        self.url
    }
}

impl Display for AIAgentInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserQuery { .. } => {
                write!(f, "UserQuery: {}", self.user_query().unwrap_or_default())
            }
            Self::AutoCodeDiffQuery { query, .. } => {
                write!(f, "AutoCodeDiffQuery: {query}")
            }
            Self::ActionResult { result, .. } => write!(f, "ActionResult: {result}"),
            Self::ResumeConversation { .. } => write!(f, "ResumeConversation"),
            Self::InitProjectRules { .. } => write!(f, "InitProjectRules"),
            Self::CreateNewProject { .. } => write!(f, "CreateNewProject"),
            Self::CloneRepository { .. } => write!(f, "CloneRepository"),
            Self::CodeReview { .. } => write!(f, "CodeReview"),
            Self::FetchReviewComments { .. } => write!(f, "FetchReviewComments"),
        }
    }
}

impl AIAgentInput {
    pub fn user_query(&self) -> Option<String> {
        match self {
            Self::UserQuery {
                query,
                user_query_mode,
                ..
            } => Some(display_user_query_with_mode(*user_query_mode, query)),
            Self::CreateNewProject { query, .. } => Some(query.clone()),
            Self::CloneRepository {
                clone_repo_url: url,
                ..
            } => Some(url.query.clone()),
            Self::InitProjectRules { display_query, .. } => display_query.clone(),
            Self::CodeReview { .. } => Some("Address these comments".to_string()),
            Self::FetchReviewComments { .. } => Some(commands::PR_COMMENTS_NAME.to_string()),
            Self::AutoCodeDiffQuery { .. }
            | Self::ActionResult { .. }
            | Self::ResumeConversation { .. } => None,
        }
    }

    /// Returns the user query text as it should be displayed in the UI.
    /// This includes the "/agent" prefix for the initial conversation query.
    pub fn display_user_query(
        &self,
        initial_conversation_query: Option<&String>,
    ) -> Option<String> {
        let mut query = self.user_query()?;
        if self
            .user_query_mode()
            .is_none_or(|mode| matches!(mode, UserQueryMode::Normal))
            && Some(&query) == initial_conversation_query
            && !self.has_custom_display_query()
        {
            query = format!("/agent {query}");
        }
        Some(query)
    }

    pub fn user_query_mode(&self) -> Option<UserQueryMode> {
        match self {
            AIAgentInput::UserQuery {
                user_query_mode, ..
            } => Some(*user_query_mode),
            _ => None,
        }
    }

    pub fn action_result(&self) -> Option<&AIAgentActionResult> {
        match self {
            Self::ActionResult { result, .. } => Some(result),
            _ => None,
        }
    }

    pub fn auto_code_diff_query(&self) -> Option<&str> {
        let Self::AutoCodeDiffQuery { query, .. } = self else {
            return None;
        };
        Some(query.as_str())
    }

    pub fn is_user_query(&self) -> bool {
        matches!(self, AIAgentInput::UserQuery { .. })
    }

    pub fn is_passive_request(&self) -> bool {
        matches!(self, AIAgentInput::AutoCodeDiffQuery { .. })
    }

    pub fn context(&self) -> Option<&[AIAgentContext]> {
        match self {
            Self::UserQuery { context, .. }
            | Self::ActionResult { context, .. }
            | Self::AutoCodeDiffQuery { context, .. }
            | Self::ResumeConversation { context, .. }
            | Self::InitProjectRules { context, .. }
            | Self::CreateNewProject { context, .. }
            | Self::CloneRepository { context, .. }
            | Self::CodeReview { context, .. }
            | Self::FetchReviewComments { context, .. } => Some(context),
        }
    }

    /// Returns all of the attachments for the given input,
    /// converting any blocks blocks attached in the context into the correct type of attachment.
    pub fn attachments(&self) -> Option<Vec<AIAgentAttachment>> {
        match self {
            Self::UserQuery {
                referenced_attachments,
                ..
            } => {
                let res: Vec<AIAgentAttachment> =
                    referenced_attachments.values().cloned().collect();
                Some(res)
            }
            Self::ActionResult { .. }
            | Self::AutoCodeDiffQuery { .. }
            | Self::ResumeConversation { .. }
            | Self::InitProjectRules { .. }
            | Self::CreateNewProject { .. }
            | Self::CloneRepository { .. }
            | Self::CodeReview { .. }
            | Self::FetchReviewComments { .. } => None,
        }
    }

    pub fn is_auto_code_diff_query(&self) -> bool {
        matches!(self, AIAgentInput::AutoCodeDiffQuery { .. })
    }

    /// Returns true if this input type provides its own display query that should be preserved
    /// without prepending "/agent".
    pub fn has_custom_display_query(&self) -> bool {
        matches!(
            self,
            AIAgentInput::InitProjectRules { .. } | AIAgentInput::FetchReviewComments { .. }
        )
    }
}

/// A globally unique ID for an `AIAgentExchange`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AIAgentExchangeId(Uuid);

impl Display for AIAgentExchangeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AIAgentExchangeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AIAgentExchangeId {
    fn default() -> Self {
        Self::new()
    }
}

impl TryFrom<String> for AIAgentExchangeId {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(Uuid::try_parse(&value)?))
    }
}

/// Represents a single user input/AI output pair. Each exchange corresponds to a request to an AI
/// backend model and its response.
#[derive(Debug, Clone)]
pub struct AIAgentExchange {
    /// Unique ID for the exchange.
    pub id: AIAgentExchangeId,

    /// The input originating from the user.
    pub input: Vec<AIAgentInput>,

    /// The status of the output stream. Updated during the course of the exchange.
    pub output_status: AIAgentOutputStatus,

    /// The ids for all messages added to the task in this exchange.
    pub added_message_ids: HashSet<MessageId>,

    /// The time the input was sent.
    pub start_time: DateTime<Local>,

    /// The time the exchange's output finished streaming, if known.
    pub finish_time: Option<DateTime<Local>>,

    // TODO(CORE-3546): add shell launch data when the input was submitted.
    /// The current working directory when the input was submitted.
    pub working_directory: Option<String>,

    /// The model to which the request was sent.
    pub model_id: LLMId,

    /// The coding model to which the request was sent.
    pub coding_model_id: LLMId,

    /// The CLI agent model to which the request was sent.
    pub cli_agent_model_id: LLMId,

    /// The computer use model to which the request was sent.
    pub computer_use_model_id: LLMId,
}

impl AIAgentExchange {
    /// Format the user input part of this exchange for copying to clipboard.
    /// We don't copy tool call results.
    pub fn format_input_for_copy(&self) -> String {
        let user_queries: Vec<String> = self
            .input
            .iter()
            .filter_map(|input| input.user_query())
            .collect();
        user_queries.join("\n")
    }

    /// Format the output part of this exchange for copying to clipboard.
    pub fn format_output_for_copy(
        &self,
        action_model: Option<&crate::ai::blocklist::BlocklistAIActionModel>,
    ) -> String {
        match self.output_status.output() {
            Some(output) => output.get().format_for_copy(action_model),
            None => String::new(),
        }
    }

    /// Format the entire exchange (both input and output) for copying to clipboard.
    /// Always adds USER: and AGENT: labels.
    /// If `skip_agent_label` is true, skips the AGENT: label (for consecutive agent outputs).
    pub fn format_for_copy(
        &self,
        action_model: Option<&crate::ai::blocklist::BlocklistAIActionModel>,
    ) -> String {
        let input_text = self.format_input_for_copy();
        let output_text = self.format_output_for_copy(action_model);
        let has_user_input = !input_text.is_empty();
        let has_agent_output = !output_text.is_empty();

        if !has_user_input && !has_agent_output {
            return String::new();
        }

        let mut parts = Vec::new();

        if has_user_input {
            parts.push(format!("USER:\n{input_text}"));
        }

        if has_agent_output {
            if has_user_input {
                parts.push(format!("AGENT:\n{output_text}"));
            } else {
                parts.push(output_text);
            }
        }

        parts.join("\n\n")
    }

    pub fn has_user_query(&self) -> bool {
        self.input.iter().any(|input| input.user_query().is_some())
    }

    pub fn has_accepted_file_edit(&self) -> bool {
        self.input.iter().any(|input| {
            matches!(
                input.action_result(),
                Some(AIAgentActionResult {
                    result: AIAgentActionResultType::RequestFileEdits(
                        RequestFileEditsResult::Success { .. }
                    ),
                    ..
                })
            )
        })
    }

    pub fn has_passive_request(&self) -> bool {
        self.input.iter().any(|input| input.is_passive_request())
    }

    pub fn has_passive_code_diff(&self) -> bool {
        self.input
            .iter()
            .any(|input| input.auto_code_diff_query().is_some())
    }

    pub fn duration(&self) -> Option<TimeDelta> {
        self.finish_time
            .map(|finish_time| finish_time.signed_duration_since(self.start_time))
    }
}
