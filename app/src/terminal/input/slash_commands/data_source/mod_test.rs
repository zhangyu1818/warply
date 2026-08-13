use crate::ai::acp::{events::AcpEvent, model::AcpAgentModel};
use crate::ai::blocklist::BlocklistAIHistoryModel;
use crate::search::SyncDataSource;
use crate::search::data_source::Query;
use crate::search::slash_command_menu::fuzzy_match::SlashCommandFuzzyMatchResult;
use crate::search::slash_command_menu::static_commands::commands;
use crate::terminal::input::tests::{add_window_with_bootstrapped_terminal, initialize_app};
use warpui::SingletonEntity;

use super::{AcceptSlashCommandOrSavedPrompt, InlineItem, prefix_match_bonus};

#[test]
fn exact_match_returns_full_bonus() {
    // Query "new" exactly matches the name "/new" (after stripping '/').
    let bonus = prefix_match_bonus("new", "/new");
    assert!((bonus - 100.0).abs() < f64::EPSILON);
}

#[test]
fn partial_prefix_returns_proportional_bonus() {
    // "for" is a prefix of "fork" → coverage 3/4 = 75.
    let bonus = prefix_match_bonus("for", "/fork");
    assert!((bonus - 75.0).abs() < f64::EPSILON);
}

#[test]
fn non_prefix_returns_zero() {
    // "new" is NOT a prefix of "create-new-project".
    let bonus = prefix_match_bonus("new", "/create-new-project");
    assert!((bonus - 0.0).abs() < f64::EPSILON);
}

#[test]
fn case_insensitive() {
    let bonus = prefix_match_bonus("new", "/New");
    assert!((bonus - 100.0).abs() < f64::EPSILON);
}

#[test]
fn name_without_slash_prefix() {
    let bonus = prefix_match_bonus("figma", "figma-create-new-file");
    let coverage = 5.0 / 21.0 * 100.0;
    assert!((bonus - coverage).abs() < f64::EPSILON);
}

#[test]
fn short_prefix_match_ranks_above_longer_fuzzy_match() {
    // Simulates the reported issue: query "new" should give /new a much
    // higher combined score than /figma-create-new-file.
    let short_match = SlashCommandFuzzyMatchResult::try_match("new", "/new", None).unwrap();
    let long_match =
        SlashCommandFuzzyMatchResult::try_match("new", "/figma-create-new-file", None).unwrap();

    const SCORE_MULTIPLIER: f64 = 1000.0;

    let short_score = short_match.score() * SCORE_MULTIPLIER
        + prefix_match_bonus("new", "/new") * SCORE_MULTIPLIER
        + 1.0 / "/new".len() as f64;
    let long_score = long_match.score() * SCORE_MULTIPLIER
        + prefix_match_bonus("new", "/figma-create-new-file") * SCORE_MULTIPLIER
        + 1.0 / "/figma-create-new-file".len() as f64;

    assert!(
        short_score > long_score,
        "/new score ({short_score}) should be greater than /figma-create-new-file score ({long_score})"
    );
}

#[test]
fn retained_terminal_static_commands_are_active() {
    warpui::App::test((), |mut app| async move {
        initialize_app(&mut app);

        let terminal = add_window_with_bootstrapped_terminal(&mut app, None, None).await;
        let input = terminal.read(&app, |view, _| view.input().clone());
        let slash_command_data_source =
            input.read(&app, |input, _| input.slash_command_data_source.clone());

        slash_command_data_source.read(&app, |data_source, _| {
            let names = active_static_command_names(data_source);

            for command in [
                commands::AGENT.name,
                commands::NEW.name,
                commands::CONVERSATIONS.name,
                commands::PROMPTS.name,
                commands::ADD_PROMPT.name,
                commands::ADD_RULE.name,
                commands::OPEN_RULES.name,
                commands::RENAME_TAB.name,
                commands::SET_TAB_COLOR.name,
                commands::EDIT.name,
                commands::CREATE_DOCKER_SANDBOX.name,
                commands::OPEN_REPO.name,
            ] {
                assert!(
                    names.contains(&command),
                    "{command} should remain available as a local app slash command"
                );
            }

            for removed in [
                commands::PLAN_NAME,
                commands::INIT_NAME,
                "/create-new-project",
                commands::PR_COMMENTS_NAME,
            ] {
                assert!(
                    !names.contains(&removed),
                    "{removed} should be supplied by ACP available commands instead"
                );
            }
        });
    });
}

#[test]
fn acp_command_inline_item_uses_slash_name_and_input_hint() {
    use agent_client_protocol::schema::{
        AvailableCommand, AvailableCommandInput, UnstructuredCommandInput,
    };

    warpui::App::test((), |mut app| async move {
        initialize_app(&mut app);

        app.read(|ctx| {
            let command = AvailableCommand::new("review", "Review changes").input(
                AvailableCommandInput::Unstructured(UnstructuredCommandInput::new("optional task")),
            );
            let item = InlineItem::from_acp_command(&command, ctx);

            assert_eq!(item.name, "/review");
            assert_eq!(item.description.as_deref(), Some("Review changes"));
            assert!(matches!(
                item.action,
                AcceptSlashCommandOrSavedPrompt::AcpCommand {
                    name,
                    input_hint: Some(_),
                    ..
                } if name == "review"
            ));
        });
    });
}

