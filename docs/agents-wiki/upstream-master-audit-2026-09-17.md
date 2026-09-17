# Upstream Master Audit 2026-09-17

Range under review: `0eefb7404..upstream/master` (5 commits)

Previous audited upstream tip: `0eefb7404 Skip the personal runs seed fetch for service-account principals (#16032)`

Current upstream tip detected: `98551490f6 Move team switcher beside profile avatar (#16041)`

Total upstream commits in this incremental range: 5

Status: triage complete. One adapted port (`221eda318`); four commits rejected or not applicable.

## Per-Commit Triage

### `0bb920b28` — Fix child-agent pill crashes after reopening closed tabs (#15996)

Decision: **N/A (reject)**. The crash fix protects the child-agent pill / orchestration pane lifecycle, which is fork-absent:

- `app/src/pane_group/mod.rs` hunks anchor on `child_agent_panes`, `pending_child_hydrations`, `pending_remote_child_hydrations`, `child_agent_origin`, `failed_viewer_child_sessions`, `swap_active_pane_to_conversation`, `pane_id_for_conversation_owner`, and `terminal_view_id_for_owned_conversation` — none exist in the fork's pane_group (verified by search; the fork's pane_group is ~5.3k lines vs upstream ~7.9k with the orchestration surface removed).
- `app/src/ai/blocklist/history_model.rs` adds `clear_conversations_for_closed_terminal_surface` over `live_conversation_ids_for_terminal_surface`/`active_conversation_for_terminal_surface` fields the fork does not have (fork's history model uses `terminal_view_id`-keyed maps and `clear_conversations_in_terminal_view`).
- `app/src/ai/active_agent_views_model.rs` adds `is_terminal_view_attached`, whose only caller is the fork-absent `terminal_view_id_for_owned_conversation` filter.
- The new `orchestration_navigation` integration-test files exercise that removed surface.

No separable generic fix: the fork's close path (`clear_conversations_in_terminal_view`) has no transferred-conversation child-pill flow to protect, and `reattach_panes` has no child-agent panes to re-remove.

### `91dad6a70` — [APP-5720] Conversation usage popover in the agent input footer (#15946)

Decision: **reject (N/A)**. Display side of the APP-5720 server-authored usage/cost stack already rejected in the 2026-09-16 audit (`acb96e6e8`, `7b7f4f7c2`):

- New `app/src/ai/blocklist/usage/usage_popover_view.rs` (+1823 lines) imports `warp_multi_agent_api as api` and `persistence::model::ChargedUsageTotals` (both fork-absent), folds `Message.RequestMetadata` charges, and reacts to `BlocklistAIHistoryEvent::ConversationUsageMetadataUpdated` (fork-absent variant). ACP conversations in this fork produce no `RequestMetadata` records, so the popover would be dead code with no producer.
- Framework hunks are consumed only by this popover: `Element::debug_child_view_ids` forwarding impls in `warpui_core` stack elements, `ActionButton::tooltip_for_test`, and `Icon::PieChart` → `bundled/svg/turn-usage-pie.svg` (a usage/billing asset that would need bundling). Porting them would add unused test-util surface with no fork consumer.

### `2f0db5c5e` — Fix steering delivery for follow-ups with file attachments (#16046)

Decision: **reject**. Bug fix for the steering-mode queued-prompt delivery introduced in #15921 (`af7d930a5`), rejected in the 2026-09-16 audit. All machinery is fork-absent (verified by search): `controller/startup_queue.rs`, `QueuedQueryOrigin::SharedSessionInjection`, `QueuedPromptDeliveryMode`, `ready_head`/`ready_query`/`PromptReady` readiness, `FileReference`, `crates/warp_tui/`, `app/src/ai/agent_sdk/driver.rs`. The fork's retained local Queueing mode (`/queue`, auto-queue, LRC auto-queue) has no attachment downloads or steering dispatch, so there is no separable local fix.

### `221eda318` — Split is_headless into is_gui/is_headless and rename AppBackend::Headless to Windowless (#16034)

Decision: **adapt/port**. Framework-level predicate split plus rename with no behavior change; keeps the fork aligned with upstream warpui naming (`new_windowless`, `Delegate::is_gui`, `AppContext::is_gui`) that future upstream commits will use.

Applied with `git diff 221eda318^ 221eda318 -- <retained paths> | git apply --3way`, then conflict resolution on the applied source. Retained paths:

- `app/src/lib.rs`: new `LaunchMode::is_gui()` predicate; `should_start_local_http_server`, the `run_internal` app-backend choice (restructured to upstream's `is_gui() → new / else new_windowless`), the Dock-visible setup gate, and `initialize_app`'s `set_app_icon` gate now use `is_gui()`; the Dock-setup comment from upstream is included.
- `app/src/terminal/terminal_manager.rs`: `compute_block_size` flips to `if ctx.is_gui() { font-based } else { hardcoded }` with upstream's windowless-backend comment.
- `crates/warpui/src/platform/app.rs`: `AppBackend::Headless` → `AppBackend::Windowless`, `new_headless` → `new_windowless`, enum doc updated.
- `crates/warpui/src/platform/mac/app.rs`: the five `AppBuilderExt` match arms renamed (applied cleanly).
- `crates/warpui/src/platform/headless/delegate.rs`: `fn is_headless() -> true` → `fn is_gui() -> false`.
- `crates/warpui_core/src/core/app.rs`: `AppContext::is_headless` → `AppContext::is_gui` (applied cleanly).
- `crates/warpui_core/src/platform/mod.rs`: `Delegate::is_headless` (default `false`) → `Delegate::is_gui` (default `true`) with upstream's doc.

Intentionally omitted paths/hunks and reasons:

- `crates/warpui/src/platform/linux/mod.rs` — fork-absent (Linux platform removed).
- `crates/warpui/tests/headless_main_thread.rs` — fork-absent (no warpui tests directory).
- `AppBuilder::enable_headless_microphone_access_query` → `enable_windowless_microphone_access_query` — the fork never had the microphone-query machinery (voice input removed); the headless `App` has no `enable_microphone_access_query`.
- `app/src/lib.rs` microphone gate (`if !launch_mode.is_headless() { ... }`) — fork-absent, and it is upstream's only remaining `LaunchMode::is_headless()` caller.
- `LaunchMode::is_headless()` predicate itself — omitted: the fork's `LaunchMode` has no `Tui`/`CommandLine` variants, so `is_headless()` would exactly complement `is_gui()` and, with the microphone gate absent, would be uncalled dead code. Restore it if a launch mode that distinguishes the two predicates ever lands.
- `app/src/lib.rs` APP-2946 background-only block (`mark_process_as_background_only` + `platform::windows::check_redirection_guard`) — fork-absent (never ported; Windows platform removed). The port keeps upstream's comment above the Dock-setup gate, which is accurate for the fork.
- `app/src/lib.rs` TUI-callbacks restructure (`TelemetryCollector` flush / `crash_reporting::uninit_sentry` / two-arg `app_callbacks` with `tracing_initialization`) — fork-absent (TUI, telemetry, crash reporting removed); the fork keeps its single-arg `app_callbacks(launch_mode.is_integration_test())` in both branches.
- `is_gui()` match arms reduced to the fork's four `LaunchMode` variants (no `CommandLine`/`Tui` arms).
- `compute_block_size` keeps the fork's inline `terminal_spacing` computation and hardcoded `SizeInfo::new_without_font_metrics(24, 80)`; upstream's `(24, 120)` value pre-dates this commit and its comment's shared-session-viewer rationale is a removed surface here, so the else-branch comment keeps the fork's "standard 80x24 terminal" wording under upstream's "windowless backend has no font" first line.
- `Delegate::microphone_access_state` trait requirement and headless-delegate impl in `warpui_core`/`warpui` — fork-absent (voice input removed).
- macOS `#[cfg(target_os = "macos")]` wrappers around the Dock-setup and `set_app_icon` gates stay dropped, matching the fork's macOS-only no-cfg form.

### `98551490f` — Move team switcher beside profile avatar (#16041)

Decision: **N/A (reject)**. Four-line reorder of `render_team_switcher_pill` next to `FeatureFlag::AvatarInTabBar`/`render_avatar_button` in `app/src/workspace/view.rs`. The fork has no team switcher, avatar button, or the flag (Teams and account surfaces removed); no anchor symbols exist.

## Verification

Run on `merge/upstream-2026-09-17` with `CARGO_PROFILE_DEV_DEBUG=0`:

- `cargo check -p warp --all-targets --message-format short` — pass (pre-existing warnings only, all in files untouched by the port).
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — pass (156/156, 1 leaky as previously recorded).
- `cargo build -p warp --all-targets --message-format short` — pass.
- Deleted-surface scans: no new hits in ported files; remaining hits are pre-existing weak-handle "upgrade" false positives, retained SSH/remote references, and the documented ConPTY zsh-bootstrap comments.

`cargo clean` run after merge/tag/push.

## Notes

- Future upstream warpui/app commits will reference `new_windowless`/`is_gui()`; the fork now matches those names. `platform::headless` module naming is unchanged (upstream also left it untouched).
- If an upstream change ever ports the APP-2946 background-only marking or the microphone-authorization query, re-apply them on top of the `is_gui()` gates introduced here rather than reintroducing `is_headless()` call sites.
