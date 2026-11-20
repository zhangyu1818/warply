use std::collections::HashSet;
use std::time::Duration;

use ai::hooks::test_util::{ScriptedRunner, test_binding, test_source_id};
use ai::hooks::{DeclaredBinding, HookBoundary, HookPayload, HookTurnId, PromptProvenance};
use chrono::Local;
use instant::Instant;
use warpui::{App, SingletonEntity};

use super::*;
use crate::ai::agent::{
    AIAgentExchange, AIAgentExchangeId, AIAgentInput, AIAgentOutputStatus, UserQueryMode,
};
use crate::ai::blocklist::hooks::test_utils::{HookTestApp, hook_test_app};
use crate::ai::blocklist::{
    BlocklistAIHistoryModel, QueuedQuery, QueuedQueryModel, QueuedQueryOrigin, ResponseStreamId,
};
use crate::ai::llms::LLMId;

fn stop_binding(loop_limit: Option<u32>) -> DeclaredBinding {
    let mut binding = test_binding(test_source_id("stop"), HookBoundary::TurnStop, 0, None);
    binding.loop_limit = loop_limit;
    binding
}

/// The scripted output that asks for a follow-up.
fn continuation(text: &str) -> ai::hooks::HookProcessOutcome {
    ScriptedRunner::exit_with(0, &format!(r#"{{"continuation": "{text}"}}"#))
}

/// Runs one held stop round by hand: hold, start, wait, resolve.
async fn stop_round(app: &mut App, harness: &HookTestApp) -> TurnStopResolution {
    let held = app.update(|ctx| hold_turn(harness.conversation_id, &harness.active_session, ctx));
    assert!(held, "a stop binding applies, so the turn is held");
    let stop = app
        .update(|ctx| start_turn_stop(harness.conversation_id, ctx))
        .expect("the held turn's hooks start");
    let outcome = stop.await;
    app.update(|ctx| resolve_turn_stop(harness.conversation_id, outcome, ctx))
}

fn queued_texts(app: &App, conversation_id: AIConversationId) -> Vec<(String, PromptProvenance)> {
    app.read(|ctx| {
        let queue = QueuedQueryModel::as_ref(ctx);
        queue
            .queue(conversation_id)
            .iter()
            .map(|row| {
                (
                    row.text().to_owned(),
                    queue.provenance_for(conversation_id, row.id()),
                )
            })
            .collect()
    })
}

/// The text and mode of the user query in the conversation's latest exchange.
fn latest_user_query(
    app: &App,
    conversation_id: AIConversationId,
) -> Option<(String, UserQueryMode)> {
    BlocklistAIHistoryModel::handle(app).read(app, |history, _| {
        history
            .conversation(&conversation_id)?
            .latest_exchange()?
            .input
            .iter()
            .find_map(|input| {
                if let AIAgentInput::UserQuery {
                    query,
                    user_query_mode,
                    ..
                } = input
                {
                    Some((query.clone(), *user_query_mode))
                } else {
                    None
                }
            })
    })
}

/// Starts a turn by hand for a conversation whose first request was mocked.
fn start_turn(app: &mut App, conversation_id: AIConversationId) -> HookTurnId {
    app.update(|ctx| {
        HookRuntime::handle(ctx).update(ctx, |runtime, _| runtime.start_turn(conversation_id))
    })
}

/// How many stops of ended turns are waiting to start or running for `conversation_id`.
fn ended_stop_count(app: &App, conversation_id: AIConversationId) -> usize {
    app.read(|ctx| {
        HookRuntime::as_ref(ctx)
            .turn_stops
            .ended_stop_count(conversation_id)
    })
}

/// A streaming exchange, the fixture shape the history model's streamed requests append.
fn streaming_exchange() -> AIAgentExchange {
    AIAgentExchange {
        id: AIAgentExchangeId::new(),
        input: vec![],
        output_status: AIAgentOutputStatus::Streaming { output: None },
        added_message_ids: HashSet::new(),
        start_time: Local::now(),
        finish_time: None,
        working_directory: None,
        model_id: LLMId::from("test-model"),
        coding_model_id: LLMId::from("test-coding-model"),
        cli_agent_model_id: LLMId::from("test-cli-agent-model"),
        computer_use_model_id: LLMId::from("test-computer-use-model"),
    }
}

#[test]
fn a_follow_up_is_queued_behind_a_persons_prompt_with_the_hooks_provenance() {
    App::test((), |mut app| async move {
        let harness = hook_test_app(&mut app);
        let binding = stop_binding(None);
        harness
            .hooks
            .runner
            .script(binding.id.clone(), continuation("keep going"));
        app.update(|ctx| harness.hooks.install_bindings(vec![binding.clone()], ctx));
        app.update(|ctx| {
            QueuedQueryModel::handle(ctx).update(ctx, |queue, ctx| {
                queue.append(
                    harness.conversation_id,
                    QueuedQuery::new(
                        "the person's".to_owned(),
                        QueuedQueryOrigin::AutoQueueToggle,
                    ),
                    ctx,
                );
            });
        });

        let resolution = stop_round(&mut app, &harness).await;
        assert_eq!(resolution, TurnStopResolution::FollowUpQueued);

        let queued = queued_texts(&app, harness.conversation_id);
        assert_eq!(queued.len(), 2);
        assert_eq!(queued[0].0, "the person's");
        assert_eq!(queued[0].1, PromptProvenance::User);
        assert_eq!(queued[1].0, "keep going");
        let PromptProvenance::HookFollowUp {
            binding: from,
            chain_id,
        } = &queued[1].1
        else {
            panic!("the follow-up names the hook that asked for it");
        };
        assert_eq!(*from, binding.id);
        let envelope = &harness.hooks.adapter.encoded_envelopes()[0];
        assert!(matches!(
            envelope.payload,
            HookPayload::TurnStop {
                status: TurnStopStatus::Completed,
                continuation_count: 0
            }
        ));

        // The next stop in the same chain counts the first follow-up.
        let resolution = stop_round(&mut app, &harness).await;
        assert_eq!(resolution, TurnStopResolution::FollowUpQueued);
        let envelope = &harness.hooks.adapter.encoded_envelopes()[1];
        assert!(matches!(
            envelope.payload,
            HookPayload::TurnStop {
                continuation_count: 1,
                ..
            }
        ));
        let queued = queued_texts(&app, harness.conversation_id);
        assert!(matches!(
            &queued[2].1,
            PromptProvenance::HookFollowUp { chain_id: later, .. } if later == chain_id
        ));
    });
}

#[test]
fn a_bindings_loop_limit_ends_its_follow_ups_with_a_diagnostic() {
    App::test((), |mut app| async move {
        let harness = hook_test_app(&mut app);
        let binding = stop_binding(Some(2));
        harness
            .hooks
            .runner
            .script(binding.id.clone(), continuation("again"));
        app.update(|ctx| harness.hooks.install_bindings(vec![binding], ctx));

        assert_eq!(
            stop_round(&mut app, &harness).await,
            TurnStopResolution::FollowUpQueued
        );
        assert_eq!(
            stop_round(&mut app, &harness).await,
            TurnStopResolution::FollowUpQueued
        );
        assert_eq!(
            stop_round(&mut app, &harness).await,
            TurnStopResolution::Finalize,
            "the third follow-up is over the definition's loop_limit"
        );
        assert_eq!(queued_texts(&app, harness.conversation_id).len(), 2);
        let diagnostics = harness.hooks.messenger.diagnostics();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("loop_limit of 2")),
            "{diagnostics:?}"
        );

        // A person's prompt starts a new chain, so the same binding may continue again.
        app.update(|ctx| {
            HookRuntime::handle(ctx).update(ctx, |runtime, _| {
                runtime.turn_stops.start_chain(harness.conversation_id);
            });
        });
        assert_eq!(
            stop_round(&mut app, &harness).await,
            TurnStopResolution::FollowUpQueued
        );
    });
}

