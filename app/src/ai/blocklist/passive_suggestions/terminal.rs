use std::sync::Arc;

use crate::ai::blocklist::controller::{BlocklistAIController, BlocklistAIControllerEvent};
use crate::ai::execution_context::AiExecutionContext;
use crate::ai::predict::terminal_prompt_suggestions::{
    TerminalPromptSuggestion, TerminalPromptSuggestionsRequest, TerminalPromptSuggestionsResponse,
};
use crate::ai::terminal_suggestions::provider::{SuggestionProvider, TerminalSuggestionProvider};
use crate::http_api::AIApiError;
use crate::settings::AISettings;
use crate::terminal::event::{BlockType, UserBlockCompleted};
use crate::terminal::model::block::BlockId;
use crate::terminal::model::session::active_session::ActiveSession;
use crate::terminal::model::terminal_model::TerminalModel;
use crate::terminal::model_events::{ModelEvent, ModelEventDispatcher};
use crate::terminal::view::{AgentModePromptSuggestion, PromptSuggestion};
use chrono::Utc;
use parking_lot::FairMutex;
use serde_json::json;
use warpui::r#async::SpawnedFutureHandle;
use warpui::{Entity, ModelContext, ModelHandle, SingletonEntity};

const NUM_TOP_BLOCK_LINES: usize = 100;
const NUM_BOTTOM_BLOCK_LINES: usize = 200;

#[derive(Clone, Debug)]
pub enum PassiveSuggestionsEvent {
    PromptSuggestionsGenerated {
        prompt_suggestion: AgentModePromptSuggestion,
        block_id: BlockId,
        command: String,
        request_duration_ms: u64,
    },
}

pub struct PassiveSuggestionsModel {
    active_session: ModelHandle<ActiveSession>,
    terminal_model: Arc<FairMutex<TerminalModel>>,
    prompt_suggestions_future_handle: Option<SpawnedFutureHandle>,
}

impl PassiveSuggestionsModel {
    pub fn new(
        active_session: ModelHandle<ActiveSession>,
        terminal_model: Arc<FairMutex<TerminalModel>>,
        ai_controller: ModelHandle<BlocklistAIController>,
        model_event_dispatcher: &ModelHandle<ModelEventDispatcher>,
        ctx: &mut ModelContext<Self>,
    ) -> Self {
        ctx.subscribe_to_model(model_event_dispatcher, |me, event, ctx| {
            me.handle_model_event(event, ctx);
        });
        ctx.subscribe_to_model(&ai_controller, |me, event, _ctx| {
            me.handle_controller_event(event, _ctx);
        });

        Self {
            active_session,
            terminal_model,
            prompt_suggestions_future_handle: None,
        }
    }

    pub fn abort_pending_requests(&mut self) {
        if let Some(handle) = self.prompt_suggestions_future_handle.take() {
            handle.abort();
        }
    }

    fn handle_model_event(&mut self, event: &ModelEvent, ctx: &mut ModelContext<Self>) {
        match event {
            ModelEvent::AfterBlockStarted { .. } => {
                self.abort_pending_requests();
            }
            ModelEvent::AfterBlockCompleted(after_block_completed_event) => {
                let BlockType::User(block_completed) = &after_block_completed_event.block_type
                else {
                    return;
                };
                self.handle_user_block_completed(block_completed, ctx);
            }
            _ => {}
        }
    }

    fn handle_controller_event(
        &mut self,
        event: &BlocklistAIControllerEvent,
        _ctx: &mut ModelContext<Self>,
    ) {
        match event {
            BlocklistAIControllerEvent::SentRequest { stream_id, .. } => {
                let _ = stream_id;
                self.abort_pending_requests();
            }
            _ => {}
        }
    }

    fn handle_user_block_completed(
        &mut self,
        block_completed: &UserBlockCompleted,
        ctx: &mut ModelContext<Self>,
    ) {
        if block_completed.was_part_of_agent_interaction {
            return;
        }

        self.abort_pending_requests();

        if should_generate_prompt_suggestions(block_completed, ctx) {
            self.generate_prompt_suggestions(block_completed.clone(), ctx);
        }
    }

