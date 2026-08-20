# Upstream Master Audit 2026-08-20

## Scope

- Current fork before this audit: `867d274bc` (`main`, `v2026.08.19`).
- Upstream source reviewed: `8ba01aa1a8..upstream/master` (18 commits, tip `4e49d04f5a`).
- Result: 1 commit accepted (`ee95ac0fd`), 4 commits partially ported as adaptations (`04a7f8342`, `27f8ee6c1`, `cff5f778c`, `216d0efe7`, `4e49d04f5` counted as five ports across four commits plus the accept); 13 rejected or not applicable.

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `dff214ab2` | Stop interpolating uid into parse_server_gso Sentry grouping message (#15291) | **Not applicable** | `app/src/server/server_api/object.rs` and all of `app/src/server/` are removed (Warp server API). |
| `d68a638ef` | Fix report_error! demotions of typed errors into extra: (APP-5522) (#15298) | **Not applicable** | Every hunk edits a `report_error!` call form or a site the fork lacks. On retained files (`agent_input_footer`, `commit.rs`, `default_terminal`, `terminal_pane`, `terminal/view.rs`, `workspace/view.rs`, `theme_creator.rs`) the fork's baseline conversion already uses `log::error!` with the message inlined; the MAA send site, docker-sandbox double-report site, `launch_remote_child`, geap credentials, orchestration, windows single-instance, and WSL/MSYS2 `crates/ai/src/paths.rs` sites have no anchor in the fork (fork `paths.rs` is 57 lines without host-path conversion). |
| `d019ddfe9` | Stop reporting unknown AmbientAgentSource to Sentry (APP-5521) (#15295) | **Not applicable** | `app/src/ai/ambient_agents/` removed. |
| `0209de56e` | review-pr-local: make report_error! form audit mandatory (APP-5522) (#15296) | **Not applicable** | Fork's `.agents/skills/review-pr-local/SKILL.md` is a fork-owned copy (references `WARP.md`, lacks the log-macro bullet upstream edited); the fork has no `report_error!` macro to audit. |
| `d80cffdec` | Preserve voice transcription transport errors' cause and actionability (#15290) | **Not applicable** | `app/src/server/server_api.rs` and voice input are removed. |
| `02fe4a9de` | Classify GraphQLError actionability so 408s stop reaching Sentry (APP-5168) (#15292) | **Not applicable** | `crates/graphql` removed. |
| `04a7f8342` | Downgrade key-binding responder-chain report_error! to log::error! (#15299) | **Adapt (ported)** | The fork's baseline already keeps both `contexts_for_responder_chain` failure sites at `log::error!` in the retained `crates/warpui_core/src/core/app.rs`; ported the exact upstream final message form `{error:#}` (cause-chain expansion) at both sites. See provenance below. |
| `27f8ee6c1` | Fix Sentry data exposure: PII/typed-content leaks missed by #15298's merge race (APP-5522) (#15300) | **Adapt (partial port)** | Only the `crates/ai/src/index/file_outline/native.rs` hunk applies: the oneshot-send failure no longer dumps the unsent outline payload (`{e:?}`) into the log; adapted from `report_error!`/`OncePerRun` to the fork's `log::error!` with the same static message. The MAA `response_stream` site, `geap_credentials` files, and wasm-only winit event-loop file are absent. See provenance below. |
| `1a29f680d` | Share one bounded recovery budget between MAA retries and resumes (REMOTE-2269) (#15293) | **Not applicable** | Entirely anchored on the removed old Warp Agent MAA backend: no `recovery_action`, `retry_count`, `schedule_auto_resume`, `PendingResume`, `RecoveryBudget`, or MAA send site anywhere in the fork's blocklist; `server/retry_strategies.rs`, `warp_tui`, `response_stream_tests.rs`, and `specs/REMOTE-1894` are absent. The fork's `response_stream.rs` is a small ACP-era identity type, a different implementation. |
| `be11be65d` | Fix settings search showing (1) on monolithic Code Indexing and Profiles sub-pages (APP-5530) (#15319) | **Not applicable** | `app/src/settings_view/agent_profiles_page.rs`, `code_indexing_page.rs`, and `.agents/skills/gui-settings-ui/` do not exist in the fork. |
| `def3fd0e3` | Bump warp-proto-apis pin for per-category charged usage (#355, #362) (#15137) | **Not applicable** | Billing/credits usage plumbing: no `warp-proto-apis` pin in the fork's `Cargo.toml`; `crates/ai/src/api_keys.rs`, `controller/shared_session.rs`, and the `platform_credits_spent`/`conversation_usage_metadata` anchors in `app/src/ai/agent/conversation.rs` are absent. |
| `33b59410b` | Promote WaitForEventsParentRegistration and OrchestrationUnifiedStack to Stable (#15321) | **Not applicable** | Neither feature flag exists in the fork (`app/src/features.rs`, `crates/warp_features/src/lib.rs`); orchestration is a removed surface. |
| `e4857bd60` | Honor server-provided repository head overrides (#13718) | **Not applicable** | Core lives in `app/src/ai/agent_sdk/driver/*` and `cloud_environments` (removed). The only retained-file hunk (`docker_sandbox/mod.rs`) changes a `prepare_environment` call imported from the removed agent_sdk driver; the fork's docker sandbox has no such import or call. `crates/warp_cli/src/agent.rs` is a removed CLI surface. |
| `fbb7e01d5` | Strip narration comments left on master by #15298 (APP-5522) (#15302) | **Not applicable** | The narration comments live in `full_source_code_embedding/*` (absent) and the docker-sandbox double-report site (absent); nothing to strip in the fork. |
| `ee95ac0fd` | Fix double cursor in finished background blocks (CORE-3798) (#15322) | **Accept (ported)** | Retained terminal rendering fix via `git cherry-pick --no-commit` with conflict resolution. See provenance below. |
| `cff5f778c` | lint_powershell: report findings from every source (plus Get-Location false-positive cleanup) (#15316) | **Adapt (partial port)** | Ported the `app/assets/bundled/bootstrap/pwsh.ps1` hunks: `(Get-Location).Path` → `$PWD.Path` (4 sites). `script/lint_powershell` and `script/windows/*` do not exist in the fork. See provenance below. |
| `216d0efe7` | Improve scroll performance for blocklists with many AIBlocks. (#15280) | **Adapt (partial port)** | Ported the tooltip no-op-dismissal elimination (`block.rs` `dismiss_ai_tooltips`, `secret_redaction.rs` `dismiss_tooltip`, `search_codebase.rs` `clear_link_tooltip` — notify/emit only when a tooltip was actually open). Omitted the recording-span cache (`action_model.rs`, the `block.rs` `has_recording_related_actions` invalidate hunk, `output.rs` render hunk): the fork has no recording/computer-use infrastructure and no anchors. See provenance below. |
| `4e49d04f5` | Follow symlinks to directories in WSL Tab completion, via guest-driven enumeration (APP-3993) (#14755) | **Adapt (partial port)** | Ported the shared-parser rework that benefits retained `WarpifiedRemote` (SSH/remote) Tab completion: `parse_ls_script_output`/`dir_entry_from_segment` operating on raw bytes (a single non-UTF-8 filename no longer empties the whole listing; truncated output is rejected instead of misparsed), the restructured `match command_output_result` remote branch, and the 8 new tests. Omitted all WSL-specific parts (`wsl_guest_listing.rs` module, `#[cfg(windows)]`/`is_wsl()` gate, its tests) — the fork is macOS-only with no WSL local host. See provenance below. |

## Provenance: `ee95ac0fd` port detail

Applied with `git cherry-pick --no-commit`; `app/src/terminal/model/block.rs` applied cleanly, `app/src/terminal/block_list_element.rs` resolved on the applied upstream source:

- `block.rs`: upstream hunks verbatim — new `is_command_cursor_visible`/`is_output_cursor_visible` methods after `set_was_long_running`.
- `block_list_element.rs`: all upstream change lines present verbatim — `use super::model::ansi::CursorShape;` import, `TermMode` dropped from the `grid_handler` import (no remaining uses), removal of the `cursor_visible` local, `command_grid_visible_cursor_shape(block)`/`output_grid_visible_cursor_shape(block)` passed to both grid `draw` calls, both `draw_cursor` conditions replaced with the new methods, and the two helper functions after the impl block.

Intentionally omitted/adapted upstream context (not change hunks): the fork's `render_block` kept its flat prompt/command structure; the upstream patch context wraps the same code in `let command_origin = if !block.should_hide_command_grid() { ... }` — a later upstream refactor absent from the fork. The fork's output-grid `draw_cursor` body (cursor hint, agent-blocked/agent-in-control colors) is retained local AgentView rendering, unchanged by the upstream patch.

## Provenance: `04a7f8342` port detail

The fork's baseline already converted both `contexts_from_responder_chain` failure sites in `crates/warpui_core/src/core/app.rs` from `report_error!` to `log::error!` (the upstream commit's own end state). Ported the exact upstream final message form — `log::error!("Unable to fetch Key Bindings for View: {error:#}")` — at both `contexts_for_window_and_view` and `editable_bindings_for_view` sites, so the log now carries the error cause chain per upstream.

## Provenance: `27f8ee6c1` port detail (native.rs hunk only)

On the retained `crates/ai/src/index/file_outline/native.rs` background-outline send site, ported the upstream final semantic: `if sender.send(result).is_err()` reporting the static message "Could not send result of outline generation to background thread" without the payload. The fork's baseline conversion had interpolated `{e:?}` (the unsent `HashMap` payload — file-outline content) into the log; upstream's fix removes it. `warp_errors::ReportErrorLogMode::OncePerRun` is expressed away by the fork's `log::error!` conversion (no `warp_errors` crate).

## Provenance: `cff5f778c` port detail

Applied the exact upstream `pwsh.ps1` hunks via three-way patch: `$newTitle = $PWD.Path`, `pwd = $PWD.Path`, and `$dir = Get-Item -LiteralPath $PWD.Path`. The `$nodeCacheKey` line of the upstream context belongs to the node-version-cache mechanism the fork's `pwsh.ps1` does not have (conflict resolved by keeping the fork's uncached walk-up loop with only the `Get-Item` argument changed). `script/lint_powershell` and `script/windows/*` paths are absent from the fork and omitted.

## Provenance: `216d0efe7` port detail

Ported from the exact upstream patch:

- `app/src/ai/blocklist/block/secret_redaction.rs`: `dismiss_tooltip` now returns `bool` via `.take().is_some()` (verbatim).
- `app/src/ai/blocklist/inline_action/search_codebase.rs`: `clear_link_tooltip` returns whether it cleared and only calls `ctx.notify()` then (verbatim).
- `app/src/ai/blocklist/block.rs`: only the `dismiss_ai_tooltips` hunk (applied at offset −7): conditional `DismissLinkTooltip`/`DismissSecretTooltip` emits, accumulated `dismissed_search_tooltip`, and `ctx.notify()` only when any tooltip was dismissed; the hover-state reset loop stays unconditional per upstream.

Intentionally omitted upstream hunks (no anchors in the fork): the entire `action_model.rs` recording-spans cache (`BlocklistAIActionModel` has no recording infrastructure; `RecordingSpanInfo`, `StartRecording`, `UseComputer`, `has_recording_related_actions` all absent), the `block.rs` `has_recording_related_actions` invalidate hunk, and the `view_impl/output.rs` render hunk (no `recording_spans_by_action_id` call site, no `TODO(vkodithala)` comment).

## Provenance: `4e49d04f5` port detail

Ported from the exact upstream patch on `app/src/completer/mod.rs`:

- The remote `WarpifiedRemote` branch restructure: `match command_output_result { Ok(command_output) => match command_output.status { ... } , Err(err) => log::warn!("Executing `ls` on remote box failed with error {err:?}") }` with `parse_ls_script_output(command_output.output()).unwrap_or_else(|| ... "malformed or truncated output" ...)` (verbatim; the region is byte-identical to upstream's final form).
- New `parse_ls_script_output` and `dir_entry_from_segment` functions with upstream doc comments (verbatim) after `ls_script_for_dir`.
- `app/src/completer/test.rs`: the import update (`use typed_path::{TypedPath, TypedPathBuf};`) and all 8 new tests appended verbatim (`test_ls_script_for_dir_builds_the_expected_command` plus 7 `parse_ls_script_output` tests).

Intentionally omitted upstream paths/hunks: `mod wsl_guest_listing;`, the `#[cfg(windows)] if self.session.is_wsl()` local-branch gate, `wsl_guest_listing.rs`, and `wsl_guest_listing_tests.rs` — WSL local host is a removed platform surface; the fork has no `is_wsl()` or `\\wsl$` path handling. The upstream test-hunk's `#[cfg(windows)] use typed_path::...` context line does not exist in the fork's test file.

## Verification

- `cargo fmt -- --check`: clean.
- `cargo check -p warp --all-targets --message-format short`: pass (pre-existing dead-code warnings only, none in touched files).
- `cargo check --workspace --all-targets --message-format short`: pass.
- `cargo nextest run -p warp -E 'test(parse_ls_script_output) | test(ls_script_for_dir)'`: 8/8 passed (new tests).
- `cargo nextest run -p warp -E 'test(completer)'`: 19/19 passed.
- `cargo nextest run -p warp -E 'test(terminal::model::block::)'`: 45/45 passed (same suite upstream reports).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: 156 passed.
- `terminal::model::blocks::selection` smart-selection test failures reproduced on clean `HEAD` (verified via `git stash`) — pre-existing, unrelated to this port.
- `cargo build -p warp --all-targets --message-format short`: succeeded (`warply` binary produced); `cargo clean` after the release push.
- Deletion-surface scans: MCP/skills scan 0 hits; broad removed-area scan hit set unchanged from the `v2026.08.19` baseline (none of the ported files appear); platform scan hits are retained SSH/remote-terminal paths (`ForwardX11=no` SSH options, zsh bootstrap ConPTY comments for remote hosts, pre-existing `#[cfg(windows)]` test cfg in `warp_util`).
