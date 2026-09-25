# Upstream Master Audit 2026-09-25

Range under review: `2120e3794..upstream/master` (2 commits)

Previous audited upstream tip: `2120e3794 Forward typed query to zsh fzf Ctrl-R history search (#16122)`

Current upstream tip: `43eae5e08 Restore focus after closing tab context menus (#16138)`

Total upstream commits in this incremental range: 2

Status: triage complete. One port (tab context menu focus restore, tests remapped to the fork's `view_test.rs`); one commit rejected.

## Per-Commit Triage

### `df5cacf89` — docs: document WARP_SKIP_COMMON_SKILLS_INSTALL opt-out for script/run prompts (#15530)

Decision: **reject**. Docs-only commit adding one line to upstream `AGENTS.md`'s Platform Setup list documenting `WARP_SKIP_COMMON_SKILLS_INSTALL=1` for the common-skills installer. The fork removed the common-skills surface entirely: no `common-skills/` checkout, no `skills-lock.json`, and zero `WARP_SKIP_COMMON_SKILLS_INSTALL`/`common-skills` references anywhere under `script/` (verified by search), so the documented env var would describe behavior that does not exist here. The fork's `AGENTS.md` is a diverged fork-contract document with no Platform Setup section. Same standing rejection as previous upstream guidance/AGENTS.md commits.

### `43eae5e08` — Restore focus after closing tab context menus (#16138)

Decision: **accept (adapted)**. Retained local tab UI fix in `app/src/workspace/`: closing a tab context menu left the hidden `Menu` view focused, so later keystrokes (`Select(Next)` + `Enter`) dispatched stale positional actions (e.g. `SaveCurrentTabAsNewConfig(5)` after the tab list shrank). Fix: `focus_active_tab(ctx)` in the `MenuEvent::Close` arm of `handle_tab_right_click_menu_event`, plus a checked `self.tabs.get(tab_index)` early return in `save_current_tab_as_new_config` as defense in depth.

Port: exact upstream patch for `app/src/workspace/view.rs` (both hunks applied cleanly via the path-filtered upstream diff; the whole-file `--3way` attempt produced spurious conflicts from unrelated fork hunks and was discarded). Recorded adaptations:

- Test hunk remapped from upstream `app/src/workspace/view_tests.rs` to the fork's `app/src/workspace/view_test.rs` (singular, the fork's standing test-filename divergence). Test bodies applied verbatim from upstream.
- Placement remapped: upstream inserts the two tests between `test_close_other_tabs_confirmation_dialog` and `test_close_tabs_right_confirmation_dialog`, neither of which exists in the fork; the fork's insertion point is after `test_close_last_vertical_tab_activates_tab_above` (the fork's close-tab test family).
- The checked-lookup hunk lands on the fork's `#[cfg(feature = "local_fs")]` real implementation; the fork's `not(local_fs)` no-op stub is unchanged (upstream has no such stub — fork-only cfg split).
- No other divergence: post-port comparison of the two test bodies against the upstream `+` lines shows them identical.

Verified: the two new tests pass; the full `workspace::view::tests` + `/tab/` selection passes 309/309.

## Verification

Run on `merge/upstream-2026-09-25` (with `SDKROOT` pinned to the Xcode 26.5 SDK per the standing CLT 27 toolchain prereq, `CARGO_PROFILE_DEV_DEBUG=0`):

- `cargo nextest run -p warp -E 'test(test_save_current_tab_as_new_config_ignores_stale_tab_index) | test(test_closing_tab_context_menu_restores_active_tab_focus)'` — 2 passed.
- `cargo nextest run -p warp -E 'test(workspace::view::tests) + test(/tab/)'` — 309 passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- `cargo check -p warp --all-targets --message-format short` — pass (8 pre-existing lib warnings, same queued-prompts/dead-code categories as the 2026-09-24 audit).
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass.
- `cargo build -p warp --all-targets --message-format short` — pass.

Deleted-surface scans over the merge diff show no new hits (the diff touches only `app/src/workspace/view.rs` and `app/src/workspace/view_test.rs`).

`cargo clean` run after merge/tag/push.

## Notes

- The `df5cacf89` rejection also covers any future common-skills/`skills-lock.json` guidance commits: the fork has no common-skills infrastructure to document against.
