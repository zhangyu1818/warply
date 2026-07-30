use super::{CollapsibleElementState, CollapsibleExpansionState};
#[cfg(feature = "local_fs")]
use super::{AIBlockEvent, open_code_action_event};
use crate::ai::acp::AcpToolCall;
#[cfg(feature = "local_fs")]
use crate::code::editor_management::CodeSource;
use crate::settings::AISettings;
#[cfg(feature = "local_fs")]
use std::path::PathBuf;
#[cfg(feature = "local_fs")]
use warp_util::path::LineAndColumnArg;
use crate::test_util::settings::initialize_settings_for_tests;
use agent_client_protocol::schema::{Diff, ToolCall, ToolCallContent, ToolKind};
use ai::diff_validation::DiffType;
use settings::Setting;
use warpui::{App, SingletonEntity};

#[test]
fn reasoning_auto_collapses_when_user_has_not_manually_toggled() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);
        let mut state = CollapsibleElementState::default();
        app.update(|ctx| {
            state.finish_reasoning(ctx);
        });

        assert!(matches!(
            state.expansion_state,
            CollapsibleExpansionState::Collapsed
        ));
    });
}

#[test]
fn always_show_thinking_stays_expanded_after_finish() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);
        AISettings::handle(&app).update(&mut app, |settings, ctx| {
            settings
                .thinking_display_mode
                .set_value(crate::settings::ThinkingDisplayMode::AlwaysShow, ctx)
                .unwrap();
        });

        let mut state = CollapsibleElementState::default();
        app.update(|ctx| {
            state.finish_reasoning(ctx);
        });

        assert!(matches!(
            state.expansion_state,
            CollapsibleExpansionState::Expanded {
                is_finished: true,
                scroll_pinned_to_bottom: false
            }
        ));
    });
}

#[test]
fn manual_collapse_while_streaming_stays_collapsed_after_finish() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);
        let mut state = CollapsibleElementState::default();

        state.toggle_expansion();
        app.update(|ctx| {
            state.finish_reasoning(ctx);
        });

        assert!(matches!(
            state.expansion_state,
            CollapsibleExpansionState::Collapsed
        ));
    });
}

#[test]
fn manual_reexpand_while_streaming_stays_expanded_after_finish() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);
        let mut state = CollapsibleElementState::default();

        state.toggle_expansion();
        state.toggle_expansion();
        app.update(|ctx| {
            state.finish_reasoning(ctx);
        });

        assert!(matches!(
            state.expansion_state,
            CollapsibleExpansionState::Expanded {
                is_finished: true,
                scroll_pinned_to_bottom: false
            }
        ));
    });
}

#[test]
fn acp_tool_call_file_diffs_convert_to_code_diff_view_model() {
    let call = AcpToolCall::from_acp(
        ToolCall::new("edit-1", "Edit file")
            .kind(ToolKind::Edit)
            .content(vec![ToolCallContent::from(
                Diff::new("src/main.rs", "fn main() {\n    println!(\"hi\");\n}\n")
                    .old_text("fn main() {}\n"),
            )]),
    );

    let diffs = super::acp_tool_call_file_diffs(&call);

    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].base.file_path, "src/main.rs");
    assert_eq!(diffs[0].base.content, "fn main() {}\n");
    let DiffType::Update { deltas, rename } = &diffs[0].diff_type else {
        panic!("expected update diff");
    };
    assert!(rename.is_none());
    assert_eq!(deltas.len(), 1);
    assert_eq!(deltas[0].replacement_line_range, 1..2);
    assert_eq!(
        deltas[0].insertion,
        "fn main() {\n    println!(\"hi\");\n}\n"
    );
}

#[cfg(feature = "local_fs")]
#[test]
fn open_code_action_routes_links_to_configured_editor_and_non_links_to_warp() {
    let linked_source = CodeSource::Link {
        path: PathBuf::from("/workspace/project/src/main.rs"),
        range_start: Some(LineAndColumnArg {
            line_num: 42,
            column_num: Some(7),
        }),
        range_end: None,
    };

    assert!(matches!(
        open_code_action_event(
            &linked_source,
            crate::util::file::external_editor::settings::EditorLayout::SplitPane,
        ),
        AIBlockEvent::OpenDetectedFilePath {
            absolute_path,
            line_and_column_num: Some(LineAndColumnArg {
                line_num: 42,
                column_num: Some(7),
            }),
            target_override: None,
        } if absolute_path.as_path() == std::path::Path::new("/workspace/project/src/main.rs")
    ));

    let non_link_source = CodeSource::ProjectRules {
        path: PathBuf::from("/workspace/project/WARP.md"),
    };

    assert!(matches!(
        open_code_action_event(
            &non_link_source,
            crate::util::file::external_editor::settings::EditorLayout::NewTab,
        ),
        AIBlockEvent::OpenCodeInWarp {
            source,
            layout: crate::util::file::external_editor::settings::EditorLayout::NewTab,
        } if source == non_link_source
    ));
}
