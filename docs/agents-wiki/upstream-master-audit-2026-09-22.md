# Upstream Master Audit 2026-09-22

Range under review: `37a7ebeff..upstream/master` (6 commits)

Previous audited upstream tip: `37a7ebeff Include third-party harness output in failure details (#15991)`

Current upstream tip detected: `f4f9b8838 REMOTE-3075: Test signal origin enrichment (#16081)`

Total upstream commits in this incremental range: 6

Status: triage complete. Two ports (one verbatim cherry-pick, one adapted three-way patch); four commits rejected or not applicable.

## Per-Commit Triage

### `c02a1887c` — REMOTE-3075: Save a handoff snapshot on SIGTERM/SIGINT (#15654)

Decision: **reject**. Cloud agent driver signal handling: first SIGTERM/SIGINT triggers cleanup (handoff snapshot + video-recording save), repeat signals exit immediately, received signals are emitted as OTel trace events, and the polling signal loop is refactored onto `signal-hook-tokio`. All source files live under the removed `app/src/ai/agent_sdk/driver/` (`termination.rs`, `termination/unix.rs`, `unix_tests.rs`, plus `driver.rs`/`error_classification*` edits); the only other change adds the `signal-hook-tokio` dependency that solely feeds that module. The fork has no `app/src/ai/agent_sdk/`, no handoff-snapshot/video-recording/OTel path, and its ACP client owns agent execution with no server-facing termination reporting. Nothing separable.

### `3c520bed7` — Disable IAP for server root URL overrides (#16108)

Decision: **N/A (reject)**. One-hunk change to `crates/warp_core/src/channel/state.rs::override_server_root_url`: an explicit `--server-root-url` override now also clears `server_config.iap_config` so a custom server target skips channel-bundled IAP authentication. The fork's `warp_core` has no `override_server_root_url`, `server_config`, or `iap_config` (verified by search); Warp server URL/IAP configuration is a removed surface with no local or ACP data flow.

### `9c870be87` — Fallback to file paths after empty native completions (#16093)

Decision: **accept (verbatim)**. Retained completions fix in `app/src/terminal/input.rs`: when `CompletionSources::NativeOnly` resolves, the shell result is now awaited first, and only an empty native result set falls back to `completer::suggestions` with `CompletionsFallbackStrategy::FilePaths` and file-path-only suggestions (previously NativeOnly never consulted Warp completions at all). The non-native dispatch's redundant `comp_sources != CompletionSources::NativeOnly` re-check is dropped. The integration-test doc comment in `crates/integration/src/test/native_shell_completions.rs` is updated to match.

Port: `git cherry-pick --no-commit`; patch-id identical to upstream (verified). Native shell completions are retained fork behavior (ported 2026-08-28 from `fc4d563b8`).

### `88cf71732` — REMOTE-3075: Enrich signal trace events with sender origin (#16080)

Decision: **reject**. Same removed surface as `c02a1887c`: adds `siginfo`-based sender-origin (pid/uid) enrichment to the agent-SDK driver's signal trace events in `app/src/ai/agent_sdk/driver/termination*.rs`, plus a `signal-hook` feature bump in `app/Cargo.toml`. Fork-absent surface, no anchors.

### `0efd26e70` — Add Neovim 0.13 line text objects (#16105)

