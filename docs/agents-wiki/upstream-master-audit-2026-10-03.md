# Upstream Master Audit 2026-10-03

Range under review: `d5d23e611..upstream/master` (9 commits)

Previous audited upstream tip: `d5d23e611 Bound MCP tool calls and environment setup commands (#16219)` (audited 2026-10-02; no port from that range was required beyond `41b83dc7c`, merged as `4cd2faf1f`)

Current upstream tip: `1645db65c Guarantee native zsh completion responses on initialization and errors (#16268)`

Total upstream commits in this incremental range: 9

Status: triage complete. Four accepted ports and one adapted port; four commits rejected or not applicable.

## Per-Commit Triage

### `3308af3ac` — [APP-4553] Skip undo-close cleanup for unavailable pane groups (#16251)

Decision: **accept** (ported as `5dbe0f4f1`). `clean_up_pane_group` in `app/src/undo_close/stack.rs` now guards with `ctx.view_with_id::<PaneGroup>(window_id, pane_group.id()).is_none()` instead of `is_window_open`, so discarding a closed item cannot update a PaneGroup that is no longer in the window's view registry (`Circular view update` panic). The fork already used the identical `view_with_id` guard for the adjacent conversation-history cleanup (stack.rs line 120), so the cherry-pick applied verbatim.

### `71cbee776` — [APP-6223] Skip settings-error sync for unavailable panes (#16249)

Decision: **accept** (ported as `711a32a55`). `sync_settings_error_state_into_settings_pane` in `app/src/workspace/view.rs` uses `let _ = self.settings_pane.try_update(...)` so a temporarily unavailable settings pane skips the best-effort sync instead of panicking. The fork ported `ViewHandle::try_update` on 2026-09-02 (`97037ec83`), so the hunk applied verbatim.

### `3f37d69ff` — Map CLI agents directly to stable telemetry strings (#16252)

Decision: **reject / not applicable**. Every hunk converts telemetry payloads: `server/telemetry/events.rs` and `code_review/telemetry_event.rs` are fork-absent (telemetry Removed System), and every retained-file hunk (`agent_input_footer`, `terminal/view.rs`, `use_agent_footer`, `workspace/view.rs`) edits `send_telemetry_from_ctx!`/`TelemetryEvent`/`NotificationAgentVariant` serialization with `CLIAgentType`, which has zero references in the fork. The `cli_agent.rs` hunks delete the `From<CLIAgent> for CLIAgentType` impl (never existed here) and add `CLIAgent::telemetry_name()` (would be dead code with zero consumers).

### `38b2c55e4` — Write the gh hosts file where gh reads it on Windows (#16238)

Decision: **not applicable**. Entirely inside `app/src/ai/agent_sdk/driver/git_credentials*.rs`, the removed agent-SDK surface, resolving Windows `%AppData%` for sandbox runners.

### `c157d411d` — Enable Windows video recording in release builds (#16260)

Decision: **not applicable**. Adds `FeatureFlag::WindowsVideoRecording` to upstream `RELEASE_FLAGS` for the Windows computer-use recorder; neither `WindowsVideoRecording` nor `VideoRecording` exists in the fork's `crates/warp_features`, and the Windows recording stack was rejected with the 2026-09-18 audit.

### `bfe8b82b7` — Report optional native Codex cache-write token usage (#16230)

Decision: **not applicable**. Entirely inside `crates/warp_harness_usage/` (cumulative/attributed harness usage reporting for the removed agent-SDK harness + warp-server billing path); the crate does not exist in this fork.

### `56c715dcc` — Use shell defaults for right-prompt placement (#16264)

Decision: **accept** (ported as `364597425`). `Block::rprompt_render_offset` selects the right margin from the block's `shell_host` shell identity (fish/PowerShell 0, zsh/Bash and unknown 1, with the upstream `ZLE_RPROMPT_INDENT` TODO), and the input's editor-decorator uses `terminal_model.prompt_block().unwrap_or_else(|| block_list().active_block())` while the new active block awaits shell metadata. Both hunks applied verbatim; the fork's `prompt_block`, `shell_host`, and `ShellType` symbols all pre-exist.

