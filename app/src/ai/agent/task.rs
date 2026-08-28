pub mod transaction;

use uuid::Uuid;

use crate::terminal::model::block::BlockId;

use super::{
    AIAgentContext, AIAgentExchange, AIAgentExchangeId, MessageId,
    conversation::context_in_exchanges,
};

pub use ai_types::TaskId;

#[derive(Debug, thiserror::Error)]
pub enum UpdateTaskError {
    #[error("Attempted to update already-finished output.")]
    OutputAlreadyFinished,
}

#[derive(Debug, Clone)]
enum TaskKind {
    Root,
    CLIAgent { block_id: BlockId },
}

#[derive(Debug, Clone)]
pub struct Task {
    id: TaskId,
    kind: TaskKind,
    description: String,
    exchanges: Vec<AIAgentExchange>,
}

impl Task {
    pub(super) fn new_optimistic_root() -> Self {
        Self {
            id: TaskId::new(Uuid::new_v4().to_string()),
            kind: TaskKind::Root,
            description: String::new(),
            exchanges: vec![],
        }
    }

    pub(super) fn new_optimistic_cli_agent_subtask(block_id: BlockId) -> Self {
        Self {
            id: TaskId::new(Uuid::new_v4().to_string()),
            kind: TaskKind::CLIAgent { block_id },
            description: String::new(),
            exchanges: vec![],
        }
    }

    pub fn id(&self) -> &TaskId {
        &self.id
    }

    pub fn parent_id(&self) -> Option<TaskId> {
        None
    }

    pub fn is_root_task(&self) -> bool {
        matches!(self.kind, TaskKind::Root)
    }

    pub fn is_cli_subagent(&self) -> bool {
        matches!(self.kind, TaskKind::CLIAgent { .. })
    }

    pub fn cli_subagent_block_id(&self) -> Option<BlockId> {
        match &self.kind {
            TaskKind::CLIAgent { block_id } => Some(block_id.clone()),
            TaskKind::Root => None,
        }
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub(super) fn update_description(&mut self, description: String) {
        self.description = description;
    }

    pub fn exchanges(&self) -> impl Iterator<Item = &AIAgentExchange> {
        self.exchanges.iter()
    }

    pub fn exchange(&self, exchange_id: AIAgentExchangeId) -> Option<&AIAgentExchange> {
        self.exchanges
            .iter()
            .find(|exchange| exchange.id == exchange_id)
    }

    pub(super) fn exchange_mut(
        &mut self,
        exchange_id: AIAgentExchangeId,
    ) -> Option<&mut AIAgentExchange> {
        self.exchanges
            .iter_mut()
            .find(|exchange| exchange.id == exchange_id)
    }

    pub fn last_exchange(&self) -> Option<&AIAgentExchange> {
        self.exchanges.last()
    }

    pub fn exchanges_len(&self) -> usize {
        self.exchanges.len()
    }

    pub fn exchanges_reversed(&self) -> impl Iterator<Item = &AIAgentExchange> {
        self.exchanges.iter().rev()
    }

    pub fn all_contexts(&self) -> impl Iterator<Item = &AIAgentContext> {
        context_in_exchanges(self.exchanges())
    }

    pub fn initial_working_directory(&self) -> Option<String> {
        self.exchanges
            .iter()
            .find_map(|exchange| exchange.working_directory.clone())
    }

    pub(super) fn append_exchange(&mut self, exchange: AIAgentExchange) {
        self.exchanges.push(exchange);
    }

    pub(super) fn remove_exchange(
        &mut self,
        exchange_id: AIAgentExchangeId,
    ) -> Option<AIAgentExchange> {
        let index = self
            .exchanges
            .iter()
            .position(|exchange| exchange.id == exchange_id)?;
        Some(self.exchanges.remove(index))
    }

    pub(super) fn truncate_exchanges_from(&mut self, from_exchange_id: AIAgentExchangeId) {
        if let Some(index) = self
            .exchanges
            .iter()
            .position(|exchange| exchange.id == from_exchange_id)
        {
            self.exchanges.truncate(index);
        }
    }

    pub(super) fn reassign_exchange_ids(&mut self) {
        for exchange in &mut self.exchanges {
            exchange.id = AIAgentExchangeId::new();
        }
    }

    pub(super) fn remove_messages(&mut self, message_ids: &std::collections::HashSet<MessageId>) {
        for exchange in &mut self.exchanges {
            exchange
                .added_message_ids
                .retain(|message_id| !message_ids.contains(message_id));
        }
    }
}