#[test]
fn acp_available_commands_are_visible_only_for_active_acp_conversation() {
    use agent_client_protocol::schema::AvailableCommand;

    warpui::App::test((), |mut app| async move {
        initialize_app(&mut app);

        let terminal = add_window_with_bootstrapped_terminal(&mut app, None, None).await;
        let terminal_view_id = terminal.read(&app, |view, _| view.view_id());
        let input = terminal.read(&app, |view, _| view.input().clone());
        let slash_command_data_source =
            input.read(&app, |input, _| input.slash_command_data_source.clone());

        let (conversation_a, conversation_b) =
            BlocklistAIHistoryModel::handle(&app).update(&mut app, |history, ctx| {
                let conversation_a = history.start_new_conversation(terminal_view_id, false, ctx);
                let conversation_b = history.start_new_conversation(terminal_view_id, false, ctx);
                (conversation_a, conversation_b)
            });

        AcpAgentModel::handle(&app).update(&mut app, |model, _| {
            model.set_available_commands_for_test(
                conversation_a,
                vec![AvailableCommand::new("alpha", "Alpha command")],
            );
            model.set_available_commands_for_test(
                conversation_b,
                vec![AvailableCommand::new("beta", "Beta command")],
            );
        });

        BlocklistAIHistoryModel::handle(&app).update(&mut app, |history, ctx| {
            history.set_active_conversation_id(conversation_a, terminal_view_id, ctx);
        });

        slash_command_data_source.read(&app, |data_source, ctx| {
            assert_eq!(acp_command_names(data_source, ctx, "alpha"), vec!["alpha"]);
            assert!(acp_command_names(data_source, ctx, "beta").is_empty());
        });

        BlocklistAIHistoryModel::handle(&app).update(&mut app, |history, ctx| {
            history.set_active_conversation_id(conversation_b, terminal_view_id, ctx);
        });

        slash_command_data_source.read(&app, |data_source, ctx| {
            assert!(acp_command_names(data_source, ctx, "alpha").is_empty());
            assert_eq!(acp_command_names(data_source, ctx, "beta"), vec!["beta"]);
        });
    });
}

#[test]
fn acp_available_commands_are_visible_in_zero_state_for_active_acp_conversation() {
    use agent_client_protocol::schema::AvailableCommand;

    warpui::App::test((), |mut app| async move {
        initialize_app(&mut app);

        let terminal = add_window_with_bootstrapped_terminal(&mut app, None, None).await;
        let terminal_view_id = terminal.read(&app, |view, _| view.view_id());
        let input = terminal.read(&app, |view, _| view.input().clone());
        let slash_command_data_source =
            input.read(&app, |input, _| input.slash_command_data_source.clone());
        let zero_state_source =
            app.add_model(|_| super::ZeroStateDataSource::new(&slash_command_data_source));

        let conversation_id =
            BlocklistAIHistoryModel::handle(&app).update(&mut app, |history, ctx| {
                let conversation_id = history.start_new_conversation(terminal_view_id, false, ctx);
                history.set_active_conversation_id(conversation_id, terminal_view_id, ctx);
                conversation_id
            });

        AcpAgentModel::handle(&app).update(&mut app, |model, _| {
            model.set_available_commands_for_test(
                conversation_id,
                vec![AvailableCommand::new("plan", "Plan the work")],
            );
        });

        let names = zero_state_source.read(&app, |source, ctx| {
            source
                .run_query(&Query::from(""), ctx)
                .expect("zero-state query should run")
                .into_iter()
                .filter_map(|result| match result.accept_result() {
                    AcceptSlashCommandOrSavedPrompt::AcpCommand { name, .. } => Some(name),
                    _ => None,
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(names, vec!["plan"]);
    });
}

#[test]
fn acp_available_commands_update_emits_active_commands_event() {
    use agent_client_protocol::schema::AvailableCommand;

    warpui::App::test((), |mut app| async move {
        initialize_app(&mut app);

        let terminal = add_window_with_bootstrapped_terminal(&mut app, None, None).await;
        let terminal_view_id = terminal.read(&app, |view, _| view.view_id());
        let input = terminal.read(&app, |view, _| view.input().clone());
        let slash_command_data_source =
            input.read(&app, |input, _| input.slash_command_data_source.clone());

        let conversation_id =
            BlocklistAIHistoryModel::handle(&app).update(&mut app, |history, ctx| {
                let conversation_id = history.start_new_conversation(terminal_view_id, false, ctx);
                history.set_active_conversation_id(conversation_id, terminal_view_id, ctx);
                conversation_id
            });

        let (tx, rx) = async_channel::unbounded();
        app.update(|ctx| {
            ctx.subscribe_to_model(&slash_command_data_source, move |_, _, _| {
                tx.try_send(()).expect("can record data source update");
            });
        });

        let commands = vec![AvailableCommand::new("init", "Initialize project rules")];
        AcpAgentModel::handle(&app).update(&mut app, |model, ctx| {
            model.set_available_commands_for_test(conversation_id, commands.clone());
            ctx.emit(AcpEvent::AvailableCommandsUpdated { commands });
        });

        assert!(
            rx.try_recv().is_ok(),
            "ACP command changes should notify slash command menus to rerun their current query"
        );
    });
}

fn active_static_command_names(data_source: &super::SlashCommandDataSource) -> Vec<&'static str> {
    data_source
        .active_commands()
        .map(|(_, command)| command.name)
        .collect()
}

fn acp_command_names(
    data_source: &super::SlashCommandDataSource,
    app: &warpui::AppContext,
    query: &str,
) -> Vec<String> {
    data_source
        .run_query(&Query::from(query), app)
        .expect("query should run")
        .into_iter()
        .filter_map(|result| match result.accept_result() {
            AcceptSlashCommandOrSavedPrompt::AcpCommand { name, .. } => Some(name),
            _ => None,
        })
        .collect()
}
