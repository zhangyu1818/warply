use ai::hooks::test_util::{test_binding, test_source_id};
use ai::hooks::{HookBoundary, HookPayload};
use warpui::App;

use super::*;
use crate::ai::blocklist::hooks::test_utils::{hook_test_app, settle};

fn stream_id() -> ResponseStreamId {
    ResponseStreamId::new_for_test()
}

fn text(id: &str, update: TextUpdate) -> StreamObservation {
    StreamObservation::Text {
        message_id: id.to_owned(),
        update,
    }
}

fn reasoning(id: &str, update: TextUpdate, finished: Option<Duration>) -> StreamObservation {
    StreamObservation::Reasoning {
        message_id: id.to_owned(),
        update,
        finished_duration: finished,
    }
}

fn completed_ids(completed: &[CompletedMessage]) -> Vec<(&str, &str)> {
    completed
        .iter()
        .map(|message| (message.message_id.as_str(), message.text.as_str()))
        .collect()
}

#[test]
fn a_text_message_is_emitted_once_when_the_next_text_message_starts() {
    let mut state = StreamObserverState::default();
    let stream = stream_id();
    let conversation = AIConversationId::new();

    let completed = state.apply_batch(
        &stream,
        conversation,
        Some(conversation),
        vec![
            text("m1", TextUpdate::Set("Hel".to_owned())),
            text("m1", TextUpdate::Append("lo".to_owned())),
        ],
    );
    assert!(completed.is_empty(), "chunks never emit");

    let completed = state.apply_batch(
        &stream,
        conversation,
        Some(conversation),
        vec![text("m2", TextUpdate::Set("Next".to_owned()))],
    );
    assert_eq!(completed_ids(&completed), vec![("m1", "Hello")]);
    assert_eq!(completed[0].conversation_id, Some(conversation));

    let completed = state.apply_batch(
        &stream,
        conversation,
        Some(conversation),
        vec![text("m1", TextUpdate::Append("late".to_owned()))],
    );
    assert!(
        completed.is_empty(),
        "a late chunk for an emitted message does not re-emit"
    );

    let completed = state.finish(&stream, false);
    assert_eq!(completed_ids(&completed), vec![("m2", "Next")]);
}

#[test]
fn reasoning_between_two_text_messages_ends_the_first_and_emits_on_finish() {
    let mut state = StreamObserverState::default();
    let stream = stream_id();
    let conversation = AIConversationId::new();

    let completed = state.apply_batch(
        &stream,
        conversation,
        Some(conversation),
        vec![
            text("t1", TextUpdate::Set("first".to_owned())),
            reasoning("r1", TextUpdate::Set("think".to_owned()), None),
            reasoning("r1", TextUpdate::Append("ing".to_owned()), None),
        ],
    );
    assert_eq!(completed_ids(&completed), vec![("t1", "first")]);
    assert!(matches!(completed[0].kind, CompletedKind::Text));

    let completed = state.apply_batch(
        &stream,
        conversation,
        Some(conversation),
        vec![
            reasoning(
                "r1",
                TextUpdate::Append(String::new()),
                Some(Duration::from_secs(3)),
            ),
            reasoning(
                "r1",
                TextUpdate::Append(String::new()),
                Some(Duration::from_secs(3)),
            ),
            text("t2", TextUpdate::Set("second".to_owned())),
        ],
    );
    assert_eq!(completed_ids(&completed), vec![("r1", "thinking")]);
    assert_eq!(
        completed[0].kind,
        CompletedKind::Reasoning {
            duration: Some(Duration::from_secs(3))
        }
    );

    let completed = state.finish(&stream, false);
    assert_eq!(completed_ids(&completed), vec![("t2", "second")]);
}

#[test]
fn a_text_message_still_streaming_is_emitted_when_the_stream_ends() {
    let mut state = StreamObserverState::default();
    let stream = stream_id();
    let conversation = AIConversationId::new();
    let completed = state.apply_batch(
        &stream,
        conversation,
        Some(conversation),
        vec![text("only", TextUpdate::Set("done".to_owned()))],
    );
    assert!(completed.is_empty());
    let completed = state.finish(&stream, false);
    assert_eq!(completed_ids(&completed), vec![("only", "done")]);
    assert!(state.finish(&stream, false).is_empty(), "state is dropped");
}

#[test]
fn a_cancelled_stream_does_not_emit_its_partial_text() {
    let mut state = StreamObserverState::default();
    let stream = stream_id();
    let conversation = AIConversationId::new();
    state.apply_batch(
        &stream,
        conversation,
        Some(conversation),
        vec![text("partial", TextUpdate::Set("half".to_owned()))],
    );
    assert!(state.finish(&stream, true).is_empty());
}

#[test]
fn a_tool_call_message_ends_the_text_before_it() {
    let mut state = StreamObserverState::default();
    let stream = stream_id();
    let conversation = AIConversationId::new();
    let completed = state.apply_batch(
        &stream,
        conversation,
        Some(conversation),
        vec![
            text("t1", TextUpdate::Set("before".to_owned())),
            StreamObservation::OtherMessage {
                message_id: "tool".to_owned(),
            },
            text("t2", TextUpdate::Set("after".to_owned())),
        ],
    );
    assert_eq!(completed_ids(&completed), vec![("t1", "before")]);
}