#[test]
fn the_host_cap_applies_to_the_whole_chain_across_bindings() {
    App::test((), |mut app| async move {
        let harness = hook_test_app(&mut app);
        // Two definitions with no limit of their own. The dispatcher keeps the first binding's
        // continuation, so the two are alternated by turning one off at a time.
        let first = stop_binding(None);
        let mut second = test_binding(test_source_id("stop"), HookBoundary::TurnStop, 1, None);
        second.loop_limit = None;
        harness
            .hooks
            .runner
            .script(first.id.clone(), continuation("from the first"));
        harness
            .hooks
            .runner
            .script(second.id.clone(), continuation("from the second"));

        for round in 0..HOOK_FOLLOW_UP_CAP {
            let active = if round % 2 == 0 {
                first.clone()
            } else {
                second.clone()
            };
            app.update(|ctx| harness.hooks.install_bindings(vec![active], ctx));
            assert_eq!(
                stop_round(&mut app, &harness).await,
                TurnStopResolution::FollowUpQueued,
                "follow-up {round} is within the cap"
            );
        }
        app.update(|ctx| harness.hooks.install_bindings(vec![first], ctx));
        assert_eq!(
            stop_round(&mut app, &harness).await,
            TurnStopResolution::Finalize
        );
        assert_eq!(
            queued_texts(&app, harness.conversation_id).len(),
            HOOK_FOLLOW_UP_CAP as usize
        );
        let diagnostics = harness.hooks.messenger.diagnostics();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("the most Warp allows (8)")),
            "{diagnostics:?}"
        );
        assert_eq!(
            app.read(|ctx| HookRuntime::as_ref(ctx)
                .turn_stops
                .chain_total(harness.conversation_id)),
            HOOK_FOLLOW_UP_CAP
        );
    });
}

