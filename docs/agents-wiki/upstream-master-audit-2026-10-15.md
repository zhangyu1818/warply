# Upstream Master Audit — 2026-10-15

Incremental audit for `d0d50955d..upstream/master` (`dc0b890ba`), 7 commits. The previous audit is `upstream-master-audit-2026-10-14.md`.

## Summary

One adapted port and six rejections/not-applicable commits. `5ca78a44d` fixes accepted `apply_patch` deletions emptying files instead of deleting them, moves saving at the original path, and adds native remote move persistence — all retained AgentView inline-diff / retained SSH remote-server behavior, ported onto the fork's current architecture. The port's prerequisite surface (`SaveFuture`, `PersistenceState`, `DiffStorage`, `HostRequestHandle`) belongs to upstream `150b4c158`/`44893c508`/`14c8c8ded`, whose rejection (2026-07-11 audit for the TUI-agnostic file-execution redesign) stands; the behavioral core was adapted to the fork's fire-and-forget `FileModel` API and `RemoteServerClient` remote path instead. The other six commits live in removed `agent_sdk`/orchestration/`server_api`/`warp_graphql_schema` surfaces with zero live fork anchors (verified per file), except `1d32d02f0`'s retained-crate piece `crates/http_client`, whose `new_external()` constructor only serves the fork-absent presigned-storage/server-API upload path and whose test requires fork-absent `ChannelState::server_root_url()` — rejected as dead code.

## Commit-by-Commit

### `1d32d02f0` Keep Warp headers off presigned storage transfers (#16445) — rejected

Keeps Warp/IAP headers off S3 presigned transfers by adding `Client::new_external()` plus an `include_warp_headers` flag to `crates/http_client`, wiring it through `warp_server_client`/`server_api` presigned-upload paths and `agent_sdk` artifact/snapshot uploaders. `agent_sdk/**`, `app/src/server/**`, and `crates/warp_server_client/**` are fork-absent. The retained-crate piece (`crates/http_client/src/lib.rs` +25, tests +52) adds a constructor with zero fork consumers (`rg new_external` empty; the only upstream callers are the removed crates), keeps wasm-only `omit_credentials` code the fork cannot exercise, and its test depends on `ChannelState::server_root_url()` which does not exist in the fork. Omitted in full; the fork's `http_client` behavior is unchanged.

### `5ca78a44d` Fix apply_patch deletion and native remote move persistence (#16444) — accepted (adapted)

Upstream fixes: accepted `apply_patch` Delete diffs emptied the source file and moves saved at the original path; destructive operations now act on the requested directory entry (including symlinks) rather than the canonicalized referent; hard-link alias moves fail before writing; remote moves use a native save-then-rename via a new `RenameFile` RPC; undo restores deletes/moves/creates per diff type. All retained surfaces (AgentView inline diff, `warp_files`, SSH remote server daemon/client). Applied via `git cherry-pick --no-commit`, then resolved against the fork's diverged architecture.

Adaptations (upstream core preserved, fork integration glue minimized):

