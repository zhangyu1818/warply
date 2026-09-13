# Upstream Master Audit 2026-09-13

Range under review: `4143c09ff..upstream/master` (2 commits)

Previous audited upstream tip: `4143c09ff Bump command signatures for du --time (#15950)`

Current upstream tip detected: `3959ea721 Route AI history updates through TerminalView (#15943)`

Total upstream commits in this incremental range: 2

Status: triage complete. Both commits ported to `merge/upstream-2026-09-13`:

- `a06279712` — full adapted port of the completions v2 / JS plugin host rip-out: `app/src/plugin/**`, `app/src/completer/js.rs`, `crates/warp_js/`, `crates/command-signatures-v2/`, every `v2.rs` module in `crates/warp_completer/`, the `completions_v2`/`plugin_host` cargo features, the workspace `command-signatures-v2`/`warp_js`/`rquickjs` deps, the `warp_cli` `PluginHost` worker command, presubmit/skills test-command simplifications, and the Node/Corepack steps in `.github/actions/prepare_environment` (only needed to build TS signatures).
- `3959ea721` — adapted port of the AI history event routing rework: `BlocklistAIHistoryModel` events are routed by each owning `TerminalView` to indexed block handles instead of every `AIBlock` subscribing for its lifetime; `AIBlock` captures `receives_live_output_updates` at construction and processes initial `PartiallyReceived` output directly; `on_output_status_update` becomes `pub(crate) handle_history_output_update`; baseline `ai_block_for_exchange` restored; `UpdatedTodoList` gains `conversation_id`.

## Per-Commit Triage

### `a06279712` — Rip out completions v2 and associated code. (#15965)

Decision: **accept/adapt** (retained dead-code cleanup; the fork carried the full v2 stack feature-gated off). The fork's pre-port state matched the upstream parent closely: `imp`/`legacy` cfg-gated dispatch in `warp_completer`, the `completions_v2`/`plugin_host` features with optional `rquickjs`/`warp_js`/`command-signatures-v2` deps, and the whole `app/src/plugin/` host tree. Applied via `git cherry-pick --no-commit` with conflict resolution; every retained path follows the upstream post-image.

Intentionally omitted paths and reasons:

- `app/src/tui_export.rs` — fork-absent (no TUI export surface).
- `app/src/bin/integration.rs`, `.github/STAKEHOLDERS` — fork-absent.
- `app/src/plugin/host/wasm/mod.rs` — fork already deleted the wasm host stub.
- `crates/warp_cli` `MinidumpServer` worker arm + `lib_tests.rs` — fork-absent (no crash-reporting worker; the fork deleted `lib_tests.rs`).
- Upstream `AGENTS.md` and `.github/workflows/ci.yml` hunks — fork-authored replacements with no v2 anchors.
- Warp-on-Web manifest sections in `crates/warp_completer/Cargo.toml` (`[target.'cfg(target_family = "wasm")'.dependencies]` etc.) — not restored; the fork keeps its single-section macOS-host manifest.

Fork-kept divergences inside applied files:

