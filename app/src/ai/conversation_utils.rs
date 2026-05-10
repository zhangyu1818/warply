use warpui::{AppContext, EntityId, SingletonEntity};

use crate::ai::agent::conversation::AIConversationId;
use crate::ai::agent_conversations_model::AgentConversationsModel;
use crate::ai::blocklist::history_model::BlocklistAIHistoryModel;

pub fn delete_conversation(
    conversation_id: AIConversationId,
    terminal_view_id: Option<EntityId>,
    ctx: &mut AppContext,
) {
    BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, model_ctx| {
        history.delete_conversation(conversation_id, terminal_view_id, model_ctx);
    });

    AgentConversationsModel::handle(ctx).update(ctx, |model, ctx| {
        model.sync_conversations(ctx);
    });
}

pub fn remove_conversation(
    conversation_id: AIConversationId,
    terminal_view_id: EntityId,
    ctx: &mut AppContext,
) {
    BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, model_ctx| {
        history.remove_conversation(conversation_id, terminal_view_id, model_ctx);
    });
}
