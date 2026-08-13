//! This module contains model, controller, and view logic for Blocklist AI.
mod action_model;
pub mod agent_view;
pub mod block;
pub mod code_block;
mod context_model;
mod controller;
mod passive_suggestions;
pub(super) use controller::RequestInput;
pub mod history_model;
pub mod inline_action;
mod input_model;
mod permissions;
mod persistence;
pub mod prompt;
pub(crate) mod queued_query;
pub mod summarization_cancel_dialog;

pub(super) mod view_util;

pub(crate) use action_model::{
    BlocklistAIActionEvent, BlocklistAIActionModel, ReadFileContextResult, ShellCommandExecutor,
    ShellCommandExecutorEvent, read_local_file_context,
};

#[cfg(any(test, feature = "integration_tests"))]
pub(crate) use block::model::testing::FakeAIBlockModel;
pub(crate) use block::{AIBlock, AIBlockEvent, init, model};

pub(crate) use context_model::{
    AttachmentType, BlocklistAIContextEvent, BlocklistAIContextModel, PendingAttachment,
    PendingFile, PendingQueryState, block_context_from_terminal_model,
};
pub(crate) use controller::{
    BlocklistAIController, BlocklistAIControllerEvent, ClientIdentifiers, SlashCommandRequest,
    response_stream::ResponseStreamId,
};
pub(crate) use history_model::{
    AIQueryHistory, AIQueryHistoryOutputStatus, AcpResponseStreamTarget, BlocklistAIHistoryEvent,
    BlocklistAIHistoryModel, ConversationStatusUpdate, FORK_PREFIX, PRE_REWIND_PREFIX,
};
pub(crate) use input_model::{
    BlocklistAIInputEvent, BlocklistAIInputModel, InputConfig, InputType,
};
pub(crate) use passive_suggestions::{
    PassiveSuggestionsModels, TerminalPassiveSuggestionsEvent, TerminalPassiveSuggestionsModel,
};
pub(crate) use persistence::PersistedAIInputType;
pub(crate) use persistence::{PersistedAIInput, SerializedBlockListItem};
pub(crate) use queued_query::{
    AutofireAction, QueuedQuery, QueuedQueryEvent, QueuedQueryId, QueuedQueryModel,
    QueuedQueryOrigin, is_lrc_auto_queue_active,
};
pub(crate) use view_util::{
    ATTACH_AS_AGENT_MODE_CONTEXT_TEXT, CLAUDE_ORANGE, ai_brand_color, ai_indicator_height,
    get_ai_block_overflow_menu_element_position_id, get_attached_blocks_chip_element_position_id,
    render_ai_agent_mode_icon,
};

pub use crate::ai::blocklist::block::{TextLocation, secret_redaction};
pub use block::keyboard_navigable_buttons;
pub use block::toggleable_items;
pub use controller::input_context::{
    BLOCK_CONTEXT_ATTACHMENT_REGEX, DIFF_HUNK_ATTACHMENT_REGEX, PLAN_CONTEXT_ATTACHMENT_REGEX,
};
pub use permissions::BlocklistAIPermissions;