- `describe.rs`/`alias.rs` keep the fork's singular test filenames (`describe_test.rs`, `alias_test.rs`).
- `suggest/test.rs` keeps the fork's plain `TEST_WORK_DIR = "/home/"` constants instead of upstream's cfg(windows)/cfg(unix) constant modules (macOS-only host policy), and the fork's grouped `use crate::{...}` import shape in `parsers/test.rs`/`suggest/test.rs`.
- `app/src/lib.rs` keeps the fork's inline worker dispatch in `run()` (the fork has no `run_worker_command`); the `PluginHost` arm is removed and the fallback arm cfg becomes `#[cfg(not(feature = "local_tty"))]` (the fork equivalent of upstream's `all(target_family = "wasm", not(feature = "local_tty"))` on a wasm-free target set).
- `ipc` crate doc comment takes upstream's post-image wording (plugin-host mention removed).
- `WARP.md` testing notes updated to the un-excluded `cargo nextest run --no-fail-fast --workspace` (fork README analog of upstream's AGENTS.md hunk).

Cargo.lock regenerated via `cargo metadata`: pure removals of `command-signatures-v2`, `warp_js`, `rquickjs{,-core,-sys}`, and `bincode` from the `warp_completer` dep set — matching the upstream lock diff's intent.

### `3959ea721` — Route AI history updates through TerminalView (#15943)

Decision: **adapt** (retained AgentView/terminal-shell architecture change; all local, ACP-compatible). The core port lands in full: per-`AIBlock` `BlocklistAIHistoryModel` subscription removed, `receives_live_output_updates` captured from the streaming status at construction, the streaming branch of `AIBlock::new` processes initial `PartiallyReceived` output via `handle_updated_output` directly (so persistence-only replays of completed restored/forked blocks no longer rerun completion side effects), `TerminalView::route_ai_block_history_event` targets indexed block handles per event kind, and `UpdatedTodoList` gains `conversation_id`.

Fork adaptations and omissions:

- `ConversationUsageMetadataUpdated` routing arm omitted: the fork's `BlocklistAIHistoryEvent` has no usage-metadata variant (the upstream usage/ancestor-rollup walk belongs to the cloud usage surface this fork removed). The fork catch-all arm instead lists `ClearedConversationsInTerminalView` and `ConversationOwnershipTransferred` (fork names for upstream's `ClearedConversationsForTerminalSurface`/`ConversationTransferredBetweenTerminalSurfaces`); upstream-only variants (`UpgradedTask`, `UpdatedConversationTitle`, `ConversationServerTokenAssigned`, `NewConversationRequestComplete`, `OrchestrationConfigUpdated`, `LocalSharedSessionEstablished`) do not exist here.
- `conversation.rs` emit-site hunks not applicable: upstream adds `conversation_id` at three `UpdateTodos` emit sites inside the old Warp Agent server-streaming `Action::AddMessagesToTask` path; the fork's `conversation.rs` has no such path (ACP renders todos through block output messages), and `UpdatedTodoList` remains dormant-but-consumed wiring in this fork, now shape-compatible with upstream.
- Event field naming stays `terminal_view_id` (the fork never took upstream's `terminal_surface_id` rename).
- Routing call inserted after the fork's own owner guard in `handle_ai_history_model_event` (the fork has no `render_owner_for_ai_history_event`; its `terminal_view_id()` guard provides equivalent ownership filtering for the routed event kinds).
- `ai_block_for_exchange` restored verbatim from the upstream baseline next to the fork's `ai_block_handle_by_view_id` (the fork's creation commit had dropped it as dead code; this port gives it a live caller).
- `todos/popup.rs` and `context_chips/display.rs` destructures add `..` in the fork's nested-if style.
- Tests: 4 of 5 upstream tests ported into `view_test.rs` (append targeting, streaming-exchange targeting, completed-restored fork replay, todo-scoped targeting), using fork field names and the fork's 3-arg `restore_conversation_after_view_creation`. The `usage_update_targets_latest_blocks_for_conversation_and_ancestors` test is omitted (no usage event/`start_new_child_conversation` anchors), and the `is_passive_conversation_is_recomputed_on_conversation_reassignment` assertion hunk is omitted (that test does not exist in the fork).
- `crates/warp_tui` test hunks not applicable (fork has no warp_tui).

## Verification

Run on `merge/upstream-2026-09-13` after all ports:

- `cargo nextest run -p warp_completer` — pass (175 run, 175 passed, 4 skipped).
- `cargo nextest run -p warp -E 'test(appended_exchange_targets) | test(streaming_exchange_targets) | test(fork_replay_does_not) | test(todo_update_targets)'` — pass (4/4).
- `cargo nextest run -p warp -E 'test(terminal::view) | test(blocklist) | test(ai::agent)'` — pass (423/423).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — pass (156/156).
- `cargo check -p warp --all-targets --message-format short` — pass (pre-existing warnings only).
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass (late rustfmt fixes for port 1 distributed via `--fixup` + autosquash rebase).
- `cargo build -p warp --all-targets --message-format short` — pass (`warply` binary links; verified by forced rebuild after touching `app/src/lib.rs`); `cargo clean` run after merge/tag/push.
- Deleted-surface scans over the added lines of both ports (`3121d84be..HEAD`): zero hits for cloud/auth/billing/telemetry, MCP/skills, or native Linux/Windows host patterns. Broad-repo scan hits are pre-existing and documented in earlier audits.

## Notes

- Disk pressure interrupted verification twice (`No space left on device`): a stale 2.3 KB corrupt `target/debug/deps/warp-*` test binary and a 7.7 GB `target/debug/incremental` cache were the causes. Resolution: remove the stale binary, drop incremental caches, and build with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0` for the remainder of the cycle. Not a code regression; recorded for future sessions on this machine.
- One leftover v2 re-export (`warp_completer::completer::JsExecutionContext`) survived the cherry-pick auto-merge because of fork context drift; removed manually to match the upstream post-image.