#[test]
fn a_conversation_split_attributes_messages_to_the_right_conversation() {
    let mut state = StreamObserverState::default();
    let stream = stream_id();
    let before = AIConversationId::new();
    let after = AIConversationId::new();
    let completed = state.apply_batch(
        &stream,
        before,
        Some(after),
        vec![
            text("old", TextUpdate::Set("in old".to_owned())),
            StreamObservation::ConversationSplit,
            text("new", TextUpdate::Set("in new".to_owned())),
            text("new2", TextUpdate::Set("also new".to_owned())),
        ],
    );
    assert_eq!(
        completed_ids(&completed),
        vec![("old", "in old"), ("new", "in new")]
    );
    assert_eq!(completed[0].conversation_id, Some(before));
    assert_eq!(completed[1].conversation_id, Some(after));

    let completed = state.finish(&stream, false);
    assert_eq!(completed_ids(&completed), vec![("new2", "also new")]);
    assert_eq!(completed[0].conversation_id, Some(after));
}

#[test]
fn reasoning_completes_once_on_the_first_update_that_carries_a_duration() {
    let mut state = StreamObserverState::default();
    let stream = stream_id();
    let conversation = AIConversationId::new();
    let completed = state.apply_batch(
        &stream,
        conversation,
        Some(conversation),
        vec![
            reasoning("r1", TextUpdate::Set("first ".to_owned()), None),
            reasoning(
                "r1",
                TextUpdate::Append("thought".to_owned()),
                Some(Duration::from_secs(1)),
            ),
            // A later replacement of the same message, as the server sometimes sends, is not
            // a second completion.
            reasoning(
                "r1",
                TextUpdate::Set("first thought, revised".to_owned()),
                Some(Duration::from_secs(1)),
            ),
            reasoning("r1", TextUpdate::Append(" more".to_owned()), None),
        ],
    );
    assert_eq!(completed_ids(&completed), vec![("r1", "first thought")]);
    assert!(state.finish(&stream, false).is_empty());
}

#[test]
fn acp_deltas_map_to_observations_and_respect_activity() {
    App::test((), |mut app| async move {
        let harness = hook_test_app(&mut app);
        let stream = stream_id();
        let deltas = vec![
            AcpStreamDelta::AssistantText { text: "Hi" },
            AcpStreamDelta::OtherMessage {
                tool_call_id: "call",
            },
            AcpStreamDelta::AssistantThought { text: "hm" },
            AcpStreamDelta::AssistantThoughtFinished,
        ];

        let conversation_id = harness.conversation_id;
        assert!(
            app.read(|ctx| observations_for_acp_deltas(&stream, &deltas, conversation_id, ctx))
                .is_empty(),
            "an empty registry means nothing is collected"
        );

        let source = test_source_id("s");
        app.update(|ctx| {
            harness.hooks.install_bindings(
                vec![
                    test_binding(source.clone(), HookBoundary::ResponseComplete, 0, None),
                    test_binding(source, HookBoundary::ReasoningComplete, 0, None),
                ],
                ctx,
            )
        });
        assert!(
            app.read(|ctx| observations_for_acp_deltas(
                &stream,
                &deltas,
                AIConversationId::new(),
                ctx
            ))
            .is_empty(),
            "a conversation this client does not drive is not observed"
        );

        // The harness conversation carries no streamed messages for this stream, so the
        // producer falls back to the per-stream prefixes.
        let observations =
            app.read(|ctx| observations_for_acp_deltas(&stream, &deltas, conversation_id, ctx));
        let assistant = format!("acp-{}-assistant", stream.as_str());
        let thought = format!("acp-{}-thought", stream.as_str());
        let tool = format!("acp-{}-tool-call", stream.as_str());
        assert_eq!(
            observations,
            vec![
                text(&assistant, TextUpdate::Append("Hi".to_owned())),
                StreamObservation::OtherMessage { message_id: tool },
                reasoning(&thought, TextUpdate::Append("hm".to_owned()), None),
                reasoning(
                    &thought,
                    TextUpdate::Append(String::new()),
                    Some(Duration::ZERO)
                ),
            ]
        );

        // The tool call ends the text and the finished marker completes the reasoning, so
        // exactly two envelopes reach the adapter once the batch is applied.
        app.update(|ctx| {
            observe(
                &stream,
                conversation_id,
                Some(conversation_id),
                observations,
                &harness.active_session,
                ctx,
            )
        });
        settle(&mut app, |_| harness.hooks.adapter.encoded_count() == 2).await;
        let encoded = harness.hooks.adapter.encoded_envelopes();
        let kinds = encoded
            .iter()
            .map(|envelope| match &envelope.payload {
                HookPayload::ResponseComplete { message_id, text } => {
                    format!("text:{message_id}:{text}")
                }
                HookPayload::ReasoningComplete {
                    message_id, text, ..
                } => format!("reasoning:{message_id}:{text}"),
                HookPayload::SessionStart { .. }
                | HookPayload::PromptSubmit { .. }
                | HookPayload::ToolPre { .. }
                | HookPayload::ToolResult { .. }
                | HookPayload::ToolFailure { .. }
                | HookPayload::TurnStop { .. } => "other".to_owned(),
            })
            .collect::<HashSet<_>>();
        assert_eq!(
            kinds,
            HashSet::from([
                format!("text:{assistant}:Hi"),
                format!("reasoning:{thought}:hm"),
            ])
        );
        assert!(encoded.iter().all(|e| e.conversation_id == conversation_id));
    });
}
