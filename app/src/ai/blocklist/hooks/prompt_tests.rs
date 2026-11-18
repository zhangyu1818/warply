use ai::hooks::test_util::{ScriptedRunner, test_binding, test_source_id};
use ai::hooks::{DeclaredBinding, HookBoundary, HookNoticeKind, HookPayload, PromptProvenance};
use warpui::{App, SingletonEntity};

use super::*;
use crate::ai::agent::CancellationReason;
use crate::ai::blocklist::hooks::test_utils::{
    HookTestApp, hook_test_app, let_background_work_settle, settle,
};
use crate::features::FeatureFlag;

fn prompt_binding() -> DeclaredBinding {
    test_binding(
        test_source_id("prompt"),
        HookBoundary::PromptSubmit,
        0,
        None,
    )
}

fn user_prompt(text: &str) -> SubmittedPrompt {
    SubmittedPrompt {
        text: text.to_owned(),
        provenance: PromptProvenance::User,
    }
}

/// Drives `text` through `prepare_turn` and settles the wait, the way the controller's
/// `send_query` does.
async fn send_prompt_through_prepare(
    app: &mut App,
    harness: &HookTestApp,
    conversation_id: AIConversationId,
    text: &str,
) -> PromptDecision {
    let preparation = app.update(|ctx| {
        prepare_turn(
            Some(user_prompt(text)),
            conversation_id,
            &harness.active_session,
            ctx,
        )
    });
    match preparation {
        TurnPreparation::Ready => panic!("expected the prompt to wait"),
        TurnPreparation::Pending(decision) => {
            let decision = decision.await;
            app.update(|ctx| finish_turn_preparation(conversation_id, &decision, ctx));
            decision
        }
    }
}

#[test]
fn prompt_submit_reaches_the_adapter_with_the_text_and_conversation() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        app.update(|ctx| harness.hooks.install_bindings(vec![prompt_binding()], ctx));

        let preparation = app.update(|ctx| {
            prepare_turn(
                Some(user_prompt("fix the build")),
                harness.conversation_id,
                &harness.active_session,
                ctx,
            )
        });
        let TurnPreparation::Pending(decision) = preparation else {
            panic!("a prompt hook applies, so the send waits for it");
        };
        let decision = decision.await;
        assert!(matches!(decision, PromptDecision::Proceed(_)));
        settle(&mut app, |_| harness.hooks.runner.invocation_count() == 1).await;

        let envelope = &harness.hooks.adapter.encoded_envelopes()[0];
        assert_eq!(envelope.boundary(), HookBoundary::PromptSubmit);
        assert_eq!(envelope.conversation_id, harness.conversation_id);
        let HookPayload::PromptSubmit {
            text,
            attachments_summary,
            provenance,
        } = &envelope.payload
        else {
            panic!("expected a prompt.submit payload");
        };
        assert_eq!(text, "fix the build");
        assert!(attachments_summary.is_empty());
        assert_eq!(*provenance, PromptProvenance::User);
        assert_eq!(
            envelope.turn_id,
            app.read(|ctx| HookRuntime::as_ref(ctx).turn_id_for(harness.conversation_id)),
            "the prompt's envelope carries the turn it starts"
        );
        assert!(envelope.turn_id.is_some());
    });
}

#[test]
fn prompt_submit_does_not_fire_for_an_unknown_conversation() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        app.update(|ctx| harness.hooks.install_bindings(vec![prompt_binding()], ctx));
        // The fork has no shared-session viewers, so the upstream mirrored-conversation
        // negative case cannot exist here; an unknown conversation is the negative.
        {
            let preparation = app.update(|ctx| {
                prepare_turn(
                    Some(user_prompt("unknown")),
                    AIConversationId::new(),
                    &harness.active_session,
                    ctx,
                )
            });
            assert!(matches!(preparation, TurnPreparation::Ready));
        }
        let_background_work_settle(&mut app).await;
        assert_eq!(harness.hooks.adapter.encoded_count(), 0);
    });
}

