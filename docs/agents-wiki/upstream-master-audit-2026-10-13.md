# Upstream Master Audit — 2026-10-13

Incremental audit for `25e82210a..upstream/master` (`21f068328`), 5 commits, completing the currently fetched upstream master tip. The previous audit is `upstream-master-audit-2026-10-12.md`.

## Summary

Two adapted ports and three rejections. `24e80bea8` adds the rebindable `Cmd+E`/`Ctrl+E` keyboard shortcut that toggles a Markdown file between rendered and raw views; it is local-only UI behavior and was ported with the workspace dispatch test adapted from upstream's `view_ancestors` (from the rejected TUI groundwork `1acbff119`) to the fork's presenter-based ancestor walk. `69c98cf99` makes `computer_use::Action::MouseDown.at` optional so a press without a coordinate lands at the pointer's current position; the retained `computer_use` crate got the enum change, the macOS actor fix, the serde helper with its tests, and the CLI `Some(pos)` update, while the fork-absent proto conversion and platform/overlay paths were omitted. `b0a86afa7` (CloudAgentExecutionConfig flag promotion), `c10fc07d7` (Warp-hosted preview-urls MCP registration), and `21f068328` (orchestration `run_agents` metadata) were rejected with zero fork anchors.

## Commit-by-Commit

### `24e80bea8` Add keyboard shortcut to toggle Markdown rendered/raw view (#16413) — adapted port

Adds a rebindable shortcut (default `cmdorctrl-e`) that swaps a Markdown file between the rendered and raw panes, as an alternative to the header Rendered/Raw control. Gated by a `MarkdownDisplayModeToggle` keymap context that `FileNotebookView` and `CodeView` insert only when the Rendered/Raw control is shown, so the keystroke falls through elsewhere; both directions share one binding name so Settings shows a single jointly rebindable row. Local-only behavior throughout. Applied via `git cherry-pick --no-commit`; ported paths:

- `app/src/notebooks/file/mod.rs` — `MARKDOWN_DISPLAY_MODE_TOGGLE_CONTEXT`, `TOGGLE_MARKDOWN_DISPLAY_MODE_*` constants, the `FileNotebookView` EditableBinding for the rendered→raw direction (`ToggleMarkdownDisplayMode(MarkdownDisplayMode::Raw)`), and the `keymap_context` override keyed on `shows_markdown_toggle()`. Verbatim upstream hunks.
- `app/src/code/view.rs` — the `CodeView` EditableBinding for the raw→rendered direction (`CodeViewAction::RenderMarkdown`, `#[cfg(feature = "local_fs")]`) and the `keymap_context` override keyed on `markdown_mode_segmented_control`. Import hunks remapped into the fork's grouped `use crate::{...}` block.
- `app/src/util/bindings_tests.rs` — `test_markdown_display_mode_toggle_is_rebound_in_both_directions`, verbatim.
- `app/src/workspace/view_test.rs` (upstream `view_tests.rs`, remapped by rename detection) — `test_markdown_toggle_shortcut_switches_between_raw_and_rendered_panes`: opens a Markdown file in the code editor, dispatches the keystroke through the focused view and its ancestors, asserts the raw→rendered→raw round trip and that a `.rs` pane leaves it unhandled.

Intentionally omitted paths and hunks:

- `app/src/notebooks/file/mod_tests.rs` — deleted at the fork baseline; the harness depends on the removed `crate::auth::AuthStateProvider`/`AuthManager` test scaffolding (`init_app` there registers the auth stack). Its two shortcut tests were dropped rather than recreated.
- Workspace-test adaptation: upstream's `ctx.view_ancestors(window_id, focused)` comes from `1acbff119` (backend-neutral view hierarchy, first PR of the rejected TUI-backend stack). The fork walks the same chain through its live dispatch equivalent: `ctx.presenter(window_id).borrow().ancestors(focused)` (the exact call the fork's real `KeyDown` path makes). `use warpui::EntityId;` and a `FileModel::new` singleton registration were added for the fork's trimmed test harness.
- Upstream-only context tests that the rename-diff pulled into the conflict (`test_open_file_notebook_focuses_existing_markdown_pane`, `test_open_file_with_target_event_preserves_requested_line`, `test_open_markdown_viewer_target_preserves_requested_line`) belong to unported upstream commits (they exercise `CodeSource::Link` line preservation and focus-reuse behavior not ported with them) and were not merged.

### `69c98cf99` Allow MouseDown at the current pointer position (#16431) — adapted port

`computer_use::Action::MouseDown.at` becomes `Option<Vector2I>`: `None` presses at the pointer's current position, matching Anthropic's `computer_toolset_20260801` drag encoding (`mouse_move`/`left_mouse_down` without a press coordinate). The `computer_use` crate is a retained workspace member (`app` depends on it for the AgentView computer-use execution path). Applied via `git cherry-pick --no-commit`; ported paths:

