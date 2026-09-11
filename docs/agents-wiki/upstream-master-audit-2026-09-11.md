# Upstream Master Audit 2026-09-11

Range under review: `d732aa5fc..upstream/master` (11 commits)

Previous audited upstream tip: `d732aa5fc Fail Sentry DIF uploads loudly after bounded retries (#15819)`

Current upstream tip detected: `6f575836c fix(workspace): keep a new tab group's terminal from eating its name (#14895)`

Total upstream commits in this incremental range: 11

Status: triage complete. Three adapted ports landed on `merge/upstream-2026-09-11`:

- `6f575836c` — full port (tab-group rename focus fix) minus the `FeatureFlag::GroupedTabs` guards this fork never had.
- `e6e1c4e0f` — framework-only port of the `WindowBackdrop` migration (warpui_core/warpui API plus the `reopen_closed_window` blur-radius fix), omitting the Windows-only setting/UI/migration machinery and the absent winit backend.
- `77105899b` — test hunk only (alias expansion test pins `NativeShellCompletionsEnabled` off); the Cargo default promotion and `PREVIEW_FLAGS` pruning were already reflected in this fork.

One pre-existing local test failure was also fixed (`AcpRegistryModel` missing from `initialize_app_for_terminal_view`).

## Per-Commit Triage

### `352d1ea44` — Add join another team chooser for native workspaces (#15881)

Decision: **reject** (Teams/workspace discovery, no anchors). Core paths are absent: `app/src/settings_view/join_teams_modal.rs` + `teams_page*` (fork has no teams page), `app/src/server/server_api/team.rs` (no `app/src/server/`), `app/src/workspaces/**` (no `workspaces` tree), `crates/graphql/**` mutations/queries (removed), `crates/warp_graphql_schema/api/schema.graphql` (removed). The single fork-present file, `app/src/integration_testing/assertions.rs`, has no anchor symbols (`join_a_workspace`, `billing_metadata`, `stripe_customer_id` → zero fork hits); the hunk only adds an `open_teams` field to a mock workspace-metadata struct the fork's assertions tree never carried. All other touched test files (`blocklist/controller_tests.rs`, `geap_credentials_tests.rs`, `command_palette/data_sources_tests.rs`, `billing_and_usage_dispatch_tests.rs`, `update_environment_form_tests.rs`, `terminal/input/prompts/data_source_tests.rs`) are absent.

### `dff3d5669` — [REV-2395] Enable native workspace team leave (#15880)

Decision: **reject** (Teams). Every touched path is absent: `app/src/drive/cloud_action_confirmation_dialog*.rs`, `app/src/settings_view/teams_page*.rs`, `app/src/workspaces/user_workspaces/*`. Native-workspace team membership management is account/workspace-discovery surface removed by contract.

### `50118d9f1` — Preserve team-scoped credentials through orchestration launches (#15863)

Decision: **reject** (removed orchestration/team-scope stack, no anchors). Substantive changes live in absent paths: `action_model/execute/{run_agents,start_agent}.rs`, `inline_action/run_agents_card_view.rs`, `ai/document/orchestration_config_block.rs`, `ai/orchestration/**`, `pane_group/child_agent/`, `app/src/server/team_scope.rs`, `app/src/workspaces/**`, `crates/warp_tui/**` (no warp_tui in fork), `tui_export.rs`/`tui_test_support.rs`. The two fork-present files (`blocklist/action_model.rs`, `blocklist/mod.rs`) only re-export `TEAM_CHANGED_DURING_CHILD_LAUNCH_ERROR` from `execute/run_agents.rs`, which does not exist here (`rg 'TEAM_CHANGED_DURING_CHILD_LAUNCH_ERROR|run_agents' app/src` → no hits).

### `77105899b` — Promote shell features to default (#15903)

Decision: **adapt** (test hunk ported as `308f9fd58`→final `47cdb1951` lineage on the merge branch). The fork's `app/Cargo.toml` default list already contains `native_shell_completions`, `shell_widget_handoff`, and `history_search_ranking_v2` (promoted locally when the shell features were ported), and the fork's `PREVIEW_FLAGS = &[FeatureFlag::MarkdownTables]` never listed them, so the Cargo/`warp_features` hunks are no-ops here. The `crates/integration/src/test.rs` hunk matters: `test_alias_expansion_has_limit` now pins `NativeShellCompletionsEnabled::storage_key() = false` via user defaults so the test keeps asserting Warp's alias-expansion limit rather than the shell's. Ported with the fork's import layout (`use warp::settings::{CtrlTabBehavior, NativeShellCompletionsEnabled};`).

### `e6e1c4e0f` — Add Windows backdrop material dropdown (#15879)

