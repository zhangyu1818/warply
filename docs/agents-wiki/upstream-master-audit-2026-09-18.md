# Upstream Master Audit 2026-09-18

Range under review: `98551490f6..upstream/master` (11 commits)

Previous audited upstream tip: `98551490f6 Move team switcher beside profile avatar (#16041)`

Current upstream tip detected: `bb0a0593ce computer_use: centralize the recording temp-file path across recorders (#16045)`

Total upstream commits in this incremental range: 11

Status: triage complete. One adapted port (`1bf1c6a2b`); ten commits rejected or not applicable.

## Per-Commit Triage

### `287888707` — [REMOTE-2064] Enable Windows recording plumbing (#15963)

Decision: **N/A (reject)**. Stack 1 of the Windows computer-use video-recording series. The fork has no computer-use recording surface at all:

- `app/src/ai/blocklist/action_model/recording_controller.rs`, `execute/start_recording.rs`, and `execute/stop_recording.rs` do not exist; the fork's `action_model/execute/` contains no recording executors (verified by directory listing and `rg -l recording`).
- `FeatureFlag::VideoRecording`/`WindowsVideoRecording` and the `video_recording_enabled()` helper have no hits in the fork's `crates/warp_features/src/lib.rs`.
- `app/src/ai/agent/api/impl.rs` (tool advertisement) is the removed old Warp Agent API surface in the fork.
- `crates/computer_use` in the fork has no recording modules, no `recording_tests.rs`, and no `windows/` directory.

### `4a7786c76` — computer_use: record the Windows desktop with gdigrab (#15964)

Decision: **N/A (reject)**. Windows-platform recorder implementation for the same removed recording surface; `crates/computer_use/src/windows/` is fork-absent and Windows host platform code is rejected outright.

### `1bf1c6a2b` — fix: eliminate startup race in flaky async searcher rebuild test (#16064)

Decision: **adapt/port**. Test-only fix for a real startup race in `test_searcher_async_rebuild_is_not_delayed_when_its_marker_is_never_sent`: the test spawned the real background writer via `create_async_searcher`, then set `pending_rebuild` on shared state afterward, so the writer's first loop iteration could observe `None` and block until the 5-second idle timeout. The fork's async searcher (APP-5389 stack, ported 2026-08-21) carries the identical test with the identical race in `app/src/search/searcher_test.rs`.

The upstream patch was applied exactly, with the path rewritten from `crates/warp_search_core/src/searcher_tests.rs` to the fork's `app/src/search/searcher_test.rs` (both are `#[cfg(test)] #[path = ...] mod test;` children of their searcher module, so the private `process_searcher_events`, `SearcherProducerState` fields, `SimpleFullTextSearcher.writer`, and `AsyncSearcher` fields remain accessible):

- Import list gains `process_searcher_events`.
- New `async_searcher_with_stranded_rebuild` helper builds the shared state with `pending_rebuild` populated before the writer task is spawned.
- The test now constructs `Background::default()` (no `Arc`), the searcher, and the `PendingRebuild` up front and hands them to the helper; its doc comment is updated to upstream's wording.

Intentionally omitted/kept-divergent hunks:

- None from this commit. The only remaining differences from upstream's final file are the fork's pre-existing tokenizer test data (`check_status:/dev/local_object-0` instead of `warp_drive-0`) and the `use crate::search::searcher`/`use warpui::r#async` import paths, both predating this port.

### `701e79cec` — Fix sharer reconnects that stall before server confirmation (#16052)

Decision: **N/A (reject)**. Shared-session sharer reconnect logic; `app/src/terminal/shared_session/` is removed in the fork (cloud session sharing removed). No anchor symbols.

### `4a7740e81` — [APP-5545] Extract native Claude and Codex usage metrics (#15927)

Decision: **reject**. New `crates/warp_harness_usage` library for agent-harness usage metrics (layer 1 of the APP-5545 usage stack). The fork has no usage tracking, no harness stack, and no consumer for the crate; `specs/APP-5545/` is rejected upstream planning docs. Matches the standing APP-5720/usage rejections in the 2026-09-16 audit.

### `856bb6e76` — Add AWS ECR credential support to managed secrets (schema + WASM) (#16044)

Decision: **N/A (reject)**. Touches only removed surfaces: `crates/managed_secrets*`, `crates/graphql`, `crates/warp_graphql_schema`, `app/src/ai/agent_sdk/`, and `app/src/ai/auth_secret_types.rs` (absent; verified).

### `3337e1000` — Bump some dependencies to shrink the Windows dependency tree (#16037)

Decision: **N/A**. Bumps `cpal` 0.17→0.18 in `crates/voice_input` and `rustls-platform-verifier` 0.6→0.7 in `crates/websocket`, plus the resulting Cargo.lock churn (drops one `windows-targets` copy). Both crates are fork-absent (voice input removed; websocket PTY/graphql-ws path removed), so neither manifest hunk applies and the lock changes describe dep graphs that do not exist in the fork's lock. The fork's single remaining `windows-targets 0.52.6` lock entry comes from other retained dependencies, same as upstream's "last copy".

### `79ef1d532` — [APP-5545] Serialize harness transcript saves and bound finalization (#15928)

Decision: **N/A (reject)**. Layer 2 of the APP-5545 stack, confined to `app/src/ai/agent_sdk/driver/harness/` (removed surface) and `specs/APP-5545/`.

### `1378696ce` — Complete explicit zero idle timeouts immediately (#16050)

Decision: **N/A (reject)**. `warp agent run --idle-on-complete 0m` behavior lives in `app/src/ai/agent_sdk/driver.rs` + `driver_tests.rs` (removed old Warp Agent CLI/harness surface). The only fork `rg "idle"` hit in the searcher is the unrelated `hit_idle_timeout` loop variable; there is no `IdleTimeoutSender` in the fork.

### `f62a045e0` — computer_use: add Windows recording overlays (#16000)

Decision: **N/A (reject)**. Stack 3 of the Windows recording series. All computer-use recording/overlay modules are fork-absent. The one shared-file hunk (`execute/use_computer.rs`, adding `recording_geometry` to the `PointerSink` recording context) anchors on `recording_context`/`recording_target`, which do not exist in the fork's `use_computer.rs` (verified: no recording refs in that file).

### `bb0a0593c` — computer_use: centralize the recording temp-file path across recorders (#16045)

Decision: **N/A (reject)**. Extracts `recording_paths.rs` shared by the Windows/macOS/Linux recorders; the fork has no recorder files at all (`crates/computer_use/src/mac/` contains only keyboard/mouse/screenshot helpers — no `recording.rs`).

## Verification

Run on `merge/upstream-2026-09-18` with `CARGO_PROFILE_DEV_DEBUG=0`:

- `cargo nextest run -p warp -E 'test(async_rebuild_is_not_delayed)'` — pass.
- `cargo nextest run -p warp -E 'test(search::searcher)'` — pass (24/24).
- `cargo check -p warp --all-targets --message-format short` — pass (pre-existing warnings only).
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — pass (156/156).
- `cargo build -p warp --all-targets --message-format short` — pass.
- Deleted-surface scans: no hits in the ported file; the only diff-touched file is `app/src/search/searcher_test.rs`. Remaining full-repo hits are the documented pre-existing ones.

`cargo clean` run after merge/tag/push.

## Notes

- The fork's async searcher tests live in the `warp` app crate (`app/src/search/searcher_test.rs`) rather than upstream's `warp_search_core` crate; future upstream `crates/warp_search_core/src/searcher_tests.rs` changes should keep being mapped there.
