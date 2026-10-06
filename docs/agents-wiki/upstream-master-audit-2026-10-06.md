# Upstream Master Audit — 2026-10-06

Incremental audit for `6398d1e96..upstream/master` (`c75f182d1`), 6 commits, completing the currently fetched upstream master tip. The previous audit is `upstream-master-audit-2026-10-04.md`.

## Summary

Two adapted ports (the revert of the tab context-menu focus restore and its guarded re-land), four zero-port commits (Namespace build-cache metadata, Oz CLI default-runner, Windows CI test flakes, upstream-owned macOS CI cache volumes). No removed product surface was restored.

## Commit-by-Commit

### `b865631c9` REMOTE-3336: Record Namespace build-cache usage metadata — rejected (fork-absent)

Writes a build-cache usage metadata snapshot during cache-enabled environment preparation. Touches only `crates/build_cache/**` (crate absent in this fork, recorded in the 2026-09-26 audit) and `app/src/ai/agent_sdk/driver/cache_setup.rs` (removed agent SDK surface), plus one `Cargo.lock` line for the absent crate graph. No anchor symbols in the fork; nothing separable — the local terminal paths never consult Namespace volumes.

### `023685490` [REMOTE-3356] Set environment default runners through the Oz CLI — rejected (removed surface)

`oz environment update --default-runner` plumbing across `app/src/ai/agent_sdk/environment.rs`, `environment_tests.rs`, `runner.rs`, and `crates/warp_cli/src/environment.rs`. All paths live in the removed agent-SDK/cloud-environment stack and the trimmed fork `warp_cli`; cloud-environment ownership and revision-aware GSO updates are server-coupled. No fork anchors.

### `8b0e86c51` [REMOTE-3350, APP-5820] Fix Windows CI reconnect and shell completion test flakes — not applicable (fork-absent)

Test-only race fixes in `app/src/shared_session/sharer/network_tests.rs` (shared-session surface removed at baseline) and `crates/warp_tui/src/terminal_session_view_tests.rs` (no `warp_tui` crate in this fork). The Windows Git-Bash completion-warmup fixture and reconnect-await rewrites have no local consumers.

### `dc4442af2` Revert "Restore focus after closing tab context menus (#16138)" — adapted port

Upstream reverted the unconditional `focus_active_tab` in the tab context-menu `Close` handler because `Menu` dispatches the item action (Rename tab/Rename pane focus their editor) *before* emitting `Close { via_select_item: true }`, so the unconditional refocus immediately stole focus from the rename editor. This fork had ported `43eae5e08` verbatim on 2026-09-25, so the regression is live here too.

Ported:

- `app/src/workspace/view.rs`: removed the `self.focus_active_tab(ctx);` line from `handle_tab_right_click_menu_event` — byte-identical to the upstream hunk.
- `app/src/workspace/view_test.rs` (fork remap of upstream `view_tests.rs`): removed `test_closing_tab_context_menu_restores_active_tab_focus`; the removed test body is verbatim upstream, only the surrounding blank-line formatting follows the fork's rustfmt layout.

Omitted: none. The independent stale-tab-index bounds guard in `save_current_tab_as_new_config` is kept, as upstream kept it.

### `0e3101e1e` macOS CI per-job Namespace cache volumes + compile diagnostics — not applicable (fork-owned CI)

Upstream's `.github/workflows/ci.yml` Namespace cache-tag rework and the new `script/macos/report_ci_cache_stats` script target upstream's Namespace runner fleet. The fork's CI workflow is fork-owned and diverged (no Namespace cache profile), consistent with prior audits treating upstream CI-workflow commits as not applicable.

### `c75f182d1` fix: preserve tab rename focus from context menus — adapted port

Re-lands the tab context-menu focus restore behind rename-in-progress guards: `Close` refocuses the active tab only when no tab, pane, or tab-group rename is in progress. Adds four regression tests exercising real menu selection via `MenuAction::Enter`.

Ported:

- `app/src/workspace/view.rs`: the guarded `focus_active_tab` hunk — byte-identical to upstream.
- `app/src/workspace/view_test.rs`: `use crate::menu::MenuAction;` plus the four tests (`test_closing_tab_context_menu_restores_active_tab_focus`, `test_selecting_rename_from_tab_context_menu_preserves_editor_focus`, `test_selecting_rename_from_pane_context_menu_preserves_editor_focus`, `test_selecting_rename_from_tab_group_context_menu_preserves_editor_focus`), verbatim upstream bodies.

Omitted/adapted:

- `let _grouped_tabs_guard = FeatureFlag::GroupedTabs.override_enabled(true);` dropped from the group-rename test — the fork has no `GroupedTabs` flag; tab grouping runs unconditionally (see `fork-upstream-structural-divergences`), matching the standing fork adaptation rule for that gate.
- The `FeatureFlag::VerticalTabs` guard is kept (the flag exists in this fork).

## Verification

Results recorded after the ports settled:

- `cargo fmt -- --check`: clean.
- `cargo check -p warp --all-targets --message-format short`: clean.
- `cargo check --workspace --all-targets --message-format short`: clean.
- `cargo nextest run -p warp -E 'test(selecting_rename_from) | test(test_closing_tab_context_menu_restores_active_tab_focus)'`: 4 passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: passed.
- `cargo build -p warp --all-targets --message-format short`: succeeded, followed immediately by `cargo clean`.
- Restored-deleted-surface scans (`rg` over `app crates script Cargo.toml`): no new hits beyond the previously documented allowances.