Decision: **accept (adapted)**. Retained vim/editor feature: `TextObjectType::Line` with `il` (current line's non-whitespace content, charwise, `None` on blank lines) and `al` (entire buffer, linewise) across the shared vim FSA, code editor, terminal input editor, and editor model, plus `VimTextObject::motion_type()` consolidating the paragraph-linewise special case. Counts remain unsupported, matching Neovim 0.13.

Port: exact three-way patch of the retained paths; the `crates/warp_tui/src/input/vim.rs` hunk is omitted (crate absent). Recorded adaptations and omissions:

- `crates/vim/src/text_objects/line.rs`: `TextBuffer` import adapted from upstream's `warpui_core::text` to the fork's `warpui::text` crate naming (matches sibling text-object modules).
- `app/src/code/editor/view/vim_handler.rs`: the new `selected_text_for_vim_register` helper (appends `\n` when a Linewise selection reaches `max_charoffset`, trims the leading newline for single selections) and the `has_nonempty_selection` edit gate are ported. The register writes keep the fork's direct `if !selected_text.is_empty()` gate instead of upstream's `register_text` `"\n"`-for-empty-linewise fallback, because that fallback was introduced by the unported prerequisite `71fafb46cf` (TUI vim mode / Replace-session stack, blanket-classified TUI-only in the 2026-07-30 audit). The ported tests' expectations are produced by the helper itself, so behavior is preserved without the prerequisite.
- `crates/vim/src/vim_tests.rs`: the two new line-text-object tests and the `assert_operation_text_object` helper are ported; the conflict region's `assert_replace_char` + Replace-mode tests are upstream context from `71fafb46cf`, not additions of this commit, and stay unported.
- `app/src/editor/view/vim_handler_tests.rs`: applied to the fork's singular `vim_handler_test.rs` (known filename divergence).
- `app/src/editor/view/mod.rs` import conflict: the fork's diverged import block is kept and only `vim_all_lines`/`vim_inner_line` added; same for `app/src/editor/view/model/mod.rs`'s split `use vim::{...}` block.
- `app/src/code/editor/model.rs`, `app/src/editor/view/model/mod.rs`, `app/src/editor/view/mod.rs` functional hunks (Line dispatch arms, `al` linewise snapping/extension, `motion_type()` dispatch) applied as upstream.

Deferred prerequisite noted: `71fafb46cf` carries separable shared-file behavior beyond its TUI core (code-editor `register_text` empty-linewise register fallback, `replace_char` advance + `replace_text` Replace-session handler methods, `vim_replace_text`/`vim_select_to_line`/`vim_move_to_last_line` model helpers, and vim FSA Replace-session tests). Porting the Replace-session stack is a separate focused task if wanted; nothing in this range requires it.

### `f4f9b8838` — REMOTE-3075: Test signal origin enrichment (#16081)

Decision: **reject**. Tests for `88cf71732` in `app/src/ai/agent_sdk/driver/termination/unix_tests.rs`, fork-absent surface.

## Verification

Run on `merge/upstream-2026-09-22` (with `SDKROOT` pinned to the Xcode 26.5 SDK per the standing CLT 27 toolchain prereq, `CARGO_PROFILE_DEV_DEBUG=0`):

- `cargo nextest run -p vim` — 74 passed (includes the four new `line_tests.rs` cases and the two new FSA tests).
- `cargo nextest run -p warp -E 'test(test_vim_line_text_objects)'` — 2 passed (code editor + editor view).
- `cargo check -p warp --all-targets --message-format short` — pass (pre-existing warnings only).
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- `cargo build -p warp --all-targets --message-format short` — pass.

The `native_shell_completions.rs` integration-test change is compile-verified only (GUI integration tests are headless-unrunnable in this environment).

Deleted-surface scans (`access token|AuthState|billing|...`, `mcp.*capab|ReadSkill|...`, `target_os = "linux"|...`) show no new hits: remaining matches are `Weak::upgrade` call sites, SSH `ForwardX11=no` config, retained remote/bootstrap ConPTY comments, and documentation — all allowed categories, unchanged from the standing audits.

`cargo clean` run after merge/tag/push.

## Notes

- The REMOTE-3075 signal-handling stack (`c02a1887c` → `88cf71732` → `f4f9b8838`) is a three-PR agent-SDK cloud-driver line; all three land in fork-absent files and introduce the `signal-hook-tokio` dependency only for that surface. Re-triage only if an equivalent local termination-snapshot behavior is ever routed through ACP.
- The `71fafb46cf` shared-file backfill candidate above is the first recorded case where a previously blanket-rejected TUI commit blocks clean application of a later retained commit; future vim-mode ports should check it first.