- `crates/computer_use/src/lib.rs` — `at: Option<Vector2I>` with `#[serde(default, with = "option_vector2i")]`; the `option_vector2i` wrapper module preserving the `{"at": {"x": .., "y": ..}}` wire shape; `#[cfg(test)] mod action_serde_tests;`.
- `crates/computer_use/src/action_serde_tests.rs` — round-trip with/without a position plus legacy-shape and omitted-position deserialization, verbatim.
- `crates/computer_use/src/mac/mod.rs` — the Actor `perform` arm skips `move_to` for `None` and presses at the current pointer. (The `remap_action_for_target` hunk inside the same file belongs to the unported window-targeting stack and had no anchor.)
- `crates/computer_use/src/bin/use_computer.rs` — Click coordinates become `Some(pos)`; the fork keeps its `button.clone()` idiom because upstream's `Copy` derive on `MouseButton` (which makes the move-then-reuse compile) is from an unported commit.
- `crates/computer_use/Cargo.toml` / `Cargo.lock` — `serde_json` dev-dependency for the new tests; the lock gains only the `computer_use` → `serde_json` edge (same minimal shape as upstream).

Intentionally omitted paths and hunks:

- `crates/ai/src/agent/action/convert.rs` — fork-absent; the proto `UseComputer` → `computer_use::Action` conversion lives in the removed server-API layer, and the fork's `crates/ai` has no `RunAgents`/convert surface for it.
- `crates/computer_use/src/linux/**`, `windows/**`, `overlay_tests.rs`, `recording_tests.rs` — deleted at the fork baseline (local Linux/Windows actors and the recording/overlay stack); `pointer_session_tests` module declaration dropped for the same reason.
- `[build-dependencies] cfg_aliases` context hunk — the fork's crate has no `build.rs`.

### `b0a86afa7` Enable CloudAgentExecutionConfig on stable (#16429) — rejected (removed surface, zero anchors)

Promotes the `cloud_agent_execution_config` cargo feature into app defaults and moves `FeatureFlag::CloudAgentExecutionConfig` from dogfood to stable runtime promotion. Cloud agent execution configuration is the removed cloud-agent/Oz execution surface: the fork has no `cloud_agent_execution_config` feature, no `app/src/features.rs` bridge, and no `CloudAgentExecutionConfig` flag variant (verified: zero hits across `app`, `crates`, `Cargo.toml`). Nothing separable — the commit is the promotion itself.

### `c10fc07d7` feat: register preview url MCP (#16372) — rejected (Warp-hosted MCP, zero anchors)

Registers the Warp-hosted preview-urls MCP server (`/api/v1/mcp/preview-urls`) as a built-in alongside Factory MCP, sharing the built-in installation construction in `agent_sdk/driver/mcp_startup.rs` and `ai/mcp/builtin.rs`. Both client surfaces are fork-absent (`app/src/ai/agent_sdk/`, `app/src/ai/mcp/` removed); MCP configuration belongs to the ACP agent process in this fork, and the server endpoint is a Warp-hosted API. The `preview_urls_mcp` cargo feature and `PreviewUrlsMcp` flag have zero fork anchors (`factory_mcp`/`well_known_mcp_ids` are likewise absent).

### `21f068328` Pass run_agents metadata through child launch and confirmation (#16396) — rejected (orchestration stack, zero anchors)

Threads `run_agents` batch metadata and child overrides through protocol conversion, `RunAgentsExecutor` dispatch, remote request preparation, GUI/TUI confirmation, child-agent launch (`RunAgents(Box<RunAgentsRequest>)` boxing), and managed-secret collision rejection. Every anchor is fork-absent: `RunAgents`/`RunAgentsRequest`/`RunAgentsExecutionMode` (no hits in `crates/ai` or `app/src`), `app/src/ai/agent_sdk/**`, `ambient_agents/**`, `orchestration/remote_child.rs`, `server/server_api/ai.rs`, `crates/graphql/**`, `crates/warp_tui/**` are all removed surfaces, and the shared-file hunks in `app/src/terminal/view.rs`, `app/src/pane_group/pane/terminal_pane.rs`, `start_agent.rs`, and `action_model.rs` edit `RunAgents`/`launch_local_*_child`/`StartAgentRequestId` symbols that do not exist in the fork (verified zero hits). The warp-proto-apis pin bump rides on the same orchestration protocol and has no local consumer.

## Verification

- `cargo check -p warp --all-targets --message-format short`: clean (pre-existing warnings only).
- `cargo check -p computer_use --all-targets --message-format short`: clean.
- `cargo check --workspace --all-targets --message-format short`: clean.
- `cargo fmt -- --check`: clean.
- `cargo nextest run -p warp -E 'test(markdown_toggle) | test(markdown_display_mode_toggle)'`: 2/2 passed.
- `cargo nextest run -p computer_use`: 2/2 passed (new serde tests).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: 156/156 passed.
- `cargo build -p warp --all-targets --message-format short`: succeeded; `cargo clean` follows after push per the release workflow.
- Restored-deleted-surface scans (`rg` over `app crates script Cargo.toml`): 232/0/6 hits — identical to the previously documented allowances (the six platform hits remain the retained SSH `ForwardX11` options, zsh bootstrap ConPTY comments, and `local_or_remote_path_tests.rs` `cfg(windows)` remote-path tests); zero hits in the files touched by these ports.