#[test]
fn a_stop_hook_without_a_follow_up_finalizes_and_its_verdicts_are_ignored() {
    App::test((), |mut app| async move {
        let harness = hook_test_app(&mut app);
        let binding = stop_binding(None);
        harness
            .hooks
            .runner
            .script(binding.id.clone(), ScriptedRunner::deny_with("no more"));
        app.update(|ctx| harness.hooks.install_bindings(vec![binding], ctx));

        assert_eq!(
            stop_round(&mut app, &harness).await,
            TurnStopResolution::Finalize
        );
        assert!(queued_texts(&app, harness.conversation_id).is_empty());
        let diagnostics = harness.hooks.messenger.diagnostics();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("can only ask for a follow-up")),
            "{diagnostics:?}"
        );
    });
}

#[test]
fn nothing_is_held_without_a_stop_binding() {
    App::test((), |mut app| async move {
        let harness = hook_test_app(&mut app);
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
        let held =
            app.update(|ctx| hold_turn(harness.conversation_id, &harness.active_session, ctx));
        assert!(!held);
        assert!(
            app.update(|ctx| start_turn_stop(harness.conversation_id, ctx))
                .is_none()
        );
        assert_eq!(harness.hooks.runner.invocation_count(), 0);
    });
}

#[test]
fn an_answer_for_a_cancelled_hold_is_dropped() {
    App::test((), |mut app| async move {
        let harness = hook_test_app(&mut app);
        let binding = stop_binding(None);
        harness
            .hooks
            .runner
            .script(binding.id.clone(), continuation("too late"));
        app.update(|ctx| harness.hooks.install_bindings(vec![binding], ctx));
        let hold = harness.hooks.runner.hold();

        assert!(app.update(|ctx| hold_turn(harness.conversation_id, &harness.active_session, ctx)));
        let stop = app
            .update(|ctx| start_turn_stop(harness.conversation_id, ctx))
            .expect("the hooks start");
        app.update(|ctx| cancel_held_turn(harness.conversation_id, ctx));
        hold.release();
        let outcome = stop.await;
        assert_eq!(
            app.update(|ctx| resolve_turn_stop(harness.conversation_id, outcome, ctx)),
            TurnStopResolution::Dropped
        );
        assert!(queued_texts(&app, harness.conversation_id).is_empty());
    });
}

