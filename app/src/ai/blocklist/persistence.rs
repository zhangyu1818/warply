//! Manages how we serialize blocklist AI data for persistence.
#![cfg_attr(not(feature = "local_fs"), allow(dead_code))]

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use crate::{
    ai::{
        agent::{
            AIAgentAttachment, AIAgentContext, AIAgentExchangeId, AIAgentInput, UserQueryMode,
            conversation::AIConversationId,
        },
        llms::LLMId,
    },
    terminal::model::block::SerializedBlock,
};

use super::AIQueryHistoryOutputStatus;
/// Data we persist for each [`AIAgentExchange`] for use in history. Does not contain output data.
#[derive(Debug, Deserialize, Clone)]
pub struct PersistedAIInput {
    pub(crate) exchange_id: AIAgentExchangeId,
    pub(crate) conversation_id: AIConversationId,
    pub(crate) start_ts: DateTime<Local>,
    pub(crate) inputs: Vec<PersistedAIInputType>,
    pub(crate) output_status: AIQueryHistoryOutputStatus,
    pub(crate) working_directory: Option<String>,
    // TODO(CORE-3546): pub(crate) shell: Option<AvailableShell>,
    pub(crate) model_id: LLMId,
    #[allow(unused)]
    pub(crate) coding_model_id: LLMId,
}

/// Pieces of data we need to persist for each [`AIAgentExchange`]'s input for session restoration.
///
/// Note: Only Query is actually used - it's used for up-arrow history.
/// TODO(roland): consider removing the ai_queries table and getting queries from tasks as well.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(crate) enum PersistedAIInputType {
    Query {
        text: String,
        #[serde(default)]
        context: Arc<[AIAgentContext]>,
        #[serde(default)]
        referenced_attachments: HashMap<String, AIAgentAttachment>,
    },
}

impl TryFrom<&AIAgentInput> for PersistedAIInputType {
    type Error = anyhow::Error;

    fn try_from(input: &AIAgentInput) -> Result<Self, Self::Error> {
        match input {
            AIAgentInput::UserQuery {
                query,
                context,
                referenced_attachments,
                ..
            } => Ok(Self::Query {
                text: query.clone(),
                context: context.clone(),
                referenced_attachments: referenced_attachments.clone(),
            }),
            AIAgentInput::AutoCodeDiffQuery { query, context } => Ok(Self::Query {
                text: query.clone(),
                context: context.clone(),
                referenced_attachments: Default::default(),
            }),
            AIAgentInput::ActionResult { .. }
            | AIAgentInput::ResumeConversation { .. }
            | AIAgentInput::InitProjectRules { .. }
            | AIAgentInput::CreateNewProject { .. }
            | AIAgentInput::CloneRepository { .. }
            | AIAgentInput::CodeReview { .. }
            | AIAgentInput::FetchReviewComments { .. } => Err(anyhow::anyhow!(
                "This input type is not persisted. Only Query inputs are persisted for up-arrow history."
            )),
        }
    }
}

impl TryFrom<PersistedAIInputType> for AIAgentInput {
    type Error = anyhow::Error;

    fn try_from(value: PersistedAIInputType) -> Result<Self, Self::Error> {
        match value {
            PersistedAIInputType::Query {
                text,
                context,
                referenced_attachments,
            } => Ok(Self::UserQuery {
                query: text,
                context,
                referenced_attachments,
                static_query_type: None,
                user_query_mode: UserQueryMode::default(),
                running_command: None,
            }),
        }
    }
}

/// The types of "blocks" we can store in our SQLite database for session restoration. Only command
/// blocks are true [`crate::terminal::model::block::Block`]s.
///
/// TODO(roland): now that there is no AI serialized block, consider removing this enum wrapper
#[derive(Debug, Clone, PartialEq)]
pub enum SerializedBlockListItem {
    Command { block: Box<SerializedBlock> },
}

impl SerializedBlockListItem {
    pub(crate) fn start_ts(&self) -> Option<DateTime<Local>> {
        match self {
            Self::Command { block } => block.start_ts,
        }
    }
}

impl From<crate::persistence::model::Block> for SerializedBlockListItem {
    fn from(value: crate::persistence::model::Block) -> Self {
        Self::Command {
            block: Box::new(SerializedBlock::from(value)),
        }
    }
}

impl From<SerializedBlock> for SerializedBlockListItem {
    fn from(value: SerializedBlock) -> Self {
        Self::Command {
            block: Box::new(value),
        }
    }
}