- **`crates/warp_files/src/lib.rs`**: `entry_path` field/constructor/accessor and the rewritten `rename_and_save`/`delete` local branches are upstream-exact except the fork's `Result<(), FileSaveError>` fire-and-forget signature and `Rc` event errors (the fork never ported `SaveFuture`/`Arc` errors — rejected `150b4c158` machinery). The remote branch dispatches through the fork's `RemoteServerClient::rename_file` instead of upstream's fork-absent `HostRequestHandle`.
- **`crates/remote_server/proto/remote_server.proto`**: `RenameFile`/`RenameFileResponse`/`RenameFileSuccess` messages are upstream-exact; envelope entries adapted to the fork's `ClientMessage` tag 28 and `ServerMessage` tag 28 (the fork has no `HostScopedRequest` envelope; its file ops are client-scoped like `WriteFile`=7/`DeleteFile`=8).
- **`crates/remote_server/src/manager.rs`**: NOT ported — the diff's `HostRequestHandle::rename_file` + `host_response::rename_file_result` live in fork-absent machinery. Instead `crates/remote_server/src/client/mod.rs` gains `rename_file`, mirroring the fork's existing `write_file`/`delete_file` client methods (fork integration glue).
- **`app/src/remote_server/server_model.rs`**: upstream-exact `FileOpKind::Rename` arms, `handle_rename_file`, and doc updates; the dispatch arm adapted into the fork's direct `client_message::Message` match (no `HostScoped` wrapper level).
- **`app/src/code/inline_diff.rs`**: `session_type` field replaces `is_new_file` (upstream), `save_content` gains the upstream `DiffType`-dispatch (Delete → delete, cross-path rename → `rename_and_save`, same-path rename → save), and the new inherent `restore_and_save_diff` carries the upstream restore dispatch arms verbatim (Create → delete; Delete+Local re-registers the original entry path because local IDs cache canonical paths; Delete+Remote saves at the remote id; cross-path rename registers the target id — local or remote — and moves back; otherwise base-content save), returning `Result<(), String>` instead of `Result<SaveFuture, String>`. `base_content` accessor ported upstream-exact. The fork's `FileModel` event subscription in `finish_file_registration` is retained as the completion path (upstream removed it in favor of `PersistenceState`, which is `150b4c158` machinery and is not ported). `restore_diff_base` removed from the `DiffViewer` trait (`diff_viewer.rs`), `InlineDiffView`, and `LocalCodeEditorView`, exactly as upstream.
- **`app/src/ai/blocklist/inline_action/code_diff_view.rs`**: `revert_changes` rewritten to per-file `restore_and_save_diff` dispatch with per-file revert-error toasts, `mark_action_as_reverted` only on full dispatch success (failures stay accepted and retryable — upstream semantics), and a sticky `revert_attempted` flag. Upstream's `revert_in_progress` is windowed by its `join_all` completion future, which the fork's fire-and-forget API cannot reproduce; because the fork's `SavingDiffs` state machine guarantees revert only runs from settled `Accepted(None)`, a sticky flag gives the same attribution (every save failure after a revert attempt is restore-caused) and the `FailedToSave` handler reports it with the upstream revert-failure toast instead of the accept-failure `safe_error!`.
- **`crates/warp_util/src/file.rs`**: `FileSaveError::Other(String)` ported as the exact `44893c508` hunk (prerequisite for the same-file and rename-failure error messages).
- **Tests**: `inline_diff_tests.rs` adapted to fork fixtures (`LocalIdentityProvider` instead of `AuthStateProvider`; no `UserWorkspaces`/`MockTeamClient`/`MockWorkspaceClient`) and event-based waits (`accept` waits on view events, `restore` on `FileModel` events since delete/move restores dispatch on freshly registered ids); `accept_propagates_delete_failure` adapted to its event-based form (`accept_delete_failure_reports_failed_save_event`). Daemon `file_operation_tests.rs` adapted to the `ClientMessage` envelope (preserving the prost encode/decode round-trip) with local result-parse helpers replacing the fork-absent `host_response` module, and without re-registering `RemoteServerManager` (the fork's terminal test initializer already provides it). `warp_files` `lib_test.rs` remote-move test asserts the dispatch error (the fork reports a disconnected host synchronously through `resolve_remote_client`) instead of upstream's future+event pair.
- **Omitted paths and reasons**: `crates/warp_tui/src/tui_diff_storage*.rs` — `crates/warp_tui` is fork-absent. `app/src/ai/blocklist/inline_action/code_diff_persistence_tests.rs` and the `DiffStorage`/`changed_lines_for_result`/`start_saving` hunks in `code_diff_view.rs` — they exercise the `DiffStorage` accept-result machinery from rejected `150b4c158` (upstream's `accept_and_save` returning a joined `RequestFileEditsResult`), which has no fork counterpart. `PersistenceState`/`track_operation` and the `overlapping_and_repeated_restores_do_not_repeat_a_move` test — in-flight dedupe/completion-wait semantics of the same rejected machinery; the fork's completion path remains `FileSaved`/`FailedToSave` events. `app/src/ai/agent_sdk/**` hunks — fork-absent surface.

### `9e57f4f6a` Add Pi as a Factory-only ACP harness (#16427) — rejected (removed harness/cloud surfaces, zero live anchors)

Adds the Pi harness (`pi.rs`, `pi_transcript.rs`, ACP `model_config`, harness registration) across `app/src/ai/agent_sdk/**`, plus `Harness::Pi` consumers. All `agent_sdk` paths are fork-absent; the fork-present files have no anchors: `app/src/ai/agent/conversation.rs` has no `AIAgentHarness` enum (removed with old harness persistence), `conversation_details_panel.rs` has no `find_conversation_id_by_server_token` continuation block, and `auth_secret_types.rs`, `local_harness_launch.rs`, `harness_availability.rs`, `harness_display.rs`, `local_harness_setup.rs`, `orchestration/**` do not exist. "Factory-only" is a Warp cloud service dependency, so the harness itself is out of scope even as a future candidate.

### `1b76b6f3e` Relay queued shared-session prompts to ACP harnesses and keep their attribution (#16475) — rejected (removed surfaces, zero anchors)

Shared-session prompt relay into `agent_sdk` ACP harnesses. `app/src/ai/blocklist/controller/` in the fork has no `external_harness.rs` or `startup_queue.rs` (only `input_context.rs`, `response_stream.rs`, `slash_command.rs`), and `agent_sdk/driver/harness/acp/{mapping,mod}.rs` are fork-absent. Shared-session relay is the removed cloud session-sharing surface.

### `af9a6a956` Enable gated Pi cloud children through shared ACP skills (#16455) — rejected (removed surfaces)

Gated Pi cloud children via orchestration + shared ACP skills dirs. All consumers are fork-absent: `app/src/ai/orchestration/**`, `app/src/ai/agent/api/convert_from.rs`, `agent_sdk/.../skill_dirs_publish*` (app-managed skills, rejected surface), the `oz-child-agent-orchestration` SKILL.md, and the `warp_multi_agent_api` rev bump (crate not in the fork's `Cargo.toml`/`Cargo.lock`).

### `bf12566b4` Instrument ACP harness start and surface auth-failure detail (#16483) — not applicable (fork-absent files)

All five touched files (`agent_sdk/driver.rs`, `driver/error_classification*.rs`, `driver/harness/acp/mod.rs`, `setup_observability.rs`) live under the removed `app/src/ai/agent_sdk/`.

### `dc0b890ba` Disable deferred repository capability in execution bootstrap (#16489) — not applicable (fork-absent files)

Touches only `app/src/ai/agent_sdk/{execution_config,mod}.rs`, `app/src/server/server_api/ai*.rs`, and `crates/graphql/**` — all removed surfaces with no fork counterpart.

## Verification

- `cargo check -p warp --all-targets --message-format short`: clean (pre-existing warnings only).
- `cargo check --workspace --all-targets --message-format short`: clean.
- `cargo fmt -- --check`: clean.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: 156/156 passed.
- `cargo nextest run -p warp_files -p remote_server`: 86/86 passed.
- Ported-area tests (`test(inline_diff) | test(file_operations::) | test(code_diff)`): 18/18 passed, including symlink delete/move entry semantics, hard-link collision, same-path move, destination-replacement permissions, native-rename-failure reporting, delete/create/restore flows.
- `cargo build -p warp --all-targets --message-format short`: succeeded; `cargo clean` follows the push per the release workflow.
- Restored-deleted-surface scans (`rg` over `app crates script Cargo.toml`): 232/0/6 hits — identical to the documented allowances from the 2026-10-14 audit; zero new hits.