#[test]
fn a_hold_that_times_out_finalizes_without_the_hooks() {
    App::test((), |mut app| async move {
        let harness = hook_test_app(&mut app);
        let mut binding = stop_binding(None);
        binding.timeout = Duration::from_millis(100);
        harness
            .hooks
            .runner
            .script(binding.id.clone(), continuation("never arrives"));
        app.update(|ctx| harness.hooks.install_bindings(vec![binding], ctx));
        let _hold = harness.hooks.runner.hold();

        let started = Instant::now();
        assert_eq!(
            stop_round(&mut app, &harness).await,
            TurnStopResolution::Finalize
        );
        assert!(started.elapsed() >= Duration::from_millis(100));
        assert!(queued_texts(&app, harness.conversation_id).is_empty());
    });
}

#[test]
fn a_status_change_ends_a_turn_only_from_a_status_that_had_not_finished() {
    use ConversationStatus::*;
    let blocked = Blocked {
        blocked_action: "ls".to_owned(),
    };
    assert_eq!(
        ended_turn_status(&InProgress, &Cancelled),
        Some(TurnStopStatus::Cancelled)
    );
    assert_eq!(
        ended_turn_status(&InProgress, &Error),
        Some(TurnStopStatus::Error)
    );
    assert_eq!(
        ended_turn_status(&blocked, &Cancelled),
        Some(TurnStopStatus::Cancelled)
    );
    assert_eq!(
        ended_turn_status(&Error, &Cancelled),
        None,
        "cancelling a conversation that already failed ends no turn"
    );
    assert_eq!(ended_turn_status(&Cancelled, &Cancelled), None);
    assert_eq!(ended_turn_status(&Success, &Cancelled), None);
    assert_eq!(
        ended_turn_status(&InProgress, &Success),
        None,
        "a completed turn is held, not ended"
    );
    assert_eq!(ended_turn_status(&InProgress, &blocked), None);
}

#[test]
fn a_cancelled_turn_fires_stop_as_aborted_and_its_follow_up_is_refused() {
    App::test((), |mut app| async move {
        let harness = hook_test_app(&mut app);
        let stop = stop_binding(None);
        harness
            .hooks
            .runner
            .script(stop.id.clone(), continuation("keep going"));
        app.update(|ctx| harness.hooks.install_bindings(vec![stop], ctx));
        let turn = start_turn(&mut app, harness.conversation_id);

        assert!(app.update(|ctx| {
            hold_ended_turn(
                harness.conversation_id,
                &ConversationStatus::InProgress,
                &ConversationStatus::Cancelled,
                &harness.active_session,
                ctx,
            )
        }));
        let stops = app.update(|ctx| start_ended_turn_stops(harness.conversation_id, ctx));
        assert_eq!(stops.len(), 1);
        let outcome = stops.into_iter().next().unwrap().await;
        app.update(|ctx| finish_ended_turn_stop(harness.conversation_id, outcome, ctx));

        let envelope = &harness.hooks.adapter.encoded_envelopes()[0];
        assert!(matches!(
            &envelope.payload,
            HookPayload::TurnStop {
                status: TurnStopStatus::Cancelled,
                continuation_count: 0
            }
        ));
        assert_eq!(envelope.turn_id, Some(turn));
        assert!(
            queued_texts(&app, harness.conversation_id).is_empty(),
            "a follow-up is only sent after a turn that completed"
        );
        let diagnostics = harness.hooks.messenger.diagnostics();
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic
                    .message
                    .contains("after a turn that was cancelled or failed")
            }),
            "{diagnostics:?}"
        );
    });
}

