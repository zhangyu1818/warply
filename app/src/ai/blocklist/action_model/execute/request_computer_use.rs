use ai::agent::action_result::{AIAgentActionResultType, RequestComputerUseResult};
use warpui::{Entity, EntityId, ModelContext, SingletonEntity};

use crate::ai::agent::AIAgentActionType;

use super::{ActionExecution, AnyActionExecution, ExecuteActionInput};

pub struct RequestComputerUseExecutor {
    terminal_view_id: EntityId,
}

impl RequestComputerUseExecutor {
    pub fn new(terminal_view_id: EntityId) -> Self {
        Self { terminal_view_id }
    }

    pub(super) fn should_autoexecute(
        &mut self,
        input: ExecuteActionInput,
        ctx: &mut ModelContext<Self>,
    ) -> bool {
        let ExecuteActionInput { action, .. } = input;
        let AIAgentActionType::RequestComputerUse(_) = &action.action else {
            return false;
        };

        // Check profile permission
        let permission = crate::ai::blocklist::BlocklistAIPermissions::as_ref(ctx)
            .get_computer_use_setting(ctx, Some(self.terminal_view_id));
        if permission.is_always_allow() {
            return true;
        }

        // Otherwise require user confirmation for computer use.
        false
    }

    pub(super) fn execute(
        &mut self,
        input: ExecuteActionInput,
        _ctx: &mut ModelContext<Self>,
    ) -> impl Into<AnyActionExecution> + use<> {
        let ExecuteActionInput {
            action,
            conversation_id: _,
        } = input;
        let AIAgentActionType::RequestComputerUse(request) = &action.action else {
            return ActionExecution::InvalidAction;
        };

        // If we're executing, that implies that computer use has been approved.
        let screenshot_params = request.screenshot_params;
        let mut actor = computer_use::create_actor();
        let platform = actor.platform();
        ActionExecution::Async {
            execute_future: Box::pin(async move {
                let result = actor
                    .perform_actions(&[], computer_use::Options { screenshot_params })
                    .await;
                (result, platform)
            }),
            on_complete: Box::new(|action_result, _ctx| match action_result {
                (
                    Ok(computer_use::ActionResult {
                        screenshot: Some(screenshot),
                        ..
                    }),
                    Some(platform),
                ) => AIAgentActionResultType::RequestComputerUse(
                    RequestComputerUseResult::Approved {
                        screenshot,
                        platform,
                    },
                ),
                (
                    Ok(computer_use::ActionResult {
                        screenshot: Some(_),
                        ..
                    }),
                    None,
                ) => AIAgentActionResultType::RequestComputerUse(RequestComputerUseResult::Error(
                    "Unknown platform".to_string(),
                )),
                (Ok(_), _) => {
                    AIAgentActionResultType::RequestComputerUse(RequestComputerUseResult::Error(
                        "Failed to capture initial screenshot".to_string(),
                    ))
                }
                (Err(err), _) => AIAgentActionResultType::RequestComputerUse(
                    RequestComputerUseResult::Error(err),
                ),
            }),
        }
    }
}

impl Entity for RequestComputerUseExecutor {
    type Event = ();
}
