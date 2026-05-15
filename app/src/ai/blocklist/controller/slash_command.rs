use std::sync::Arc;

use warp_core::features::FeatureFlag;
use warpui::{AppContext, ModelContext, SingletonEntity};

use crate::{
    ai::{
        agent::{
            conversation::AIConversationId, AIAgentContext, AIAgentInput, CancellationReason,
            CloneRepositoryURL,
        },
        blocklist::agent_view::AgentViewEntryOrigin,
    },
    terminal::input::slash_commands::SlashCommandTrigger,
    BlocklistAIHistoryModel,
};

use super::{input_context_for_request, BlocklistAIController, RequestInput};

pub enum SlashCommandRequest {
    CreateNewProject { query: String },
    CloneRepository { url: String },
    InitProjectRules,
    FetchReviewComments { repo_path: String },
}

impl SlashCommandRequest {
    /// Parses user input into a SlashCommandRequest for slash commands that are handled
    /// via the AI query flow (as opposed to action-based slash commands handled in input.rs).
    pub fn from_query(query: &str) -> Option<SlashCommandRequest> {
        // Check if this is an exact /init query and route it to InitProjectRules instead
        if query == "/init" {
            return Some(Self::InitProjectRules);
        }

        None
    }

    pub(super) fn send_request(
        self,
        controller: &mut BlocklistAIController,
        is_queued_prompt: bool,
        ctx: &mut ModelContext<BlocklistAIController>,
    ) {
        let conversation_id = self.conversation_id(controller, ctx);
        let context = input_context_for_request(
            false,
            controller.context_model.as_ref(ctx),
            controller.active_session.as_ref(ctx),
            conversation_id,
            vec![],
            ctx,
        );
        let inputs = self.input(context);
        if inputs.is_empty() {
            return;
        }
        let active_conversation_id = BlocklistAIHistoryModel::as_ref(ctx)
            .active_conversation_id(controller.terminal_view_id);

        // If no existing conversation, create a new one.
        // When AgentView is enabled, enter agent view which creates the conversation
        // and ensures AI blocks render correctly in the agent view.
        let Some(conversation_id) = conversation_id.or_else(|| {
            if FeatureFlag::AgentView.is_enabled() {
                controller.context_model.update(ctx, |context_model, ctx| {
                    context_model
                        .try_enter_agent_view_for_new_conversation(
                            AgentViewEntryOrigin::SlashCommand {
                                trigger: SlashCommandTrigger::input(),
                            },
                            ctx,
                        )
                        .ok()
                })
            } else {
                Some(controller.start_new_conversation_for_request(ctx).id())
            }
        }) else {
            log::error!("Failed to get conversation ID for slash command request");
            return;
        };

        let cancellation_reason = CancellationReason::FollowUpSubmitted {
            is_for_same_conversation: active_conversation_id
                .is_some_and(|id| id == conversation_id),
        };
        if let Some(active_conversation_id) = active_conversation_id {
            controller.cancel_conversation_progress(
                active_conversation_id,
                cancellation_reason,
                ctx,
            );
        }

        let Some(conversation) =
            BlocklistAIHistoryModel::as_ref(ctx).conversation(&conversation_id)
        else {
            return;
        };

        let request_input = RequestInput::for_task(
            inputs,
            conversation.get_root_task_id().clone(),
            &controller.active_session,
            conversation_id,
            ctx,
        );
        match controller.send_request_input(
            request_input,
            /*default_to_follow_up_on_success*/ true,
            is_queued_prompt,
            ctx,
        ) {
            Ok(_) => {}
            Err(e) => log::error!("Failed to send agent slash command request: {e:?}"),
        }
    }

    pub(super) fn conversation_id(
        &self,
        controller: &BlocklistAIController,
        app: &AppContext,
    ) -> Option<AIConversationId> {
        match self {
            Self::FetchReviewComments { .. } => controller
                .context_model
                .as_ref(app)
                .selected_conversation_id(app),
            _ => None,
        }
    }

    fn input(self, context: Arc<[AIAgentContext]>) -> Vec<AIAgentInput> {
        match self {
            SlashCommandRequest::CreateNewProject { query } => {
                vec![AIAgentInput::CreateNewProject { query, context }]
            }
            SlashCommandRequest::CloneRepository { url } => {
                vec![AIAgentInput::CloneRepository {
                    clone_repo_url: CloneRepositoryURL::new(url),
                    context,
                }]
            }
            SlashCommandRequest::InitProjectRules => vec![AIAgentInput::InitProjectRules {
                context,
                display_query: Some("/init".to_string()),
            }],
            SlashCommandRequest::FetchReviewComments { repo_path } => {
                vec![AIAgentInput::FetchReviewComments { repo_path, context }]
            }
        }
    }
}