#[test]
fn a_failed_turn_fires_stop_as_error_once_its_dispatch_starts() {
    App::test((), |mut app| async move {
        let harness = hook_test_app(&mut app);
        let stop = stop_binding(None);
        app.update(|ctx| harness.hooks.install_bindings(vec![stop], ctx));
        let turn = start_turn(&mut app, harness.conversation_id);

        assert!(app.update(|ctx| {
            hold_ended_turn(
                harness.conversation_id,
                &ConversationStatus::InProgress,
                &ConversationStatus::Error,
                &harness.active_session,
                ctx,
            )
        }));
        assert_eq!(
            ended_stop_count(&app, harness.conversation_id),
            1,
            "the stop waits to be started"
        );
        assert_eq!(harness.hooks.runner.invocation_count(), 0);

        let stops = app.update(|ctx| start_ended_turn_stops(harness.conversation_id, ctx));
        let outcome = stops.into_iter().next().unwrap().await;
        app.update(|ctx| finish_ended_turn_stop(harness.conversation_id, outcome, ctx));

        let envelope = &harness.hooks.adapter.encoded_envelopes()[0];
        assert!(matches!(
            &envelope.payload,
            HookPayload::TurnStop {
                status: TurnStopStatus::Error,
                continuation_count: 0
            }
        ));
        assert_eq!(envelope.turn_id, Some(turn));
        assert_eq!(ended_stop_count(&app, harness.conversation_id), 0);
    });
}

#[test]
fn a_turn_cancelled_under_its_completed_stop_fires_no_second_stop() {
    App::test((), |mut app| async move {
        let harness = hook_test_app(&mut app);
        let stop = stop_binding(None);
        harness
            .hooks
            .runner
            .script(stop.id.clone(), ScriptedRunner::deny_with("no more"));
        app.update(|ctx| harness.hooks.install_bindings(vec![stop], ctx));

        // The turn's completed stop ran for the conversation's first and only exchange count.
        assert_eq!(
            stop_round(&mut app, &harness).await,
            TurnStopResolution::Finalize
        );
        assert_eq!(harness.hooks.runner.invocation_count(), 1);

        // The turn is then cancelled: the same exchange count fires no second stop.
        assert!(!app.update(|ctx| {
            hold_ended_turn(
                harness.conversation_id,
                &ConversationStatus::InProgress,
                &ConversationStatus::Cancelled,
                &harness.active_session,
                ctx,
            )
        }));
        assert_eq!(ended_stop_count(&app, harness.conversation_id), 0);

        // The next turn is a turn of its own and fires its stop when it is cancelled.
        app.update(|ctx| {
            BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                history
                    .conversation_mut(&harness.conversation_id)
                    .expect("the conversation exists")
                    .append_reassigned_exchange(
                        &ResponseStreamId::new(),
                        streaming_exchange(),
                        harness.terminal_view_id,
                        ctx,
                    )
                    .expect("the exchange is appended");
            });
        });
        let turn = start_turn(&mut app, harness.conversation_id);
        assert!(app.update(|ctx| {
            hold_ended_turn(
                harness.conversation_id,
                &ConversationStatus::InProgress,
                &ConversationStatus::Cancelled,
                &harness.active_session,
                ctx,
            )
        }));
        let stops = app.update(|ctx| start_ended_turn_stops(harness.conversation_id, ctx));
        let outcome = stops.into_iter().next().unwrap().await;
        app.update(|ctx| finish_ended_turn_stop(harness.conversation_id, outcome, ctx));

        let payloads = harness
            .hooks
            .adapter
            .encoded_envelopes()
            .iter()
            .map(|envelope| envelope.payload.clone())
            .collect::<Vec<_>>();
        assert!(matches!(
            payloads.as_slice(),
            [
                HookPayload::TurnStop {
                    status: TurnStopStatus::Completed,
                    ..
                },
                HookPayload::TurnStop {
                    status: TurnStopStatus::Cancelled,
                    ..
                }
            ]
        ));
        assert_eq!(
            harness.hooks.adapter.encoded_envelopes()[1].turn_id,
            Some(turn)
        );
    });
}
