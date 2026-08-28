use std::sync::Arc;

use parking_lot::{FairMutex, Mutex};
use warp_core::SessionId;
use warpui::App;

use super::*;
use crate::terminal::model::session::{SessionInfo, Sessions};

#[derive(Clone, Default)]
struct TestEventLoopSender {
    messages: Arc<Mutex<Vec<Message>>>,
}

impl EventLoopSender for TestEventLoopSender {
    fn send(&self, message: Message) -> Result<(), EventLoopSendError> {
        self.messages.lock().push(message);
        Ok(())
    }
}

fn terminal_model() -> Arc<FairMutex<TerminalModel>> {
    Arc::new(FairMutex::new(TerminalModel::mock(None, None)))
}

#[test]
fn native_shell_completions_queues_the_generator_command_for_the_active_sessions_shell() {
    App::test((), |mut app| async move {
        let model = terminal_model();
        let (model_events_tx, model_events_rx) = async_channel::unbounded();
        let (_executor_command_tx, executor_command_rx) = async_channel::unbounded();
        let mut sessions = Sessions::new_for_test();
        let session_id = SessionId::from(42);
        sessions.register_session_for_test(
            SessionInfo::new_for_test()
                .with_id(session_id)
                .with_shell_type(ShellType::Fish),
        );
        let sessions = app.add_model(|_| sessions);
        let model_events = app.add_model(|ctx| {
            let mut dispatcher = ModelEventDispatcher::new(model_events_rx, sessions.clone(), ctx);
            dispatcher.set_active_session_id(session_id);
            dispatcher
        });
        let line_editor_status =
            app.add_model(|ctx| LineEditorStatus::new(model_events.clone(), sessions.clone(), ctx));
        let sender = TestEventLoopSender::default();
        let controller = app.add_model(|ctx| {
            PtyController::new(
                sender.clone(),
                model_events,
                line_editor_status,
                sessions,
                executor_command_rx,
                model,
                ctx,
            )
        });

        let (results_tx, _results_rx) = async_channel::unbounded();
        controller.update(&mut app, |controller, ctx| {
            controller.run_native_shell_completions("git ch".to_owned(), results_tx, ctx);
        });

        // The line editor isn't active by default, so the write should still be queued rather
        // than sent to the event loop.
        assert!(sender.messages.lock().is_empty());
        controller.read(&app, |controller, _| {
            assert_eq!(controller.pending_writes.len(), 1);
            let Some(PtyWrite::RunNativeShellCompletions {
                command,
                shell_type,
                ..
            }) = controller.pending_writes.front()
            else {
                panic!("expected a queued RunNativeShellCompletions write");
            };
            assert_eq!(*shell_type, ShellType::Fish);
            assert_eq!(
                command,
                " warp_run_generator_command_native_completions 676974206368"
            );
        });

        drop(model_events_tx);
    });
}

#[test]
fn native_shell_completions_reports_no_matches_without_an_active_session() {
    App::test((), |mut app| async move {
        let model = terminal_model();
        let (model_events_tx, model_events_rx) = async_channel::unbounded();
        let (_executor_command_tx, executor_command_rx) = async_channel::unbounded();
        let sessions = app.add_model(|_| Sessions::new_for_test());
        let model_events =
            app.add_model(|ctx| ModelEventDispatcher::new(model_events_rx, sessions.clone(), ctx));
        let line_editor_status =
            app.add_model(|ctx| LineEditorStatus::new(model_events.clone(), sessions.clone(), ctx));
        let sender = TestEventLoopSender::default();
        let controller = app.add_model(|ctx| {
            PtyController::new(
                sender.clone(),
                model_events,
                line_editor_status,
                sessions,
                executor_command_rx,
                model,
                ctx,
            )
        });

        let (results_tx, results_rx) = async_channel::unbounded();
        controller.update(&mut app, |controller, ctx| {
            controller.run_native_shell_completions("git ch".to_owned(), results_tx, ctx);
        });

        let (completions, replacement_span) = results_rx
            .try_recv()
            .expect("should immediately receive empty results");
        assert!(completions.is_empty());
        assert!(replacement_span.is_none());
        controller.read(&app, |controller, _| {
            assert!(controller.pending_writes.is_empty());
        });
        assert!(sender.messages.lock().is_empty());

        drop(model_events_tx);
    });
}
