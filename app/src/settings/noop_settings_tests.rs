#[test]
fn retained_settings_updates_are_not_noops() {
    let files = [
        ("app/src/lib.rs", include_str!("../lib.rs")),
        (
            "app/src/pane_group/mod.rs",
            include_str!("../pane_group/mod.rs"),
        ),
        (
            "app/src/quit_warning/mod.rs",
            include_str!("../quit_warning/mod.rs"),
        ),
        (
            "app/src/resource_center/mod.rs",
            include_str!("../resource_center/mod.rs"),
        ),
        (
            "app/src/workspace/view.rs",
            include_str!("../workspace/view.rs"),
        ),
        (
            "app/src/workspace/header_toolbar_editor.rs",
            include_str!("../workspace/header_toolbar_editor.rs"),
        ),
        (
            "app/src/terminal/view.rs",
            include_str!("../terminal/view.rs"),
        ),
        (
            "app/src/terminal/view/open_in_warp.rs",
            include_str!("../terminal/view/open_in_warp.rs"),
        ),
        (
            "app/src/terminal/view/zero_state_block.rs",
            include_str!("../terminal/view/zero_state_block.rs"),
        ),
        (
            "app/src/prompt/editor_modal.rs",
            include_str!("../prompt/editor_modal.rs"),
        ),
        (
            "app/src/ai/blocklist/agent_view/agent_input_footer/editor.rs",
            include_str!("../ai/blocklist/agent_view/agent_input_footer/editor.rs"),
        ),
    ];

    let disallowed = [
        "Settings::handle(ctx).update(ctx, |_",
        "Settings::handle(model_ctx).update(model_ctx, |_",
        "Prompt::handle(ctx).update(ctx, |_prompt, _ctx| {});",
        "if tips_completed.skipped_or_completed {}",
        "if let Some(_path) = tab_config_path {}",
        "let _selection = if",
        "let _new_setup = self",
        "fn set_terminal_font_size(&mut self, _new_font_size",
        "let _next_index = if increase",
    ];

    let mut failures = Vec::new();
    for (path, contents) in files {
        for pattern in disallowed {
            if contents.contains(pattern) {
                failures.push(format!("{path}: contains `{pattern}`"));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "retained settings updates must persist values:\n{}",
        failures.join("\n")
    );
}
