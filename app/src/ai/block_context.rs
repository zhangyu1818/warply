use chrono::{DateTime, Local};
use parking_lot::FairMutex;
use serde::{Deserialize, Serialize};
use warp_core::command::ExitCode;

use crate::terminal::event::UserBlockCompleted;
use crate::terminal::model::TerminalModel;
use crate::terminal::model::block::BlockId;
use crate::terminal::model::terminal_model::BlockIndex;

/// Contains context about a completed terminal command block.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockContext {
    /// The ID of the block whose contents are included in the query.
    ///
    /// This is not actually included in the query payload but is used for tracing blocks
    /// passed as context in conversation history, which may be useful for deduping instances
    /// of passing the same block as context or for rendering UI.
    #[serde(rename = "block_id")]
    pub id: BlockId,
    /// The index into the blocklist where this block is located.
    pub index: BlockIndex,
    pub command: String,
    pub output: String,
    pub exit_code: ExitCode,
    /// Whether this block was auto-attached rather than manually attached by the user.
    pub is_auto_attached: bool,
    /// Timestamp when the command started executing.
    pub started_ts: Option<DateTime<Local>>,
    /// Timestamp when the command finished executing.
    pub finished_ts: Option<DateTime<Local>>,

    // Environment fields — populated by the constructors below, left as
    // None at construction sites that don't need them.
    /// The working directory where the command was executed.
    pub pwd: Option<String>,
    /// The shell type (e.g., "zsh", "bash").
    pub shell: Option<String>,
    /// The username of the user who executed the command.
    pub username: Option<String>,
    /// The hostname of the machine where the command was executed.
    pub hostname: Option<String>,
    /// The git branch at the time of execution.
    pub git_branch: Option<String>,
    /// The operating system name (e.g., "MacOS", "Linux").
    pub os: Option<String>,
    /// The terminal session ID.
    pub session_id: Option<u64>,
}

impl BlockContext {
    /// Construct a BlockContext from a [`UserBlockCompleted`]. `model` is used to resolve the
    /// block's lazily-computed fields (see [`UserBlockCompleted`]'s accessor methods); it's only
    /// locked for fields that aren't already cached.
    pub fn from_completed_block(
        block_completed: &UserBlockCompleted,
        model: &FairMutex<TerminalModel>,
    ) -> Box<Self> {
        let serialized_block = block_completed.serialized_block.get_with(|compute| {
            let model = model.lock();
            compute(model.block_list())
        });
        Box::new(Self {
            id: serialized_block.id.clone(),
            index: block_completed.index,
            command: block_completed
                .command_with_obfuscated_secrets
                .get_with(|compute| {
                    let model = model.lock();
                    compute(model.block_list())
                })
                .to_owned(),
            output: block_completed
                .output_truncated_with_obfuscated_secrets
                .get_with(|compute| {
                    let model = model.lock();
                    compute(model.block_list())
                })
                .to_owned(),
            exit_code: serialized_block.exit_code,
            is_auto_attached: false,
            started_ts: serialized_block.start_ts,
            finished_ts: serialized_block.completed_ts,
            pwd: serialized_block.pwd.clone(),
            shell: serialized_block
                .shell_host
                .as_ref()
                .map(|sh| sh.shell_type.name().to_owned()),
            username: serialized_block
                .shell_host
                .as_ref()
                .map(|sh| sh.user.clone()),
            hostname: serialized_block
                .shell_host
                .as_ref()
                .map(|sh| sh.hostname.clone()),
            git_branch: serialized_block.git_head.clone(),
            os: Some("MacOS".to_owned()),
            session_id: serialized_block.session_id.map(|sid| sid.as_u64()),
        })
    }
}

#[cfg(test)]
#[path = "block_context_tests.rs"]
mod tests;
