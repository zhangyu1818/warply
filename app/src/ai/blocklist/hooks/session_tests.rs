use std::ffi::OsString;
use std::time::Duration;

use ai::hooks::PromptProvenance;
use ai::hooks::test_util::{ScriptedRunner, test_binding, test_source_id};
use ai::hooks::{DeclaredBinding, HookBoundary, HookPayload, HookRegistry};
use chrono::Local;
use instant::Instant;
use warpui::{App, SingletonEntity};

use super::*;
use crate::ai::agent::CancellationReason;
use crate::ai::blocklist::hooks::prompt;
use crate::ai::blocklist::hooks::test_utils::{hook_test_app, let_background_work_settle, settle};
use crate::ai::blocklist::{BlocklistAIHistoryModel, RequestInput, ResponseStreamId};
use crate::ai::llms::LLMId;
use crate::features::FeatureFlag;

fn session_binding() -> DeclaredBinding {
    test_binding(
        test_source_id("session"),
        HookBoundary::SessionStart,
        0,
        None,
    )
}

fn prompt_binding() -> DeclaredBinding {
    test_binding(
        test_source_id("prompt"),
        HookBoundary::PromptSubmit,
        0,
        None,
    )
}

fn session_start_count(harness: &crate::ai::blocklist::hooks::test_utils::HookTestApp) -> usize {
    harness
        .hooks
        .adapter
        .encoded_envelopes()
        .iter()
        .filter(|envelope| envelope.boundary() == HookBoundary::SessionStart)
        .count()
}

/// Drives `text` through `prompt::prepare_turn` and settles the wait, the way the
/// controller's `send_query` does.
async fn send_prompt_through_prepare(
    app: &mut App,
    harness: &crate::ai::blocklist::hooks::test_utils::HookTestApp,
    conversation_id: AIConversationId,
    text: &str,
) -> prompt::PromptDecision {
    let preparation = app.update(|ctx| {
        prompt::prepare_turn(
            Some(prompt::SubmittedPrompt {
                text: text.to_owned(),
                provenance: PromptProvenance::User,
            }),
            conversation_id,
            &harness.active_session,
            ctx,
        )
    });
    match preparation {
        prompt::TurnPreparation::Ready => panic!("expected the request to wait"),
        prompt::TurnPreparation::Pending(decision) => {
            let decision = decision.await;
            app.update(|ctx| prompt::finish_turn_preparation(conversation_id, &decision, ctx));
            decision
        }
    }
}

#[test]
fn session_start_fires_once_before_a_new_conversations_first_request() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        app.update(|ctx| harness.hooks.install_bindings(vec![session_binding()], ctx));

        // The first request of the conversation waits for session.start and then goes out.
        let conversation_id = harness.conversation_id;
        let preparation = app.update(|ctx| {
            prompt::prepare_turn(
                Some(prompt::SubmittedPrompt {
                    text: "hello".to_owned(),
                    provenance: PromptProvenance::User,
                }),
                conversation_id,
                &harness.active_session,
                ctx,
            )
        });
        let prompt::TurnPreparation::Pending(decision) = preparation else {
            panic!("the first request waits for the session hooks");
        };
        let decision = decision.await;
        app.update(|ctx| prompt::finish_turn_preparation(conversation_id, &decision, ctx));
        let_background_work_settle(&mut app).await;
        assert_eq!(session_start_count(&harness), 1);
        let envelope = &harness.hooks.adapter.encoded_envelopes()[0];
        assert_eq!(envelope.conversation_id, conversation_id);
        assert!(matches!(
            envelope.payload,
            HookPayload::SessionStart {
                is_background_agent: false
            }
        ));

        // The conversation's later prompts do not start another session.
        let preparation = app.update(|ctx| {
            prompt::prepare_turn(
                Some(prompt::SubmittedPrompt {
                    text: "again".to_owned(),
                    provenance: PromptProvenance::User,
                }),
                conversation_id,
                &harness.active_session,
                ctx,
            )
        });
        assert!(
            matches!(preparation, prompt::TurnPreparation::Ready),
            "a started session does not start again"
        );
        let_background_work_settle(&mut app).await;
        assert_eq!(session_start_count(&harness), 1);
    });
}