#[test]
fn every_prompt_starts_a_new_turn_and_the_turns_envelopes_carry_its_id() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        // Only a tool binding is active, so a prompt starts a turn without waiting on anything.
        app.update(|ctx| {
            harness.hooks.install_bindings(
                vec![test_binding(
                    test_source_id("s"),
                    HookBoundary::ToolPre,
                    0,
                    None,
                )],
                ctx,
            )
        });
        let mut turn_ids = Vec::new();
        for prompt in [
            Some(user_prompt("first")),
            Some(user_prompt("second")),
            None,
        ] {
            let preparation = app.update(|ctx| {
                prepare_turn(
                    prompt,
                    harness.conversation_id,
                    &harness.active_session,
                    ctx,
                )
            });
            assert!(matches!(preparation, TurnPreparation::Ready));
            turn_ids.push(
                app.read(|ctx| HookRuntime::as_ref(ctx).turn_id_for(harness.conversation_id))
                    .expect("a turn started"),
            );
        }
        assert_ne!(turn_ids[0], turn_ids[1]);
        assert_ne!(turn_ids[1], turn_ids[2]);
        assert_eq!(
            app.read(|ctx| HookRuntime::as_ref(ctx).turn_id_for(AIConversationId::new())),
            None
        );
    });
}

#[test]
fn a_halting_hook_stops_the_prompt_and_tells_the_person_why() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        let binding = prompt_binding();
        harness.hooks.runner.script(
            binding.id.clone(),
            ScriptedRunner::exit_with(0, r#"{"halt": true, "reason": "not before lunch"}"#),
        );
        app.update(|ctx| harness.hooks.install_bindings(vec![binding], ctx));

        let conversation_id = harness.conversation_id;
        let decision =
            send_prompt_through_prepare(&mut app, &harness, conversation_id, "deploy everything")
                .await;
        assert!(matches!(decision, PromptDecision::Halted(_)));
        let_background_work_settle(&mut app).await;

        let notices = harness.hooks.messenger.notices();
        assert_eq!(notices.len(), 1);
        assert_eq!(notices[0].kind, HookNoticeKind::Denied);
        assert_eq!(notices[0].text, "not before lunch");
        assert_eq!(notices[0].conversation_id, conversation_id);
        assert!(!app.read(|ctx| is_pending(conversation_id, ctx)));
    });
}

#[test]
fn a_denying_or_asking_hook_halts_the_prompt_too() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        let binding = prompt_binding();
        app.update(|ctx| harness.hooks.install_bindings(vec![binding.clone()], ctx));

        harness
            .hooks
            .runner
            .script(binding.id.clone(), ScriptedRunner::deny_with("denied it"));
        let denied = harness.conversation_id;
        let decision = send_prompt_through_prepare(&mut app, &harness, denied, "one").await;
        assert!(matches!(decision, PromptDecision::Halted(_)));
        settle(&mut app, |_| harness.hooks.messenger.notices().len() == 1).await;

        harness.hooks.runner.script(
            binding.id.clone(),
            ScriptedRunner::exit_with(0, r#"{"verdict": "ask", "reason": "are you sure"}"#),
        );
        let decision = send_prompt_through_prepare(&mut app, &harness, denied, "two").await;
        assert!(matches!(decision, PromptDecision::Halted(_)));
        settle(&mut app, |_| harness.hooks.messenger.notices().len() == 2).await;
        let_background_work_settle(&mut app).await;

        let notices = harness.hooks.messenger.notices();
        assert_eq!(notices[0].text, "denied it");
        assert!(notices[1].text.starts_with(ASK_UNAVAILABLE_HALT_REASON));
        assert!(notices[1].text.contains("are you sure"));
    });
}

