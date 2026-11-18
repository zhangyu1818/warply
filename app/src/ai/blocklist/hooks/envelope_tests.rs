use ai::agent::action_result::RequestCommandOutputResult;
use ai::hooks::{HOOK_PROTOCOL_VERSION, HookBoundary, HookPayload, HookSurface, RunMetadata};
use warp_core::execution_mode::ExecutionMode;
use warpui::App;

use super::*;
use crate::ai::agent::CancellationReason;
use crate::ai::blocklist::hooks::RunIdentity;
use crate::ai::blocklist::hooks::test_utils::{
    hook_test_app, install_test_hooks, mock_active_session, shell_action,
};
use crate::features::FeatureFlag;
use crate::test_util::settings::{
    initialize_settings_for_tests_with_mode,
    initialize_settings_for_tests_with_mode_and_headlessness,
};

#[test]
fn tool_pre_envelope_carries_the_action_and_common_fields() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        let action = shell_action("a1", "ls -la");
        let envelope = app.read(|ctx| {
            let site = EnvelopeSite::new(harness.conversation_id, &harness.active_session);
            envelope_for_tool_pre(&action, &site, ctx)
        });

        assert_eq!(envelope.boundary(), HookBoundary::ToolPre);
        assert_eq!(envelope.conversation_id, harness.conversation_id);
        assert_eq!(envelope.action_id, Some(action.id.clone()));
        assert_eq!(envelope.protocol_version, HOOK_PROTOCOL_VERSION);
        assert_eq!(envelope.surface, HookSurface::Gui);
        assert!(!envelope.warp_version.is_empty());
        assert_eq!(envelope.turn_id, None);
        assert_eq!(envelope.exchange_id, None, "no exchange has been sent yet");
        assert_eq!(
            envelope.cwd, None,
            "the mock session has no working directory"
        );
        assert!(envelope.roots.is_empty());
        let HookPayload::ToolPre {
            action: payload_action,
        } = &envelope.payload
        else {
            panic!("expected a tool.pre payload");
        };
        assert_eq!(payload_action, &action.action);
    });
}

#[test]
fn an_interactive_sdk_launch_uses_the_gui_surface() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        initialize_settings_for_tests_with_mode_and_headlessness(
            &mut app,
            ExecutionMode::Headless,
            false,
        );
        app.update(install_test_hooks);
        let active_session = mock_active_session(&mut app);
        let envelope = app.read(|ctx| {
            envelope_for_session_start(
                true,
                &EnvelopeSite::new(AIConversationId::new(), &active_session),
                ctx,
            )
        });

        assert_eq!(envelope.surface, HookSurface::Gui);
    });
}

#[test]
fn result_and_failure_envelopes_carry_the_result_and_cancellation_reason() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        let action = shell_action("a1", "ls");
        let result = AIAgentActionResultType::RequestCommandOutput(
            RequestCommandOutputResult::CancelledBeforeExecution,
        );
        let (result_envelope, failure_envelope) = app.read(|ctx| {
            let site = EnvelopeSite::new(harness.conversation_id, &harness.active_session);
            (
                envelope_for_tool_result(&action, &result, &site, ctx),
                envelope_for_tool_failure(
                    &action,
                    &result,
                    Some(CancellationReason::ManuallyCancelled),
                    &site,
                    ctx,
                ),
            )
        });
        assert_eq!(result_envelope.boundary(), HookBoundary::ToolResult);
        assert_eq!(failure_envelope.boundary(), HookBoundary::ToolFailure);
        assert_eq!(failure_envelope.action_id, Some(action.id.clone()));
        let HookPayload::ToolFailure {
            cancellation_reason,
            ..
        } = &failure_envelope.payload
        else {
            panic!("expected a tool.failure payload");
        };
        assert_eq!(cancellation_reason.as_deref(), Some("user_cancelled"));
    });
}

#[test]
fn cancellation_reason_names_are_stable() {
    // Scripts match on these strings. Changing one is a breaking change for hook authors.
    let expected = [
        (CancellationReason::ManuallyCancelled, "user_cancelled"),
        (
            CancellationReason::FollowUpSubmitted {
                is_for_same_conversation: true,
            },
            "follow_up_submitted",
        ),
        (
            CancellationReason::FollowUpSubmitted {
                is_for_same_conversation: false,
            },
            "follow_up_submitted",
        ),
        (CancellationReason::UserCommandExecuted, "user_ran_command"),
        (CancellationReason::Reverted, "reverted"),
        (CancellationReason::Deleted, "conversation_deleted"),
        (
            CancellationReason::OptimisticCLISubagentCompletion,
            "command_finished",
        ),
        (CancellationReason::HookEndedRun, "hook_ended_run"),
    ];
    for (reason, name) in expected {
        assert_eq!(cancellation_reason_name(reason), name, "{reason:?}");
    }
}

#[test]
fn envelopes_carry_the_conversations_turn_id_once_a_turn_started() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        let action = shell_action("a1", "ls");
        let before = app.read(|ctx| {
            let site = EnvelopeSite::new(harness.conversation_id, &harness.active_session);
            envelope_for_session_start(false, &site, ctx)
        });
        assert_eq!(before.boundary(), HookBoundary::SessionStart);
        assert_eq!(before.turn_id, None);
        assert_eq!(before.exchange_id, None);

        let turn_id = app.update(|ctx| {
            HookRuntime::handle(ctx).update(ctx, |runtime, _| {
                runtime.start_turn(harness.conversation_id)
            })
        });
        let (same_conversation, other_conversation) = app.read(|ctx| {
            let site = EnvelopeSite::new(harness.conversation_id, &harness.active_session);
            let other = EnvelopeSite::new(AIConversationId::new(), &harness.active_session);
            (
                envelope_for_tool_pre(&action, &site, ctx).turn_id,
                envelope_for_tool_pre(&action, &other, ctx).turn_id,
            )
        });
        assert_eq!(same_conversation, Some(turn_id));
        assert_eq!(other_conversation, None);
    });
}

#[test]
fn run_metadata_reaches_the_envelope_without_a_cloud_surface() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        initialize_settings_for_tests_with_mode(&mut app, ExecutionMode::Headless);
        app.update(install_test_hooks);
        let active_session = mock_active_session(&mut app);
        let conversation_id = AIConversationId::new();

        let before = app.read(|ctx| {
            envelope_for_session_start(
                true,
                &EnvelopeSite::new(conversation_id, &active_session),
                ctx,
            )
        });
        assert_eq!(before.run, None);
        assert_eq!(before.surface, HookSurface::CliRun);

        let run = RunMetadata {
            run_id: "run-1".to_owned(),
            parent_run_id: None,
            task_id: Some("task-1".to_owned()),
        };
        // The fork has no server-dispatched runs, so a recorded run is a command-line one.
        for (surface, expected) in [(RunSurface::CommandLine, HookSurface::CliRun)] {
            app.update(|ctx| {
                HookRuntime::handle(ctx).update(ctx, |runtime, _| {
                    runtime.set_run(Some(RunIdentity {
                        metadata: run.clone(),
                        surface,
                    }));
                });
            });
            let after = app.read(|ctx| {
                envelope_for_session_start(
                    true,
                    &EnvelopeSite::new(conversation_id, &active_session),
                    ctx,
                )
            });
            assert_eq!(after.run.as_ref(), Some(&run));
            assert_eq!(after.surface, expected);
        }
    });
}
