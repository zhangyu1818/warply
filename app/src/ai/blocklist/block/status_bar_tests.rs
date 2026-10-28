use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use chrono::Local;
use warpui::{App, EntityId, SingletonEntity};

use super::BlocklistAIStatusBarEvent;
use crate::ai::agent::conversation::AIConversationId;
use crate::ai::agent::task::TaskId;
use crate::ai::agent::{AIAgentExchange, AIAgentExchangeId, AIAgentOutputStatus};
use crate::ai::blocklist::{BlocklistAIHistoryEvent, BlocklistAIHistoryModel, ResponseStreamId};
use crate::ai::llms::LLMId;
use crate::test_util::terminal::{add_window_with_terminal, initialize_app_for_terminal_view};

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
fn inactive_exchange_updates_do_not_close_summarization_dialog() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let terminal = add_window_with_terminal(&mut app, None);
        let terminal_view_id = terminal.id();
        let status_bar = terminal.read(&app, |terminal, ctx| {
            terminal.input().as_ref(ctx).agent_status_bar().clone()
        });
        let history = app.read(BlocklistAIHistoryModel::handle);
        let (conversation_id, task_id, exchange_ids): (
            AIConversationId,
            TaskId,
            Vec<AIAgentExchangeId>,
        ) = history.update(&mut app, |history, ctx| {
            let conversation_id = history.start_new_conversation(terminal_view_id, false, ctx);
            let task_id = history
                .conversation(&conversation_id)
                .expect("conversation should exist")
                .get_root_task_id()
                .clone();
            let mut exchange_ids = Vec::new();
            for _ in 0..2 {
                let exchange = streaming_exchange();
                let exchange_id = exchange.id;
                history
                    .conversation_mut(&conversation_id)
                    .expect("conversation should exist")
                    .append_reassigned_exchange(
                        &ResponseStreamId::new_for_test(),
                        exchange,
                        terminal_view_id,
                        ctx,
                    )
                    .expect("exchange should append");
                exchange_ids.push(exchange_id);
            }
            (conversation_id, task_id, exchange_ids)
        });
        for exchange_id in &exchange_ids {
            history.update(&mut app, |_, ctx| {
                ctx.emit(BlocklistAIHistoryEvent::AppendedExchange {
                    exchange_id: *exchange_id,
                    conversation_id,
                    task_id: task_id.clone(),
                    terminal_view_id,
                    is_hidden: false,
                    response_stream_id: None,
                });
            });
        }
        let dialog_events = Rc::new(RefCell::new(Vec::new()));
        let observed_dialog_events = dialog_events.clone();
        app.update(|ctx| {
            ctx.subscribe_to_view(&status_bar, move |_, event, _| {
                let BlocklistAIStatusBarEvent::SummarizationCancelDialogToggled { is_open } = event;
                observed_dialog_events.borrow_mut().push(*is_open);
            });
        });
        status_bar.update(&mut app, |bar, ctx| {
            bar.open_summarization_cancel_dialog(ctx);
        });

        history.update(&mut app, |_, ctx| {
            ctx.emit(BlocklistAIHistoryEvent::UpdatedStreamingExchange {
                exchange_id: exchange_ids[0],
                conversation_id,
                terminal_view_id,
                is_hidden: false,
            });
        });
        assert_eq!(*dialog_events.borrow(), vec![true]);

        history.update(&mut app, |_, ctx| {
            ctx.emit(BlocklistAIHistoryEvent::UpdatedStreamingExchange {
                exchange_id: exchange_ids[1],
                conversation_id,
                terminal_view_id: EntityId::new(),
                is_hidden: false,
            });
        });
        assert_eq!(*dialog_events.borrow(), vec![true, false]);
    });
}