Decision: **adapt** (framework portion ported; see below for the omitted Windows-only surface). Although the feature is Windows DWM backdrop selection, the commit also migrates the shared windowing API from the Windows-oriented `background_blur_texture: bool` to `platform::WindowBackdrop`, and — the macOS-relevant fix — `AppContext::reopen_closed_window` now takes `background_blur_radius_pixels`/`background_backdrop` from callers instead of hardcoding `None`/`false` behind a `TODO(vorporeal)`, so undo-close reopened windows keep the configured blur radius.

Ported paths (exact upstream hunks, conflicts resolved on the applied source):

- `crates/warpui_core/src/platform/mod.rs` — `WindowBackdrop` enum (+`ALL`, `settings_value` impl), `WindowOptions.background_blur_texture → background_backdrop`, Debug field rename, default-no-op `Window::set_background_backdrop`, removal of the `set_all_windows_background_blur_texture` trait method. Conflict: upstream inserted the enum after a wasm-gated `KEYS_TO_IGNORE` block; the fork keeps its own non-wasm `KEYS_TO_IGNORE` and drops the wasm block.
- `crates/warpui_core/src/core/mod.rs` — `AddWindowOptions` field rename.
- `crates/warpui_core/src/core/app.rs` — `open_window` plumbing + `reopen_closed_window(data, background_blur_radius_pixels, background_backdrop)`.
- `crates/warpui_core/src/windowing/state.rs`, `crates/warpui_core/src/platform/test/delegate.rs`, `crates/warpui/src/platform/mac/window.rs` (both WindowManager impls), `crates/warpui/src/platform/headless/windowing.rs` — removal of the blur-texture no-ops.
- `app/src/undo_close/stack.rs` — reopen passes `Some(*window_settings.background_blur_radius)` (the macOS fix) and, adapted, `WindowBackdrop::None`.
- `app/src/root_view.rs` — six call sites renamed; the fork's previous hardcoded `background_blur_texture: false` becomes `WindowBackdrop::None` (exact behavioral equivalent), and the `open_from_restored` tuple now carries the settings-backed blur radius plus `WindowBackdrop::None`.

Omitted paths and reasons:

- `app/src/window_settings.rs` — `BackgroundBackdrop` setting and `LegacyOverrideBlurTexture` retention are `SupportedPlatforms::WINDOWS` and were never carried by this fork (no `background_blur_texture` setting exists here); the migration helpers (`stage_legacy_background_backdrop`, `migrate_legacy_background_backdrop`) and `warp_errors::report_if_error!` plumbing have no fork counterpart (fork removed `warp_errors`).
- `app/src/lib.rs` — the migration hook rides `CloudPreferencesSyncerEvent::InitialLoadCompleted`; the fork has no cloud preferences syncer.
- `app/src/settings/init.rs`, `app/src/settings_view/mod.rs` (`WINDOW_BLUR_TEXTURE_FLAG`), `app/src/settings_view/appearance_page.rs` (Windows dropdown UI; the fork's appearance page has no acrylic/backdrop control), `app/src/workspace/view.rs` (`WindowSettingsChangedEvent::BackgroundBackdrop` handler) — all depend on the Windows-only setting that does not exist here.
- `crates/warpui/src/windowing/winit/window.rs` — no winit windowing backend in this fork.

Where upstream reads `*window_settings.background_backdrop`, the fork passes `WindowBackdrop::None` — the exact equivalent of the previous hardcoded `false` — since the macOS `set_background_backdrop` is the trait's default no-op.

### `b61e936f4` — [REV-2214] Add workspace removal to team member actions (#15905)

Decision: **reject** (Teams/workspace management via GraphQL). All touched paths absent: `drive/cloud_action_confirmation_dialog*`, `server/server_api/workspace.rs`, `settings_view/teams_page*`, `workspaces/user_workspaces/*`, `crates/graphql/**`, `crates/warp_graphql_schema/**`.

### `3ec493c14` — [REMOTE-3146] Discover nested build cache roots (#15844)

Decision: **not applicable**. `crates/build_cache/` does not exist in this fork (Warp build-infrastructure crate, never carried), and `Cargo.lock` changes only track that crate's dependency graph. `specs/REMOTE-3146/TECH.md` is rejected by contract (upstream specs).

### `c12e2130b` — Fix duplicate requests on shared session load (#15829)

Decision: **reject** (server-API shared-session paths, no anchors). The dedup logic (`in_flight_server_metadata_fetches`) wraps `ServerApiProvider`/`list_ai_conversation_metadata` calls that do not exist in the fork's `history_model.rs` (`rg 'list_ai_conversation_metadata|set_server_metadata_for_conversation|in_flight_conversation_renames' app/src/ai/blocklist/history_model.rs` → zero hits — the fork's history model has no server metadata fetch at all). `orchestration_event_streamer*`, `terminal/shared_session/viewer/**`, and `script/wasm/bundle` are likewise absent.

### `2012bacff` — Remove Warp-managed Grok plugin installation (#15914)

