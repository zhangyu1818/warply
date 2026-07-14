# Upstream Master Audit 2026-07-14

Range under review: `a01df387a..upstream/master` (27 commits)

Previous audited upstream tip: `a01df387a Cache TUI transcript block heights to fix long-transcript scroll lag (#13592)`

Current upstream tip detected: `62da4ee72 [CODE-1829] Render agent task lists in the TUI transcript (#13570)`

Total upstream commits in this incremental range: 27

Status: triage complete. Retained code review, slash command, macOS-platform, and completion fixes were ported or adapted manually. TUI-only changes, skills, telemetry, orchestration, agent_sdk/video-recording, and wasm-specific changes were rejected or marked not applicable.

## Docker Sandbox Note

During this review the "Local Docker Sandbox" new-tab menu item was investigated for a "click has no response" report. This is **not** a merge regression — the feature was untouched by recent merges. It is a retained local terminal feature (`sbx`-managed Docker sandbox, not Warp cloud isolation) whose `LocalDockerSandbox` feature flag was intentionally removed in `b5a0e471b1` so the menu item always shows.

The "no response" is by-design silent behavior: `add_docker_sandbox_tab` resolves the external `sbx` CLI binary via `resolve_sbx_path_from_user_shell`; when `sbx` is not installed on the user's machine the spawn callback logs `sbx binary not found; cannot create Docker sandbox` and returns without creating a tab or showing any user-facing error. `sbx` is a user-provided external binary (analogous to `docker`); the app does not download or bundle it.

## Ported Or Adapted

