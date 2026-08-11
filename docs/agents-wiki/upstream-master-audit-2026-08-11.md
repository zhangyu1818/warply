# Upstream Master Audit 2026-08-11

Range under review: `7d93fa468..upstream/master` (22 commits)

Previous audited upstream tip: `7d93fa468 [QUALITY-1333] Prevent background TUI agents from stealing focus (#14829)`

Current upstream tip detected: `683d40782 [CODE-1946] Multi-level orchestration UI in the Warp TUI (#14884)`

Total upstream commits in this incremental range: 22

Status: triage complete. 5 commits ported (2 accept, 3 adapt), 1 already applied, 16 rejected/not applicable.

## Ported Commits

### `dd9ccd6f1` — Downgrade zero-size resize log from info to debug (#14897)

Decision: **accept**

Direct one-line log-level change in `resize_internal` (`app/src/terminal/view.rs`): `log::info!` → `log::debug!` for the zero-size resize guard. The guard recurs on every layout pass when a pane is collapsed, flooding release logs at frame rate. Control flow unchanged; only the log level corrected.

### `738c25bcc` — Fix Find in File search field being unclickable after Enter in Vim mode (#14836)

Decision: **accept**

Ported the full change to `app/src/code/editor/find/view.rs`: added `FindAction::FocusFindInput`, `find_editor_mouse_state`/`find_editor_position_id` fields, `activate_find_input` helper, wrapped the find input in `Hoverable` + `SavePosition` so a click re-activates a Vim-disabled field, and refactored `on_focus` to reuse the helper. The test file `vim_handler_tests.rs` was not ported (upstream tests use Linux `Presenter` rendering helpers not exercised in this fork's test setup).

### `724579a87` — Fix code review pane "Unable to load file content" for renamed files (#14655)

Decision: **adapt**

Upstream edits `app/src/code_review/diff_state/local.rs`, which does not exist in this fork — the logic lives in `app/src/code_review/diff_state.rs` instead. Ported the two git-argument fixes to the fork's `diff_state.rs`:

1. `get_file_content_at_head`: added a `GitFileStatus::Renamed { old_path }` branch that reads `HEAD:<old_path>` instead of falling through to `HEAD:<file_path>` (which fails because the rename is uncommitted).
2. `get_file_diff` (no-commit/head branch): changed the `Renamed` diff args from index-only (`git diff -- <file_path>`) to working-tree-vs-HEAD with both paths (`git diff HEAD -- <old_path> <file_path>`), so a staged rename+modify produces a non-empty diff.

The upstream tests in `local_tests.rs` were not ported (file absent); the fork's `diff_state_tests.rs` covers the existing renamed-file paths.

### `a4769955f` — Kill generator process groups on cancellation (#14853)

Decision: **adapt**

Replaces `spawned_children_pids: HashSet<u32>` with `ActiveProcessGroups`/`SpawnedChildCleanup` in `app/src/terminal/model/session/command_executor/local_command_executor.rs`. Both cancellation paths (per-command future drop via `SpawnedChildCleanup::drop`, and session-wide `cancel_active_commands` via `cancel_all`) now converge on the same idempotent `cancel(&record)` that sends `SIGKILL` to `-pgid`. This fixes zombie wrapper/child processes left behind when prompt-chip generator futures are dropped.

Adaptations from upstream:
- `terminate_process_group` uses `safe_warn!` instead of `log::warn!` to match the fork's safe-logging convention.
- No `#[cfg(test)] mod tests` reference added (the upstream test file `local_command_executor_tests.rs` does not exist in this fork).
- No `#[cfg(windows)]`/`#[cfg(not(unix))]` gates (the fork's `CommandBuilder` has no Windows branch and the maintained target is macOS).

### `7469abe37` — Fix warpctrl file.open relative paths and the crash on retrying or closing a failed markdown pane (#14832)

Decision: **adapt** (partial port)

Ported only the macOS-fatal safety guard to `crates/watcher/src/lib.rs`: the new `ensure_watchable_path` function rejects an empty path before it reaches `register_path`/`unregister_path`, preventing the `notify` FSEventWatcher `CFRelease(NULL)` trap that killed the process on macOS. Both methods now route through `ensure_watchable_path(path).and_then(...)` and log the combined error.

Not ported:
- The `warpctrl file.open` relative-path resolution in `app/src/local_control/handlers/app_state.rs` — the `local_control` module does not exist in this fork.
- The `WatcherType::Individual(PathBuf)` refactor in `crates/warp_files/src/lib.rs` and the `release_file_model`/reload lifecycle changes in `app/src/notebooks/file/mod.rs` — these are a larger refactor tightly coupled to the warpctrl entry point and the upstream `local_control` app_state handler. The empty-path guard in the watcher is the boundary that prevents the crash from any caller, which is the safety-critical part.

## Already Applied

### `2655ec7a8` — Skip the Frameworks rpath add when it is already present (#14873)

Decision: **already applied** in `baadc6cac` (2026-08-10 merge). The fork's `script/macos/add_framework_rpath` helper already exists and is wired into `script/macos/bundle` and `script/macos/run`. The fork's copy diverges from upstream's comment text (references Sparkle instead of Sentry, since the fork removed Sentry.framework).

## Rejected / Not Applicable

### `4f15a21ba` — Drop the existence guard from the Factory definition checkout clone (#14852)

Decision: **not applicable** (removed agent_sdk surface). The touched files (`app/src/ai/agent_sdk/driver.rs`, `driver/environment.rs`) do not exist in the fork.

### `b076027de` — Remove exclamation mark from model discount chip copy (#14876)

Decision: **not applicable** (removed `/model` selector surface). The touched file `app/src/terminal/input/models/data_source.rs` does not exist in the fork; no `discount_percentage`/model-discount chip code is present.

### `b83d10500` — Show the platform a cloud run executes on in run details (#14790)

Decision: **not applicable** (removed cloud run / agent_sdk / cloud environment surfaces). The touched files `app/src/ai/agent_sdk/runner.rs`, `runner_display.rs`, `crates/cloud_object_models/src/cloud_environment.rs` are absent. The shared-file edits to `conversation_details_panel.rs` and `ai/mod.rs` have no anchor: `rg "runner|agent_config_snapshot|AmbientAgentEnvironment|default_runner"` in the fork's `conversation_details_panel.rs` returns no hits. `Icon::Apple` was not added because the platform-row UI that needs it does not exist.

### `f858ae4b8` — Hide shared-session copy link until a link exists (#13813)

Decision: **reject** (removed cloud Warp Drive shared-session surface). The touched files `app/src/drive/sharing/dialog/`, `app/src/terminal/shared_session/`, and the sharing/copy-link symbols (`session_id_for_link`, `SharedSessionManager`, `NotShared`, `FinishedViewer`) are absent. The shared files that exist (`pane_group/pane/mod.rs`, `tab.rs`, `terminal/view.rs`, etc.) have no `session_id_for_link`/`copy_link` anchor.

### `683d40782` — Multi-level orchestration UI in the Warp TUI (#14884)

Decision: **not applicable** (`crates/warp_tui/` is absent and cloud orchestration surfaces are removed). 100% of the diff lives in `crates/warp_tui/` or `specs/`.

### `c2e7d045c` — Update factory-mcp skill: add create_factory and message_foreman tools (#14906)

Decision: **reject** (removed app-bundled skills). The touched files under `resources/bundled/skills/factory-mcp/` do not exist in the fork.

### `e45bcb420` — Nudge review agent to audit diff comments against AGENTS.md guidance (#14902)

Decision: **reject** (documentation structure divergence). The upstream bullet references the "Comments" guidance under "Development Guidelines" in `AGENTS.md`. This fork's `AGENTS.md` is the fork-contract entry point and does not contain a "Comments" section; the coding-style comment rule lives in `WARP.md` (single bullet). Porting the upstream bullet would reference a section that does not exist in the fork's documentation structure.

### `7eee2f802` — fix(winit): apply the shaper's GPOS offsets when converting shaped glyphs (#14322)

Decision: **not applicable** (no winit platform). The touched files `crates/warpui/src/windowing/winit/fonts/text_layout.rs` do not exist; the fork uses the macOS Metal platform (`crates/warpui/src/platform/mac/text_layout.rs`), which has its own glyph-positioning path.

### `4166259c0` — Fix flaky build_cache process_runner classification test (#14892)

Decision: **not applicable** (removed build_cache crate). `crates/build_cache/` does not exist in the fork.

### `aa9f3a436` — Run context chips only for active input surfaces (#14854)

Decision: **deferred** (large refactor, missing dependency, lower priority). The change restructures `CurrentPrompt` to track three independent active surfaces (prompt, agent footer, CLI agent footer) via `ActiveChipSurfaces` and subscribes to `AgentViewController`/`CLIAgentSessionsModel`/`AISettingsChangedEvent::ShouldRenderCLIAgentToolbar`. The fork's `current_prompt.rs` uses a simpler `agent_footer_chip_tracking_enabled: bool` model, and `AISettingsChangedEvent::ShouldRenderCLIAgentToolbar` does not exist in the fork's `AISettings`. The core zombie-process problem this addresses alongside `a4769955f` is already fixed by the process-group kill port; the chip-surface gating is a secondary optimization that can be revisited if the fork adopts the `ShouldRenderCLIAgentToolbar` setting.

### CI/Dependabot bumps (`4be6998cf`, `6be115977`, `4156d8e2e`, `01fb0dc0d`, `98a2a716e`, `9167a6445`)

Decision: **not applicable**. The fork's CI is simplified to `.github/workflows/ci.yml` and `create_release.yml`. The bumped workflow files (`feature_flag_cleanup.yml`, `changelog_draft.yml`) do not exist, and the fork's `ci.yml` does not use `dorny/paths-filter`. The `create_release.yml` uses hash-pinned actions rather than tag-pinned. `crates/warp_graphql_schema/` (the `js-yaml` bump target) is absent.

## Verification

- `cargo check -p warp --all-targets --message-format short` — passed.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo fmt -- --check` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- `cargo nextest run -p warp -E 'test(code_review) | test(find) | test(code::editor)'` — 228 passed, 1 pre-existing failure (`test_find_url_omits_trailing_periods`, confirmed failing on `main` before this merge).
- Deleted-surface scans re-run: only allowed hits (`upgrade()` weak-handle calls, tokenizer JSON vocabulary, retained SSH `ForwardX11=no`, retained `ConPTY` explanatory comment).

## Notes

- The `7469abe37` partial port (watcher empty-path guard only) is the safety-critical boundary: it prevents the macOS `CFRelease(NULL)` crash from any caller, including future code paths that might produce an empty watch directory. The full `WatcherType::Individual(PathBuf)` refactor and warpctrl relative-path resolution were not ported because they depend on the removed `local_control/handlers/app_state.rs` entry point.
- The `aa9f3a436` deferral is tracked here so a future merge can revisit it if the fork adopts the `ShouldRenderCLIAgentToolbar` setting or needs the finer-grained chip-surface gating.
