# Upstream Master Audit 2026-09-02

## Scope

- Current fork before this audit: `7265b6ad8` (`main`, `v2026.09.01`).
- Upstream source reviewed: `6fac731c4..upstream/master` (11 commits, tip `db6ab73056`).
- Result: five accepted/adapted ports, two small retained infra ports, one pre-existing fork-baseline test failure fixed, and four commits rejected or not applicable.

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `ccf683193` | Remove unused orphaned cargo features (#15694) | **Adapt (ported)** | Of the 18 upstream-removed orphaned feature declarations, four still existed in the fork's `app/Cargo.toml` (`quake_mode`, `rich_history`, `system_theme`, `toggle_bootstrap_block`) and were removed; the other fourteen were already absent. Verified none is in `default`, referenced by `cfg(feature)`, or mapped through the feature-flag pipeline; runtime quake-mode windowing and system-theme settings code is unchanged, matching upstream's rationale. |
| `2e645f916` | Install protobuf in macOS bootstrap (#15710) | **Accept (ported)** | `brew install protobuf` added to `script/macos/bootstrap` after `brew install llvm`. Upstream's motivating crate (`warp_multi_agent_api`) is removed here, but the retained `app` and `remote_server` crates compile protos with `prost-build`, which needs `protoc` on PATH, so the retained build has the same requirement. |
| `09f0c2bf7` | Re-enable LRC process activity signals on Windows (#15690) | **Not applicable** | The fork's 2026-08-31 adapted LRC port already dropped both platform gates; `lrc_activity_signals_supported()` is just `FeatureFlag::LrcActivitySignal.is_enabled()` (the fork has no wasm or Windows host target). Upstream's change is subsumed. |
| `83e270f1d` | Fix em_width panic when a glyph has no bounding box (#15705) | **Accept (ported)** | `Cache::em_width` now falls back to the 'm' glyph's horizontal advance, then to `font_size.max(1.0)` with a `log::warn!`, instead of `expect`-panicking. `fonts_test.rs` gains the `EmWidthFontDB` mock and four tests; the only adaptation is dropping the mock's three `#[cfg(not(target_family = "wasm"))]` attributes (no wasm target in this fork). |
| `97037ec83` | Add ViewHandle::try_update and use it to guard shared session retention (#15695) | **Adapt (ported)** | See port record below. |
| `83387d2ed` | Pin winit to X11 request_user_attention panic fix (#15717) | **Not applicable** | `winit` is not a dependency of this fork (macOS AppKit windowing; no `winit` in `Cargo.toml`/`Cargo.lock`). The pin fixes an X11-only crash path. |
| `92a98662f` | Fix crash on empty streamed Agent document update (#15720) | **Accept (ported)** | `Buffer::ensure_plain_text` also inserts the `<text>` marker when the buffer is empty, so a streamed update that replaces all formatted content with an empty document no longer panics in selection handling. Applied verbatim to `crates/editor` including the round-trip regression test. |
| `e47c7ae9c` | Replace hand-rolled test polling with assert_eventually (#15721) | **Adapt (ported)** | See port record below. |
| `09127d8be` | [REMOTE-2661] Allow a debug agent in a retained setup-failure session (warp client) (#14916) | **Reject** | Entirely a cloud feature: GraphQL `setupFailureDebugAuthorization` query, ambient-agent task retention, `warp_isolation_platform` workload tokens, shared-session no-token prompt authorization, `AgentConversationsModel` task fetch gating, and `AIQueryRouting::RetainedSetupFailureDebug`. All retained-file hunks (`terminal/input.rs`, `slash_commands/*`, `terminal/view.rs`, `agent/conversation.rs`, `local_tty/terminal_view_adaptor.rs`) exist only to wire that cloud path onto anchors absent from this fork (`AmbientAgentTaskId`, `resolve_ai_query_routing`, `shared_session_status`, `ServerApiProvider` AI client). No separable local behavior. |
| `712baa4cf` | [REMOTE-3086] Make text in Thought dropdowns selectable and copyable (#15691) | **Adapt (ported)** | See port record below. |
| `db6ab7305` | Disambiguate MCP 401s so headless runs report the real failure (#15714) | **Reject / not applicable** | Touches only app-managed MCP (`app/src/ai/mcp/**`, `crates/mcp/**`) and the old agent SDK driver — all absent from this fork (MCP belongs to the ACP agent process). |

## Port record: `97037ec83` ViewHandle::try_update

### Runtime-ownership review

Pure warpui_core framework change: view checkout failures (window closed, circular
update) become a typed `Result` instead of panics, so queued work that outlives its
window can fail gracefully. Tests are the consumer in this fork.

### Applied from the exact upstream source

- `crates/warpui_core/src/core/view/handle.rs`: `ViewUpdateError` enum, `ViewHandle::try_update`, `UpdateView::try_update_view` trait method.
- `crates/warpui_core/src/core/app.rs`: `try_update_view` on `App`/`AppContext`, panic conversion in `update_view`, `Result`-returning `take_view_for_update`, `simulate_window_closed` test-util helper.
- `crates/warpui_core/src/core/model/context.rs` and `core/view/context.rs`: `try_update_view` forwarding impls.
- `crates/warpui_core/src/core/try_update_view_tests.rs`: all three tests.

### Fork adaptations

- `take_view_for_update` returns `Result<Box<dyn AnyView>, ViewUpdateError>` (fork's checkout type; upstream uses `StoredView`).
- All `try_update_view` signatures use the fork's `T: View` bound (the fork's `ViewContext` requires `T: View`; upstream uses `T: Entity`).
- `warp_errors` plumbing dropped with the fork's removed error-registry crate: no `ErrorExt` impl, no `register_error!`, and the doc comment's "reaches Sentry" clause removed.
- Tests registered in the fork's `core/mod_test.rs` (upstream adds them to `core/mod_tests.rs`).

### Intentionally omitted paths (with reasons)

- `app/src/ai/agent_sdk/driver/terminal.rs` usage site (shared-session retention guard): `agent_sdk` is a removed surface with no `TerminalDriver`/session-sharing anchors in this fork.

## Port record: `712baa4cf` Thought dropdowns selectable

### Applied from the exact upstream source

- `app/src/ai/blocklist/block/view_impl/output.rs`: `render_collapsible_text_block_section` now passes `selectable: true` into `TextSectionsProps` (was `let selectable = false;`).
- `crates/warpui_core/src/elements/event_handler.rs`: `Element::as_selectable_element` + `impl SelectableElement for EventHandler` forwarding all five methods to the child, so selection reaches text wrapped in event handlers (the collapsible reasoning-body sandwich).

### Intentionally omitted paths (with reasons)

- `selectable_area.rs` `child_max_z_index` hunk and the new `selectable_area_tests.rs`: they patch the z-index mouse-down gate (`event.at_z_index(...)`) that upstream `da3b560654` (#10433) added to `SelectableArea::dispatch_event`. That gate was never ported to this fork (its Ask-User-Question speedbump feature stack went a different route; see change-map), and the fork's `SelectableArea` dispatch has no z-index gate, so the covered-by-child-layer bug path does not exist here.
- The `debug_child_view_ids()` test-util forwarding on `EventHandler`: no consumer in this fork (same decision as the 2026-09-01 audit of `6fac731c4`).

## Port record: `e47c7ae9c` assert_eventually test polling

Applied to the three retained test files whose polling loops exist here:

- `app/src/pane_group/mod_tests.rs`: bootstrap wait in `test_pane_focus_does_not_have_an_infinite_event_loop`.
- `app/src/terminal/find/model/async_find_tests.rs`: both async-find completion polls.
- `app/src/terminal/input_test.rs` (fork's singular filename): local `poll_until` removed; both bundled-spec `input_tab` polls converted.

Omitted files (absent in this fork): `agent_sdk/driver_tests.rs`, `server/sync_queue_tests.rs`, `settings/cloud_preferences_syncer_tests.rs` (removed surfaces), `notebooks/notebook_tests.rs` (upstream's cloud-sync save-loop test architecture; the fork has no such file), and `search/command_palette/data_sources_tests.rs` (upstream cloud/team workflow-indexer test file; fork has no equivalent).

## Pre-existing failure fixed: focused-pane synchronization test

`pane_group::tests::test_focused_pane_is_synchronized_with_application_focus`
failed on clean `main`, on `v2026.08.22`/`v2026.07.11`/`v2026.06.04`, and at the fork
baseline `19659d12` — broken since baseline, not by any merge. Root cause: the baseline
replaced upstream's synchronous `PaneGroupAction::HandleFocusChange` dispatch from
`TerminalView::on_focus` with the owning terminal pane event subscription (recorded in
change-map's 2026-05 runtime warning cleanup: child focus paths may not have a
`PaneGroup` responder). The pane group therefore updates when the queued
`FocusChanged` event is delivered — one queue tick later than the test's
`FocusDetectionView` mid-queue observer expected.

Fix (fork-adapted test, `d24841517`): drop the `FocusDetectionView` observer, keep
the pane layout, focus, and focused-pane/active-session assertions, and poll for
propagation with effect-flushing updates via `assert_eventually!`. Per the fork's
recorded architecture this remains the correct propagation mechanism; the test now
asserts it directly instead of upstream's dispatch timing.

## Verification

- `cargo fmt -- --check`: clean (one follow-up fmt commit for import ordering after the ports).
- `cargo check -p warp --all-targets --message-format short`: pass (8 pre-existing dead-code warnings; no new warnings).
- `cargo check --workspace --all-targets --message-format short`: pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: 156 passed.
- `cargo nextest run -p warpui_core -p warp_editor`: 736 passed, 7 skipped (includes the new em_width, try_update, and editor empty-stream tests).
- `cargo nextest run -p warp -E 'test(pane_group::tests) + test(find::model) + test(input_tab)'`: 47 passed (was 46/47 with the baseline failure; now green).
- `cargo build -p warp --all-targets --message-format short`: pass. `cargo clean` after the release push.
- Deletion-surface scans: no new hits; workspace-wide matches remain the pre-existing allowed set (`WeakHandle::upgrade` method names, retained SSH/remote platform detection, bootstrap script comments).
- `CARGO_PROFILE_DEV_DEBUG=0` used for check/test/build as in prior audits.