    fn generate_prompt_suggestions(
        &mut self,
        block_completed: UserBlockCompleted,
        ctx: &mut ModelContext<Self>,
    ) {
        let block_id = block_completed.serialized_block.id.clone();
        let command = block_completed.command.clone();
        let start_ts_ms = Utc::now().timestamp_millis();

        let Some(execution_context) = self
            .active_session
            .as_ref(ctx)
            .ai_execution_environment(ctx)
        else {
            return;
        };
        let Some(request) = build_prompt_suggestions_request(
            &block_completed,
            execution_context,
            &self.terminal_model,
        ) else {
            return;
        };

        let suggestions_config = AISettings::as_ref(ctx).terminal_suggestions_config();
        let provider = TerminalSuggestionProvider::default();
        let request_future = async move {
            let Some(config) = suggestions_config else {
                log::warn!(
                    "[terminal-suggestions] prompt suggestion skipped: endpoint or model is not configured"
                );
                return Err(AIApiError::Other(anyhow::anyhow!(
                    "Terminal suggestions endpoint and model must be configured"
                )));
            };

            log::debug!(
                "[terminal-suggestions] prompt suggestion request prepared context_messages={} exit_code={} model={}",
                request.context_messages.len(),
                request.exit_code,
                config.model,
            );
            provider.generate_prompt_suggestions(config, request).await
        };

        self.prompt_suggestions_future_handle =
            Some(ctx.spawn(request_future, move |me, result, ctx| {
                me.prompt_suggestions_future_handle = None;
                let end_ts_ms = Utc::now().timestamp_millis();
                let request_duration_ms = end_ts_ms.saturating_sub(start_ts_ms) as u64;
                let prompt_suggestion = match result {
                    Ok(response) => {
                        log::debug!(
                            "[terminal-suggestions] prompt suggestion response received duration_ms={} has_suggestion={}",
                            request_duration_ms,
                            response.suggestion.is_some(),
                        );
                        map_prompt_suggestions_response(response)
                    }
                    Err(err) => {
                        log::warn!("[terminal-suggestions] prompt suggestion request failed: {err}");
                        AgentModePromptSuggestion::Error
                    }
                };

                ctx.emit(PassiveSuggestionsEvent::PromptSuggestionsGenerated {
                    prompt_suggestion: prompt_suggestion.clone(),
                    block_id: block_id.clone(),
                    command,
                    request_duration_ms,
                });
            }));
    }
}

impl Entity for PassiveSuggestionsModel {
    type Event = PassiveSuggestionsEvent;
}

fn should_generate_prompt_suggestions(
    block_completed: &UserBlockCompleted,
    ctx: &ModelContext<PassiveSuggestionsModel>,
) -> bool {
    should_generate_terminal_prompt_suggestions(&block_completed.command, AISettings::as_ref(ctx))
}

fn should_generate_terminal_prompt_suggestions(command: &str, settings: &AISettings) -> bool {
    !command.trim().is_empty() && settings.is_terminal_prompt_suggestions_enabled()
}

fn build_prompt_suggestions_request(
    block: &UserBlockCompleted,
    execution_context: AiExecutionContext,
    terminal_model: &Arc<FairMutex<TerminalModel>>,
) -> Option<TerminalPromptSuggestionsRequest> {
    let exit_code = block.serialized_block.exit_code;
    let working_dir = block.serialized_block.pwd.as_ref();
    let (processed_input, processed_output) = {
        let model = terminal_model.lock();
        let terminal_width = model.block_list().size().columns();
        let Some(current_block) = model.block_list().block_with_id(&block.serialized_block.id)
        else {
            log::error!(
                "Failed to fetch prompt suggestions, could not find block with ID: {:?}",
                block.serialized_block.id
            );
            return None;
        };
        current_block.get_block_content_summary(
            terminal_width,
            NUM_TOP_BLOCK_LINES,
            NUM_BOTTOM_BLOCK_LINES,
        )
    };

    let json_message = json!({
        "command": processed_input,
        "output": processed_output,
        "exit_code": exit_code,
        "pwd": working_dir,
    });
    Some(TerminalPromptSuggestionsRequest {
        context_messages: vec![json_message.to_string()],
        system_context: execution_context.to_json_string(),
        exit_code: exit_code.value(),
    })
}

fn map_prompt_suggestions_response(
    response: TerminalPromptSuggestionsResponse,
) -> AgentModePromptSuggestion {
    let is_valid_code_delegation = response.is_valid_code_delegation();
    let Some(suggestion) = response.suggestion else {
        return AgentModePromptSuggestion::None;
    };

    match suggestion {
        TerminalPromptSuggestion::Coding(coding_query) if is_valid_code_delegation => {
            AgentModePromptSuggestion::Success(PromptSuggestion {
                id: response.id,
                label: None,
                prompt: coding_query.query,
                coding_query_context: Some(
                    coding_query
                        .files
                        .into_iter()
                        .map(Into::into)
                        .collect::<Vec<_>>(),
                ),
                should_start_new_conversation: true,
            })
        }
        TerminalPromptSuggestion::Simple(simple_query) => {
            AgentModePromptSuggestion::Success(PromptSuggestion {
                id: response.id,
                label: None,
                prompt: simple_query.query,
                coding_query_context: None,
                should_start_new_conversation: true,
            })
        }
        _ => AgentModePromptSuggestion::None,
    }
}

#[cfg(test)]
mod terminal_prompt_suggestion_tests {
    use super::*;
    use crate::test_util::settings::initialize_settings_for_tests;
    use settings::Setting;
    use warpui::App;

    #[test]
    fn test_terminal_prompt_suggestions_use_terminal_setting_only() {
        App::test((), |mut app| async move {
            initialize_settings_for_tests(&mut app);

            app.read(|ctx| {
                assert!(should_generate_terminal_prompt_suggestions(
                    "echo hello",
                    AISettings::as_ref(ctx),
                ));
                assert!(!should_generate_terminal_prompt_suggestions(
                    "   ",
                    AISettings::as_ref(ctx),
                ));
            });

            crate::settings::AISettings::handle(&app).update(&mut app, |settings, ctx| {
                settings
                    .terminal_prompt_suggestions_enabled
                    .set_value(false, ctx)
                    .unwrap();
            });

            app.read(|ctx| {
                assert!(!should_generate_terminal_prompt_suggestions(
                    "echo hello",
                    AISettings::as_ref(ctx),
                ));
            });
        });
    }
}