#[test]
fn a_hook_with_no_opinion_lets_the_prompt_through_after_it_answers() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        let binding = prompt_binding();
        app.update(|ctx| harness.hooks.install_bindings(vec![binding], ctx));
        let hold = harness.hooks.runner.hold();

        let conversation_id = harness.conversation_id;
        let preparation = app.update(|ctx| {
            prepare_turn(
                Some(user_prompt("go ahead")),
                conversation_id,
                &harness.active_session,
                ctx,
            )
        });
        let TurnPreparation::Pending(decision) = preparation else {
            panic!("the request waits for the hook");
        };
        assert!(app.read(|ctx| is_pending(conversation_id, ctx)));

        hold.release();
        let decision = decision.await;
        assert!(matches!(decision, PromptDecision::Proceed(_)));
        app.update(|ctx| finish_turn_preparation(conversation_id, &decision, ctx));
        let_background_work_settle(&mut app).await;
        assert!(!app.read(|ctx| is_pending(conversation_id, ctx)));
        assert!(harness.hooks.messenger.notices().is_empty());
    });
}

#[test]
fn a_newer_prompt_for_the_same_conversation_replaces_one_still_waiting() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        let binding = prompt_binding();
        app.update(|ctx| harness.hooks.install_bindings(vec![binding], ctx));
        let hold = harness.hooks.runner.hold();

        let conversation_id = harness.conversation_id;
        let first = app.update(|ctx| {
            prepare_turn(
                Some(user_prompt("first")),
                conversation_id,
                &harness.active_session,
                ctx,
            )
        });
        let TurnPreparation::Pending(first_decision) = first else {
            panic!("the first prompt waits");
        };
        let second = app.update(|ctx| {
            prepare_turn(
                Some(user_prompt("second")),
                conversation_id,
                &harness.active_session,
                ctx,
            )
        });
        let TurnPreparation::Pending(second_decision) = second else {
            panic!("the second prompt waits");
        };

        hold.release();
        let first_outcome = first_decision.await;
        assert!(matches!(first_outcome, PromptDecision::Cancelled));
        app.update(|ctx| finish_turn_preparation(conversation_id, &first_outcome, ctx));
        let second_outcome = second_decision.await;
        assert!(matches!(second_outcome, PromptDecision::Proceed(_)));
        app.update(|ctx| finish_turn_preparation(conversation_id, &second_outcome, ctx));
        let_background_work_settle(&mut app).await;
        assert!(!app.read(|ctx| is_pending(conversation_id, ctx)));
    });
}

#[test]
fn cancelling_the_conversation_drops_a_waiting_prompt() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        let binding = prompt_binding();
        app.update(|ctx| harness.hooks.install_bindings(vec![binding], ctx));
        let hold = harness.hooks.runner.hold();

        let conversation_id = harness.conversation_id;
        let preparation = app.update(|ctx| {
            prepare_turn(
                Some(user_prompt("first")),
                conversation_id,
                &harness.active_session,
                ctx,
            )
        });
        let TurnPreparation::Pending(decision) = preparation else {
            panic!("the prompt waits");
        };
        app.update(|ctx| cancel_pending(conversation_id, ctx));
        assert!(!app.read(|ctx| is_pending(conversation_id, ctx)));

        hold.release();
        assert!(matches!(decision.await, PromptDecision::Cancelled));
        let_background_work_settle(&mut app).await;
    });
}

#[test]
fn with_hooks_off_a_prompt_is_sent_on_the_same_call() {
    App::test((), |mut app| async move {
        let harness = hook_test_app(&mut app);
        app.update(|ctx| harness.hooks.install_bindings(vec![prompt_binding()], ctx));
        let conversation_id = harness.conversation_id;
        let preparation = app.update(|ctx| {
            prepare_turn(
                Some(user_prompt("right away")),
                conversation_id,
                &harness.active_session,
                ctx,
            )
        });
        assert!(matches!(preparation, TurnPreparation::Ready));
        assert_eq!(harness.hooks.adapter.encoded_count(), 0);
        assert_eq!(
            app.read(|ctx| HookRuntime::as_ref(ctx).turn_id_for(conversation_id)),
            None
        );
    });
}