#[test]
fn the_first_request_waits_for_the_session_hooks_and_then_goes_out() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        app.update(|ctx| harness.hooks.install_bindings(vec![session_binding()], ctx));
        let hold = harness.hooks.runner.hold();

        let conversation_id = harness.conversation_id;
        let preparation = app.update(|ctx| {
            prompt::prepare_turn(
                Some(prompt::SubmittedPrompt {
                    text: "hello".to_owned(),
                    provenance: PromptProvenance::User,
                }),
                conversation_id,
                &harness.active_session,
                ctx,
            )
        });
        let prompt::TurnPreparation::Pending(decision) = preparation else {
            panic!("the request waits for the session hooks");
        };
        assert!(
            app.read(|ctx| prompt::is_pending(conversation_id, ctx)),
            "the request waits for the session hooks"
        );

        hold.release();
        assert!(matches!(decision.await, prompt::PromptDecision::Proceed(_)));
        let_background_work_settle(&mut app).await;
    });
}

#[test]
fn a_session_hook_that_fails_or_times_out_adds_nothing_and_does_not_block_further() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        let failing = session_binding();
        harness
            .hooks
            .runner
            .script(failing.id.clone(), ScriptedRunner::exit_with(1, ""));
        app.update(|ctx| {
            harness
                .hooks
                .install_bindings(vec![failing, prompt_binding()], ctx)
        });

        let conversation_id = harness.conversation_id;
        let decision =
            send_prompt_through_prepare(&mut app, &harness, conversation_id, "hello").await;
        assert!(matches!(decision, prompt::PromptDecision::Proceed(_)));
        let invocations = harness.hooks.runner.invocations();
        assert_eq!(invocations.len(), 2);
        assert!(
            !invocations[1]
                .env
                .iter()
                .any(|(name, _)| name == &OsString::from("HOOK_SESSION")),
            "a failed session.start sets nothing"
        );
        assert!(
            app.read(|ctx| HookRuntime::as_ref(ctx).session_env_for(conversation_id))
                .is_empty()
        );

        // A hook that never answers holds the first request only until its timeout.
        let mut slow = session_binding();
        slow.timeout = Duration::from_millis(100);
        harness.hooks.adapter.clear_encoded();
        app.update(|ctx| harness.hooks.install_bindings(vec![slow], ctx));
        let _hold = harness.hooks.runner.hold();
        let started = Instant::now();
        let slow_conversation = crate::ai::blocklist::hooks::test_utils::new_conversation(
            &mut app,
            harness.terminal_view_id,
        );
        let decision =
            send_prompt_through_prepare(&mut app, &harness, slow_conversation, "hello again").await;
        assert!(matches!(decision, prompt::PromptDecision::Proceed(_)));
        assert!(started.elapsed() >= Duration::from_millis(100));
        assert_eq!(session_start_count(&harness), 1);
    });
}

#[test]
fn the_session_environment_reaches_every_later_hook_in_the_conversation() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        let session = session_binding();
        harness.hooks.runner.script(
            session.id.clone(),
            ScriptedRunner::exit_with(0, r#"{"env": [["HOOK_SESSION", "yes"]]}"#),
        );
        app.update(|ctx| {
            harness
                .hooks
                .install_bindings(vec![session, prompt_binding()], ctx)
        });

        let conversation_id = harness.conversation_id;
        let decision =
            send_prompt_through_prepare(&mut app, &harness, conversation_id, "hello").await;
        assert!(matches!(decision, prompt::PromptDecision::Proceed(_)));
        let invocations = harness.hooks.runner.invocations();
        assert_eq!(invocations.len(), 2);
        assert_eq!(
            invocations[0].binding.event, "session.start",
            "the session hook runs first"
        );
        assert!(
            invocations[1]
                .env
                .contains(&(OsString::from("HOOK_SESSION"), OsString::from("yes"))),
            "{:?}",
            invocations[1].env
        );
        assert_eq!(
            app.read(|ctx| HookRuntime::as_ref(ctx).session_env_for(conversation_id)),
            vec![("HOOK_SESSION".to_owned(), "yes".to_owned())]
        );
        let other = AIConversationId::new();
        assert!(app.read(|ctx| HookRuntime::as_ref(ctx).session_env_for(other).is_empty()));
    });
}