- `fcf1fda8c` Ported the code review pinned file header corner leak fix. `render_file_header` gained an `is_pinned: bool` parameter. When pinned (the sticky overlay drawn on top of the diff while scrolling), the outer backing's corner radius is squared off (`CornerRadius::default()`) so the opaque panel background fills the corner notches that would otherwise let the green/red diff show through. The at-rest card keeps its rounded corners. Both call sites updated; the at-rest call passes `false`, the sticky overlay passes `true`.
- `f9a0bf80e` Ported the imported PR comment context-line outdated fix. Added `LineDiffContent::imported_original_text()` which strips exactly one leading unified-diff marker (`+`, `-`, or the context-line space), recovering the raw file line for imported comments whose stored content is always a diff line. `relocate_comments` in `code_review_view.rs` now routes `ImportedFromGitHub` comments through `imported_original_text()` (instead of `original_text()`, which only strips `+`/`-` and left context-line comments permanently marked outdated). Native comments keep `original_text()` so significant leading indentation is preserved. Added 5 `imported_original_text_*` unit tests in `comment_tests.rs`.
- `5348e67f2` Adapted the repo-gated slash command stale-cache fix. The fork's `SlashCommandDataSource` uses `recompute_active_commands` (not upstream's `base_availability` trait method on `data_source/core.rs`, which does not exist in the fork). Replaced the cached `active_repo_root.is_some()` REPOSITORY derivation with a live `cwd_is_in_repository(ctx)` check gated on `is_local`, using `DetectedRepositories::get_root_for_path` with the same `launch_data().maybe_convert_absolute_path(cwd)` path conversion the fork already uses elsewhere. `active_repo_root`/`set_active_repo_root` are retained as the recompute trigger that fires once async detection caches a newly-entered repo. Skipped upstream's `slash_command_model_tests.rs` regression tests (the fork's `data_source/mod.rs` layout differs and uses a different test harness; the fork has no `simulate_directory_for_completion` helper).
- `b0d37352b` Adapted the Info.plist EventKit usage-description additions. Added `NSCalendarsFullAccessUsageDescription`, `NSRemindersUsageDescription`, and `NSRemindersFullAccessUsageDescription` to `script/update_plist`. Adapted: description strings say "Warply" (the fork's app name), not upstream's "Warp".
- `c2a72dde6` Ported the macOS bootstrap `HOMEBREW_NO_ASK=1` addition. Added the export to `script/macos/bootstrap` before `brew update` so every `brew install` runs non-interactively. Did not port the unrelated `13902088a` headless-bootstrap follow-up (it adds `--skip-gcloud-auth` and touches `script/linux/bootstrap` + `script/windows/bootstrap.ps1`; the fork is macOS-only and the gcloud auth check is retained SSH/remote-server test infrastructure).
- `7cdb02c4e` Ported the `warp-command-signatures` rev bump (`a937ae35d` → `ec1ae8e84`) in `Cargo.toml` + `Cargo.lock`. The crate is used by `warp_completer` and `app` (always-on, not gated behind `completions_v2`), bringing git status parsing fixes for paths with spaces/renames and nested untracked `git add` completion.

## Rejected Or Not Applicable

| Commit | Decision | Reason |
| --- | --- | --- |
| `62da4ee72` | N/A | TUI agent task-list rendering; `crates/warp_tui/` absent. |
| `ec58e691a` | N/A | TUI multiline paste handling; `crates/warp_tui/` absent. |
| `b43821d0a` | N/A | TUI slash command menu styling; `crates/warp_tui/` absent. |
| `3a2b49c73` | N/A | STAKEHOLDERS orchestration ownership; `.github/` is fork-owned. |
| `bff27233a` | Reject | Remove orchestration viewer streamer flags; touches removed `orchestration_event_streamer`, shared-session cloud-agent viewer, and orchestration viewer model. |
| `d1b26eefd` | N/A | TUI transcript viewport-edge selection drag; `crates/warpui_core/src/elements/tui/` primitives may exist but the TUI consumer is absent. |
| `a77348c67` | Reject | Low-effort slash commands for TUI; bulk is TUI/`crates/warp_tui/` rendering + clipboard. The `conversation_export.rs` extraction depends on TUI export paths; `input.rs` refactor splits GUI/TUI behavior. |
| `96d6857e3` | N/A | TUI scroll anchoring; `crates/warpui_core/src/elements/tui/viewported_list.rs` changes serve the TUI only. |
| `992610422` | N/A | Conversation persistence/restoration for TUI; `crates/warp_tui/` absent and the fork's ACP conversation restoration is separate. |
| `0e6871373` | Reject | Restore telemetry emission calls; telemetry removed in fork. |
| `4e8992935` | N/A | Remove leftover `PRCommentsSlashCommand` test references; the fork already removed `/pr-comments`. |
| `13902088a` | N/A | Headless macOS bootstrap (`--skip-gcloud-auth`); touches `script/linux/bootstrap` and `script/windows/bootstrap.ps1` (rejected platforms). The macOS `gcloud auth` check is retained SSH test infrastructure. |
| `2723ae197` | Reject | Share slash command behavior between GUI and TUI; the TUI/GUI split (`tui_export.rs`, `crates/warp_tui/`) is absent. |
| `7607cc7c1` | Reject | Remove `/pr-comments` (superseded by bundled skill); the fork already removed `/pr-comments`, and this commit's bulk depends on removed agent SDK conversion + skills. |
| `cd6c56eb3` | Reject | `tui-verify-change` skill docs; skills rejected. |
| `fc7bca638` | N/A | `cmov` 0.5.3→0.5.4 bump; `cmov` is not in the fork's `Cargo.lock` (TUI/wasm-only dependency). |
| `6d8562c39` | Reject | VA video recording finalize; depends on removed `agent_sdk`, cloud recording, and `crates/computer_use` actor construction from the removed agent SDK flow. |
| `a4a857688` | N/A | MCP tool-call monospace rendering; the `McpJsonTreeView` feature flag, `app/src/ui_components/json_tree.rs`, and the `should_render_mcp_content`/`render_json_tree` code paths do not exist in the fork (app-managed MCP JSON tree rendering was removed). |
| `995e3dd7a` | N/A | TUI stray-character fix from wide-grapheme cursor shift; `crates/warp_tui/` absent. The `crates/editor` and `crates/warpui_core` shared-primitive changes are wasm/TUI-renderer-scoped and have no consumer in the fork. |
| `6edb32152` | Reject | Build bundled skills in `./script/run-tui`; skills rejected and TUI absent. |
| `97a9ff5f9` | N/A | Wasm debug panic in `StandardizedPath::from_local_absolute_unchecked`; the fork is macOS-only (no wasm target), so `Path::is_absolute()` is correct on `cfg(unix)`. The `debug_assert!` never trips natively. |

## Verification

Commands run after porting:

- `cargo check -p warp --all-targets --message-format short` — passed (24.89s, no errors).
- `cargo fmt -- --check` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions) | test(imported_original_text) | test(original_text) | test(relocate_comments)'` — 179 passed, 2394 skipped.
- `cargo nextest run -p warp -E 'test(code_review) | test(comment)'` — 98 passed, 2475 skipped.
- Deleted-surface scan of the full diff for `access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment` — no restored surfaces.
- Deleted-surface scan of the full diff for `mcp.*capab|mcp_server|mcpServers|bundled skills|channel-gated|ReadSkill|InvokeSkill` — no restored surfaces.
- Deleted-surface scan of the full diff for `target_os.*linux|target_os.*windows|cfg(windows)|WSL|MSYS2|ConPTY` — no restored surfaces.
- No new Cargo dependencies were added (only the `warp-command-signatures` git rev advanced; `Cargo.lock` updated via `cargo update -p warp-command-signatures`).