Decision: **not applicable**. The fork is pre-`plugin_manager` lineage: `app/src/terminal/cli_agent_sessions/plugin_manager/**` never existed here, and the fork's Grok support (ported from upstream `2718b6658`) is listener/OSC9-based with no Warp-managed plugin installation path. Upstream's semantic — Grok no longer gets a Warp-managed installer — already holds trivially in this fork. `specs/GH11727/**` rejected by contract.

### `437b862e4` — [QUALITY-2068] Fix macOS computer-use recording output frame rate (#15920)

Decision: **not applicable**. `crates/computer_use/src/mac/recording.rs` and `recording_tests.rs` do not exist in the fork: the SCStream recording feature (cloud-artifact computer-use recordings) was never carried; the fork's `computer_use` crate is keyboard/mouse/screenshot utilities only. `app/src/ai/blocklist/orchestration_event_streamer_tests.rs` is also absent.

### `6f575836c` — fix(workspace): keep a new tab group's terminal from eating its name (#14895)

Decision: **accept** (ported in full, adapted for fork test layout). Creating a tab group opens the inline name editor and spawns a terminal; when the terminal's bootstrap block became visible it stole focus, blurring the editor — and blur was treated as confirmation, persisting a half-typed group name and sending the trailing keystrokes to the shell. Two layers, both ported:

- `TerminalView::handle_terminal_event` skips `focus_terminal` on `ModelEvent::VisibleBootstrapBlock` while a tab/tab-group rename editor is focused, read via `WorkspaceRegistry::as_ref(ctx).get(self.window_id, ctx)` → `Workspace::is_inline_rename_editor_focused`.
- `Workspace::handle_tab_group_rename_editor_event` now treats `Blurred` as discard (cancel) like `Escape`; `Enter` remains the only commit path. Tab and pane rename keep commit-on-blur, as upstream.

Fork adaptations:

- Dropped both `FeatureFlag::GroupedTabs.override_enabled(true)` guards (tests and none else) — tab grouping is unconditional in this fork; the flag does not exist in `crates/warp_features`.
- Tests live in `view_test.rs` (fork naming): `test_tab_group_rename_blur_does_not_commit_unfinished_name` in `workspace/view_test.rs`, and the two `visible_bootstrap_block_leaves_focus_on_*` tests in `terminal/view_test.rs`.
- `workspace::view::tests` module is now `pub(crate)` with `pub(crate) fn initialize_app`/`mock_workspace`, matching upstream's visibility so `terminal/view_test.rs` can import them as `crate::workspace::view::tests::{initialize_app as initialize_workspace_app, mock_workspace}`.
- `terminal/view.rs` import conflict resolved into the fork's existing `use crate::workspace::{CommandSearchOptions, ToastStack, WorkspaceAction, WorkspaceRegistry};` (upstream's import block layout differs).

## Additional Fix (Not Upstream)

`app/src/test_util/terminal.rs` — `initialize_app_for_terminal_view` now registers `AcpRegistryModel::new_for_test`. Pre-existing failure on `main`: `terminal::view::tests::inline_agent_view_persists_across_transfer_takeover_for_monitored_long_running_command` panicked with "Cannot get singleton model AcpRegistryModel that was never registered" after the agent-view controller started consulting the ACP registry; every other test initializer (`terminal/input_test.rs`, `workspace/view_test.rs`) already registers it. Verified the failure reproduces on `main` before fixing on the branch.

## Verification

- `cargo check -p warp --all-targets --message-format short` — pass after each port (`CARGO_PROFILE_DEV_DEBUG=0`).
- `cargo check --workspace --all-targets --message-format short` — pass (framework migration check).
- `cargo fmt -- --check` — pass (after folding rustfmt import-order fixes into their owning commits).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- New/touched suites: the three new regression tests pass; `test(terminal::view::tests) + test(workspace::view::tests)` — 173 passed (after the `AcpRegistryModel` fix).
- `cargo build --workspace --all-targets --message-format short` — pass (workspace build, required because the WindowBackdrop port is a warpui_core framework migration), followed by `cargo clean`.
- Deleted-surface scans re-run over the changed-file set — zero new hits; full-tree scans show only the same allowed hits as the 2026-09-10 audit (doc wording, retained SSH remote strings, wasm-gated test cfg).

## Notes

- Upstream is mid-flight on native-workspace multi-team management (REV-2395/REV-2214/APP-5806); expect more `workspaces/**` + `graphql/**` commits that will keep landing in reject.
- The `WindowBackdrop` API is now upstream-shaped in this fork even though the enum's materials are Windows-only; if upstream later adds a macOS backdrop variant, the setting/UI port should start from that future commit, not from `e6e1c4e0f`'s Windows dropdown.
- Deferred ports unchanged: `9921300b7` (Ctrl-C harness cancel, waiting on upstream local-keystroke wiring) and the mermaid toggle from `#10431`.