#[test]
fn a_conversation_with_exchanges_is_not_a_new_session() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        app.update(|ctx| harness.hooks.install_bindings(vec![session_binding()], ctx));
        // A restored conversation arrives with its exchanges; the same is true of a resumed one.
        app.update(|ctx| {
            BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                let task_id = history
                    .conversation(&harness.conversation_id)
                    .unwrap()
                    .get_root_task_id()
                    .clone();
                history
                    .update_conversation_for_new_request_input_for_test(
                        RequestInput {
                            conversation_id: harness.conversation_id,
                            input_messages: Vec::from([(task_id, vec![])]),
                            working_directory: None,
                            model_id: LLMId::from("test-model"),
                            coding_model_id: LLMId::from("test-coding-model"),
                            cli_agent_model_id: LLMId::from("test-cli-agent-model"),
                            computer_use_model_id: LLMId::from("test-computer-use-model"),
                            request_start_ts: Local::now(),
                        },
                        ResponseStreamId::new_for_test(),
                        harness.terminal_view_id,
                        ctx,
                    )
                    .unwrap();
            });
        });

        let wait = app.update(|ctx| barrier(harness.conversation_id, &harness.active_session, ctx));
        assert!(wait.is_none());
        let_background_work_settle(&mut app).await;
        assert_eq!(harness.hooks.adapter.encoded_count(), 0);
        assert_eq!(harness.hooks.runner.invocation_count(), 0);
    });
}

/// Bindings can be in the registry before the loader finalizes it. A request in that window is
/// not held, and the conversation is not marked as started, so its first request after
/// finalization still fires `session.start`.
#[test]
fn a_request_before_finalization_does_not_use_up_the_session_start() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        app.update(|ctx| harness.hooks.stage_bindings(vec![session_binding()], ctx));

        let early =
            app.update(|ctx| barrier(harness.conversation_id, &harness.active_session, ctx));
        assert!(early.is_none());
        assert!(app.read(|ctx| {
            !HookRuntime::as_ref(ctx)
                .session_starts
                .states
                .contains_key(&harness.conversation_id)
        }));
        assert_eq!(harness.hooks.runner.invocation_count(), 0);

        app.update(|ctx| {
            HookRegistry::handle(ctx).update(ctx, |registry, _| {
                registry.finalize().expect("finalizes once");
            });
        });
        let after = app
            .update(|ctx| barrier(harness.conversation_id, &harness.active_session, ctx))
            .expect("the first request after finalization waits for session.start");
        after.await;
        assert_eq!(harness.hooks.runner.invocation_count(), 1);
    });
}

#[test]
fn a_second_request_while_the_hooks_run_waits_on_the_same_firing() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        app.update(|ctx| harness.hooks.install_bindings(vec![session_binding()], ctx));
        let hold = harness.hooks.runner.hold();

        let first = app
            .update(|ctx| barrier(harness.conversation_id, &harness.active_session, ctx))
            .expect("the first request waits");
        let second = app
            .update(|ctx| barrier(harness.conversation_id, &harness.active_session, ctx))
            .expect("the second request waits on the same hooks");
        settle(&mut app, |_| harness.hooks.runner.invocation_count() == 1).await;
        assert!(app.read(|ctx| {
            HookRuntime::as_ref(ctx)
                .session_starts
                .is_in_flight(harness.conversation_id)
        }));

        hold.release();
        first.await;
        second.await;
        settle(&mut app, |app| {
            !app.read(|ctx| {
                HookRuntime::as_ref(ctx)
                    .session_starts
                    .is_in_flight(harness.conversation_id)
            })
        })
        .await;
        assert_eq!(harness.hooks.runner.invocation_count(), 1);
        assert!(
            app.update(|ctx| barrier(harness.conversation_id, &harness.active_session, ctx,))
                .is_none(),
            "a conversation whose session started does not start it again"
        );
    });
}