### `b231216aa` — Defer inline history navigation to avoid circular view updates (#14396)

Decision: **adapted port** (ported as `680889746`). Adds `InputAction::SelectPreviousInlineHistoryItem`/`SelectNextInlineHistoryItem` typed actions plus handlers, and replaces the direct `InlineHistoryMenuView` updates in `editor_up`/`editor_down` with `ctx.dispatch_typed_action_deferred(...)` so Up/Down cannot reenter a view update for an already-checked-out menu (the macOS History action routes through `InputAction::Up`).

- `app/src/terminal/input.rs`: applied via `git diff <c>^ <c> -- <path> | git apply --3way`; two conflicts resolved in favor of upstream's deferred-dispatch arms. Fork adaptation: the two new handler methods keep only the plain `inline_history_menu_view` path — upstream's `is_cloud_mode_input_v2_composing`/`cloud_mode_v2_history_menu_view` branches are fork-absent (shared-session input paths removed at baseline).

- Tests remapped from upstream `input_tests.rs` to the fork's `input_test.rs` (singular), inserted before `test_history_up_multiline` with upstream's `selected_inline_history_command` helper and both regression tests verbatim except one fork-harness adaptation: after `simulate_directory_for_completion`, the test re-binds `model_event_dispatcher().set_active_session_id(session_id)`. Root cause (probe-verified): the fork's harness processes `TerminalModel::new_for_test`'s fake session-123 lifecycle (hardcoded `session_id = 123`, real-hostname shell host) after the explicit `bootstrap_terminal`/`set_active_session_id(0)` of the seeded session, and the fork's older `simulate_directory_for_completion` lacks upstream's dispatcher re-bind step (the API is `#[cfg(test)]`-gated while the helper also serves integration builds). Without the re-bind the menu's data source resolves the fake session 123, which owns none of the seeded restored commands. Production behavior is untouched by this adaptation.

### `1645db65c` — Guarantee native zsh completion responses on initialization and errors (#16268)

Decision: **accept** (ported as `69eaa536a`). `warp_main_completer` returns early when `_generic` is unavailable so the client receives a completed empty native response and applies its filepath fallback, and the armed ZLE widget owns the response frame via `warp_mark_start_of_completions_for_compadd_override` + an `always` teardown (end-of-completions marker, `unset COMPADD_OVERRIDE`, single-space accept-line) instead of relying on `compprefuncs`/`comppostfuncs` hooks that may never run when the completion system is missing or errors. Cherry-picked cleanly: the fork's `zsh_body.sh` region matched upstream's pre-change state exactly (`zsh -n` verified). The integration test `test_zsh_native_completions_without_compinit_use_filepaths` and its registrations applied verbatim against the fork's identical helper signatures (`execute_command_for_single_terminal_in_tab` 4-arg form, `ExpectedExitStatus`, `write_rc_files_for_test`, `specs_first_completion_defaults`); compile-verified only, since GUI integration tests are headless-unrunnable in this fork.

## Verification

Run on `merge/upstream-2026-10-03` (with `CARGO_PROFILE_DEV_DEBUG=0`):

- `cargo check -p warp --all-targets --message-format short` — pass (pre-existing warnings only).
- `cargo check -p integration --all-targets --message-format short` — pass.
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- `cargo nextest run -p warp -E 'test(inline_history) | test(history_up) | test(slash_command)'` — 93 passed.
- `cargo nextest run -p warp -E 'test(history_up_does_not_reenter) | test(editor_down_does_not_reenter)'` — 2 passed (new tests).
- `zsh -n app/assets/bundled/bootstrap/zsh_body.sh` — pass.
- `cargo build -p warp --all-targets --message-format short` — pass.
- Deleted-surface scans — no new hits; all hits are the pre-existing documentation/test/retained-naming baseline, and zero hits fall inside the merge diff.

`cargo clean` run after merge/tag/push.
