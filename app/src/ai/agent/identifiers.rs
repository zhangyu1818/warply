use serde::Serialize;

use crate::ai::llms::LLMId;

use super::{conversation::AIConversationId, AIAgentExchangeId};

#[derive(Clone, Default, Debug, Serialize)]
pub struct AIIdentifiers {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_conversation_id: Option<AIConversationId>,
    #[serde(rename = "exchange_id", skip_serializing_if = "Option::is_none")]
    pub client_exchange_id: Option<AIAgentExchangeId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<LLMId>,
}