#[test]
fn a_newer_prompt_replaces_one_waiting_only_on_the_session_barrier() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        // No prompt hook: the first prompt waits on nothing but the session hooks.
        app.update(|ctx| harness.hooks.install_bindings(vec![session_binding()], ctx));
        let hold = harness.hooks.runner.hold();

        let conversation_id = harness.conversation_id;
        let first = app.update(|ctx| {
            prompt::prepare_turn(
                Some(prompt::SubmittedPrompt {
                    text: "first".to_owned(),
                    provenance: PromptProvenance::User,
                }),
                conversation_id,
                &harness.active_session,
                ctx,
            )
        });
        let prompt::TurnPreparation::Pending(first_decision) = first else {
            panic!("the first prompt waits on the session barrier");
        };
        settle(&mut app, |_| harness.hooks.runner.invocation_count() == 1).await;
        assert!(app.read(|ctx| prompt::is_pending(conversation_id, ctx)));
        let second = app.update(|ctx| {
            prompt::prepare_turn(
                Some(prompt::SubmittedPrompt {
                    text: "second".to_owned(),
                    provenance: PromptProvenance::User,
                }),
                conversation_id,
                &harness.active_session,
                ctx,
            )
        });
        let prompt::TurnPreparation::Pending(second_decision) = second else {
            panic!("the second prompt waits on the same barrier");
        };

        hold.release();
        // The replaced prompt does not go out; the newer one does.
        let first_outcome = first_decision.await;
        assert!(matches!(first_outcome, prompt::PromptDecision::Cancelled));
        app.update(|ctx| prompt::finish_turn_preparation(conversation_id, &first_outcome, ctx));
        let second_outcome = second_decision.await;
        assert!(matches!(second_outcome, prompt::PromptDecision::Proceed(_)));
        app.update(|ctx| prompt::finish_turn_preparation(conversation_id, &second_outcome, ctx));
        let_background_work_settle(&mut app).await;
        assert!(!app.read(|ctx| prompt::is_pending(conversation_id, ctx)));
    });
}

#[test]
fn cancelling_during_the_session_wait_drops_the_prompt() {
    App::test((), |mut app| async move {
        let _flag = FeatureFlag::AgentHooks.override_enabled(true);
        let harness = hook_test_app(&mut app);
        app.update(|ctx| harness.hooks.install_bindings(vec![session_binding()], ctx));
        let hold = harness.hooks.runner.hold();

        let conversation_id = harness.conversation_id;
        let preparation = app.update(|ctx| {
            prompt::prepare_turn(
                Some(prompt::SubmittedPrompt {
                    text: "hello".to_owned(),
                    provenance: PromptProvenance::User,
                }),
                conversation_id,
                &harness.active_session,
                ctx,
            )
        });
        let prompt::TurnPreparation::Pending(decision) = preparation else {
            panic!("the prompt waits on the session barrier");
        };
        settle(&mut app, |_| harness.hooks.runner.invocation_count() == 1).await;
        app.update(|ctx| prompt::cancel_pending(conversation_id, ctx));
        assert!(!app.read(|ctx| prompt::is_pending(conversation_id, ctx)));

        hold.release();
        assert!(matches!(decision.await, prompt::PromptDecision::Cancelled));
        let_background_work_settle(&mut app).await;
    });
}

#[test]
fn inactive_hooks_do_not_fire_session_start() {
    App::test((), |mut app| async move {
        // The fork has no shared-session or CLI-transcript conversations, so the upstream
        // viewer/transcript negative case cannot exist here; the flag stays off, so nothing
        // fires for any conversation.
        let harness = hook_test_app(&mut app);
        let conversation_id = harness.conversation_id;
        {
            let wait = app.update(|ctx| barrier(conversation_id, &harness.active_session, ctx));
            assert!(wait.is_none());
        }
        let_background_work_settle(&mut app).await;
        assert_eq!(harness.hooks.adapter.encoded_count(), 0);
    });
}
