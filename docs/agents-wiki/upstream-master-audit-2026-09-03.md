# Upstream Master Audit 2026-09-03

## Scope

- Current fork before this audit: `6a5db8885` (`main`, `v2026.09.02`).
- Upstream source reviewed: `db6ab73056..upstream/master` (4 commits, tip `b9c21aa01f`).
- Result: two accepted/adapted feature ports, one separable UI improvement port, and one rejected marketing commit. The PR-branch prototype `3ac9c7b7f6` (ctrl-r fzf/atuin handoff) was identified as not an ancestor of master; `bf2364bc9` is the self-contained authoritative implementation.

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `4fa1a3c66` | History search: rank on history priors, not fuzzy score alone (APP-5650) (#15591) | **Adapt (ported)** | See port record below. |
| `f349ffe8b` | Add Warp Factories early access section to README (#15534) | **Reject** | Upstream README marketing for Warp Factories (Warp-hosted cloud agent infrastructure, build.warp.dev dashboard, Oz-credits successor). Pure cloud-product marketing; the fork README is fork-owned and must not track it. |
| `bf2364bc9` | Shell widget handoff: hand ctrl-r and ctrl-t to fzf/atuin (CORE-3807) (#15513) | **Adapt (ported)** | See port record below. |
| `b9c21aa01` | Support stored screenshot references in computer-use tasks (#15587) | **Reject core / adapt lightbox** | Core is a Warp-server feature: `warp_multi_agent_api::StoredScreenshotRef` (crate absent here), `stored_screenshots.rs` downloads bytes from Warp-managed object storage through the removed `server/server_api` AIClient, server-issued task-message updates swap inline screenshot bytes for stored refs, `StoredScreenshots` rollout flag, and the `warp_multi_agent_api` rev bump. The `UseComputerResult::Success` struct-variant refactor and `ScreenshotSource` enum exist only to model the Stored variant and were not ported. The separable lightbox `CurrentImageState` improvement was ported (see below). |

## Port record: `4fa1a3c66` history search ranking

### Runtime-ownership review

Purely local: Ctrl+R / Command Search history ranking over `HistoryEntry`
priors (recency, session, exit status) plus fzf-style whitespace AND
tokenization, all computed on-device against local history. The feature flag
is wired default-on through the fork's cargo-feature pipeline
(`history_search_ranking_v2` in `app/Cargo.toml` `default` + the
`app/src/lib.rs` `enabled_features()` bridge), not upstream's
`PREVIEW_FLAGS` rollout. Disabling the flag still returns to the exact
legacy `fuzzy_match_history_legacy` path.

### Applied from the exact upstream source

- `app/src/search/command_search/history/rank.rs` + `rank_tests.rs`: verbatim (all constants, `MatchQuality`, `tokenize_query`, `match_history_command`, `rank`); only `crate::terminal::model::session::SessionId` → `warp_core::SessionId`.
- `history_data_source.rs`: live per-query candidate rebuild, `CHUNK_SIZE`, flag gate, legacy fallback.
- `history_search_item.rs`: `score` field; `mod.rs`: `mod rank`.
- `view.rs`: both `history_data_source_for_session(session_id)` call sites (source creation + `HistoryEvent::Initialized` re-registration).
- `searcher_test.rs` (fork's singular filename): `FixedResults`, live-exit-status test, blank-query chronological-order test, exact-vs-prefix ordering update, flag-off legacy tests, expected zero-state order updates.

### Intentionally omitted paths (with reasons)

- `app/src/terminal/history.rs` (`CommandHistorySummary` removal and `persisted_commands_summary` type simplification): upstream deleted the count data because its only consumer (the WelcomePalette-adjacent paths) was removed upstream in `2735ae10a2` (#12614). This fork retains `app/src/search/command_search/projects/suggested_projects_data_source.rs`, which consumes `History::command_summaries()` counts as project popularity. The ranking feature does not read this map, so the fork keeps `CommandHistorySummary`.
- Notebooks-removal hunks (`command_search/notebooks/` deletion, `mod notebooks;`, `AcceptNotebook` in searcher.rs/workspace view, telemetry `Notebook` variant, `notebooks:` sample chip, WarpDriveSettings filter split, `notebook_raw_text_shared`): all anchors are absent here — this fork's Command Search never had the notebooks data source. Only the stale `valid_query_filters` doc-comment removal applies and was ported.
- `app/src/server/telemetry/events.rs`: telemetry crate removed.
- Cross-source scale test adaptation: the upstream test's `WorkflowSearchItem`/saved-prompt competitors (and `crate::search::command_search::workflows`, `WorkflowIdentity`, `WorkflowType` imports) are absent because the Command Search workflow sources were removed with Warp Drive cloud. The ported test keeps the AI prompt-history competitor (`AIQuerySearchResultItem`), which is the other source the fork's Command Search mixer actually registers, and asserts `results.len() == 2`.
- Test `History::handle` needs the `warpui::SingletonEntity` import in the fork's test file.

## Port record: `bf2364bc9` shell widget handoff

### Runtime-ownership review

Purely local shell-integration feature: bootstrap scripts detect whether
ctrl-r/ctrl-t are bound to fzf/atuin widgets, Warp hands the keypress to a
bootstrap helper that runs the widget as a foreground command, and the
selection returns via the `ExternalShellWidgetSelection` DCS hook into the
input editor (replace for ctrl-r, cursor-splice for bash/zsh ctrl-t,
fish's real `fzf-file-widget` output for fish). No Warp service dependency.
`FeatureFlag::ShellWidgetHandoff` is wired default-on through the fork's
cargo-feature pipeline instead of upstream's Preview rollout.

### Applied from the exact upstream source

- `zsh_body.sh`: both helper functions, zshaddhistory filter, bindkey detection (clean apply).
- `bash_body.sh`: helpers, HISTIGNORE patterns, `bind -X` extraction (accepting bash 5.3's space separator), atuin init-flag fallback.
- `fish.sh`: widget reporting helpers, ctrl-r/ctrl-t runners with fd-3 swap, `fish_should_add_to_history` composition, `shell_plugins` detection.
- `dcs_hooks.rs`/`dcs_hooks_tests.rs`/`handler.rs`/`ansi/mod.rs`/`mod_test.rs`: `ExternalShellWidgetSelection` hook end to end, with the upstream parse/dispatch tests.
- `event.rs`/`model_events.rs`/`terminal_model.rs`: `Event`/`ModelEvent` variants, redacting `Debug` impl, `Handler` dispatch.
- `blocks.rs`: `hide_block`; `tab_metadata.rs`: `!block.is_hidden()` tab-title guard.
- `input.rs`: `PendingShellWidgetHandoff`, `ShellWidgetApplyMode`, the ctrl-t `EditableBinding`, `set_external_shell_widget_selection`, `trigger_external_shell_widget_handoff`, the `should_add_command_to_history` threading through `try_execute_command_from_source`/`start_block_and_write_command_to_pty`, and the block-completed restore branch.
- `terminal/view.rs`: plugin-tag constants, `maybe_trigger_external_ctrl_r_history_search`, `maybe_trigger_external_ctrl_t_file_search`, `ModelEvent::ExternalShellWidgetSelection` handling, `write_user_bytes_to_pty` visibility.
- `workspace/view.rs`: ctrl-r handoff gate in `show_command_search`, `trigger_external_ctrl_t_file_search`, the `WorkspaceAction::TriggerExternalCtrlTFileSearch` arm.
- `workspace/view_test.rs`: ctrl-t pty-forwarding test (clean apply).

### Fork adaptations

- Path remapping per change-map's pre-split rule: `crates/warp_terminal/src/model/ansi/**` → `app/src/terminal/model/ansi/**`; `input_tests.rs` → `input_test.rs`, `view_tests.rs` → `view_test.rs`, `mod_tests.rs` → `mod_test.rs` (fork's singular test filenames).
- Flag wiring through `app/src/lib.rs` + `app/Cargo.toml` default (the fork has no `app/src/features.rs` bridge; `features.rs` is a re-export).
- `Event`/`ModelEvent` variants inserted without the removed `FinishUpdate` neighbors; the terminal-model handler uses the fork's `send_terminal_event` (upstream: `send_app_event`).
- `hide_block` anchored before the fork's de-Oz-renamed `is_executing_agent_environment_startup_commands` (upstream anchor `is_executing_oz_environment_startup_commands` is absent).
- `bash_body.sh`: the `WARP_IN_MSYS2` gate is dropped (fork removed MSYS2; an unset variable would disable detection entirely). Detection runs unconditionally.
- `fish.sh`: Bootstrapped JSON gains `shell_plugins` at upstream's position but without upstream's `wsl_name` (fork removed WSL payload fields).
- Shared-session-only code omitted with the removed surface: `try_execute_command_on_behalf_of_shared_session_participant`, `try_execute_command_with_options`, and the `is_readonly_shared_session_active` guard in `trigger_external_ctrl_t_file_search`. The retained `try_execute_command`/`execute_queued_command`/prompt-chip call sites pass `should_add_command_to_history = true` directly, matching upstream's behavior for those paths.
- `workspace/action.rs`: only the `TriggerExternalCtrlTFileSearch` variant and predicate-list arm ported. The commit's removal of doc comments on `RenameActivePane`, `SetActiveTabColor`, `ToggleTabSelectionRightClickMenu`, `CancelActiveRename`, `ClearTabMultiSelection`, `ToggleTabGroupColor`, and `OpenWarpDrive` is an upstream rebase artifact (the PR branch predated `d2f26ae9b` #9712 and `b24fce3db` #12229 comments); this fork ported those comments with their features and keeps them.
- `input_test.rs`: the new tests manipulate the fork's `latest_input_block_id` gate where upstream manipulates `deferred_remote_operations.latest_block_id` (absent mechanism), add the fork-only `block_latency_data: None` field, import `SerializedBlock`, and are appended at file end (upstream's mid-file anchor test has a different fork counterpart). All 12 new handoff tests pass.
- A 3-way-merge artifact (`use crate::terminal::keys::TerminalKeybindings;` materialized from upstream context into `terminal/view.rs`) was removed as unused.

## Port record: `b9c21aa01` lightbox load state (separable part)

- `crates/ui_components/src/lightbox.rs`: `CurrentImageState` enum (Loading/Loaded/Failed), `Params.current_image_state`, Failed-to-load rendering branch; the ported code includes upstream's `.layout_using_paint_bounds()` call, available in the fork's `warpui_core` `Image`.
- `crates/ui_components/examples/library.rs` and `app/src/workspace/lightbox_view.rs`: state computation from the asset cache (`AssetState::FailedToLoad` → Failed).

Generic retained UI behavior: a failed image fetch now renders "Failed to
load image" instead of an endless spinner. All other paths of the commit are
rejected (see triage table).

## Verification

- `cargo check -p warp --all-targets --message-format short`: pass (no new warnings).
- `cargo check --workspace --all-targets --message-format short`: pass.
- `cargo fmt` / `cargo fmt -- --check`: clean.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: 156 passed.
- `cargo nextest run -p warp -E 'test(rank) + test(command_search) + test(searcher_test)'`: 37 passed; rank/mixer tests 12 passed.
- `cargo nextest run -p warp -E 'test(ctrl_r_handoff) + test(ctrl_t_handoff) + test(ctrl_t_apply_mode) + test(ctrl_t_binding) + test(shell_widget_handoff_selection) + test(parse_dcs_external_shell_widget_selection) + test(every_hook_tag_dispatches) + test(ctrl_t_action_forwards_to_pty)'`: 18 passed.
- `cargo nextest run -p warp -E 'test(terminal::history) + test(input) + test(ansi)'`: 352 passed.
- `cargo build -p warp --all-targets --message-format short`: pass. `cargo clean` after the release push.
- Deletion-surface scans: no new hits; remaining matches are the pre-existing allowed set (`WeakHandle::upgrade` method names, ONNX tokenizer vocabulary, retained SSH `ForwardX11=no`, retained remote-path `#[cfg(windows)]` tests, bootstrap ConPTY comments).
- `bash -n` on `bash_body.sh`: clean.
- `CARGO_PROFILE_DEV_DEBUG=0` used for check/test/build as in prior audits.
