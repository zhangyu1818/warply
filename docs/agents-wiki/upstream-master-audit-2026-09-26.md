# Upstream Master Audit 2026-09-26

Range under review: `43eae5e08..upstream/master` (3 commits)

Previous audited upstream tip: `43eae5e08 Restore focus after closing tab context menus (#16138)`

Current upstream tip: `5af88f49f Preserve model preference across team catalog refreshes (#16029)`

Total upstream commits in this incremental range: 3

Status: triage complete. One port (vertical-tabs group-name search + highlight); two commits rejected with zero fork anchors.

## Per-Commit Triage

### `9fb32eddd` — Make vertical tabs search match and highlight tab group names (#14690)

Decision: **accept (adapted)**. Retained local vertical-tabs UI feature in `app/src/workspace/`: the panel search previously matched only pane/tab text, so a named tab group could not be found by its name (and collapsed groups hid matches entirely). The port adds group-name matching through two helpers (`group_display_name` with the `"New Group"` fallback, `merge_group_name_matches` folding name-matched groups' members into the filtered list ordered by tab index), admits every member of a name-matched group in `matching_tab_indices` so search-driven tab cycling visits exactly what renders, renders surviving groups expanded while a query is active (local clone only; persisted `collapsed` flag untouched), and bold-highlights matching portions of the header title via WarpUI `Text::with_single_highlight`.

Port: exact upstream patch for `app/src/workspace/view.rs`, `app/src/workspace/view/vertical_tabs.rs`, and `app/src/workspace/view/vertical_tabs_tests.rs` applied via `git apply --3way` on the path-filtered upstream diff. Recorded adaptations:

- Omitted paths: `app/src/ai/blocklist/shared_session/sharer/network_tests.rs` (Windows CI reconnect-backoff timing fix — the fork removed the shared-session sharer surface; directory absent) and `specs/GH14689/{product,tech}.md` (upstream specs rejected by contract).
- Placement remapped in `app/src/workspace/view.rs`: the 3-way apply located both `matching_tab_indices(&self.tabs, +&self.tab_groups, ...)` hunks at unrelated `} else {` blocks inside a tab-creation function; the mis-applied hunks were reverted and the exact one-line-per-callsite upstream change (`&self.tab_groups,` argument) applied at the real `activate_prev_tab`/`activate_next_tab` call sites.
- Import blocks adapted: fork's `warpui::elements::{...}` list differs from upstream's (fork imports `DispatchEventResult` separately and lacks upstream-only items); `HashSet` added to `std::collections`, `Highlight` inserted between `Flex` and `Hoverable` per upstream ordering.
- Test-file import adaptation: upstream's new symbols (`group_display_name`, `group_name_highlight_indices`, `matched_group_ids`, `merge_group_name_matches`, `tab_admitted_by_group_name`) inserted into the fork's `super::{...}` list; upstream-only symbols (`VerticalTabsSummaryPrimaryLabel`, `push_normalized_unique_summary_label`, `shows_synced_inputs_indicator`, `sort_summary_primary_labels_status_first`, `label()` helper) not introduced (fork-absent). `use crate::workspace::tab_group::{TabGroup, TabGroupId};` added following the fork's import layout. All 22 appended test bodies are byte-identical to upstream (verified by whole-file diff against `9fb32eddd` — the only remaining diff lines are pre-existing fork divergences: `SummaryPaneKind::Workflow` vs `Notebook`, `is_agent` vs `is_oz_agent`, `local_object_icon_color`, synced-inputs/sort-label refactor remnants).
- No flag adaptation needed: upstream develops under `--features grouped_tabs`, but the fork already runs tab grouping unconditionally (standing GroupedTabs divergence), so the ported code needs no gate.

### `d263c71bb` — Use OZ_RUN_ID for federated issue tokens (#16156)

Decision: **reject**. `oz federate issue-token` is removed cloud-agent CLI surface: the fork's `crates/warp_cli/src/` has no `federate.rs`/`lib_tests.rs` (only `completions.rs`, `config_file.rs`, `json_filter*.rs`, `lib.rs`), and `OZ_RUN_ID`/`federate` have zero references anywhere under `app/`, `crates/`, `script/` (the only `federate` substring hits are `"confederate"` tokens in the ONNX input-classifier vocabulary JSON). Nothing separable for a local terminal/ACP path.

### `5af88f49f` — Preserve model preference across team catalog refreshes (#16029)

Decision: **reject**. The entire change lives in the removed Warp-server model-catalog machinery: `LLMPreferences`, `TeamScope`/`UserWorkspaces::feature_model_choice_for_team_uid`, `AdminDisabled`/`DisableReason` reconciliation, and the server-refresh-driven `update_available_llms` cleanup — model routing belongs to ACP adapter configuration in this fork. The fork's `app/src/ai/llms.rs` is a one-line `pub use ai::LLMId;` re-export stub, `app/src/ai/llms_tests.rs` does not exist, and `LLMPreferences|usable_model_info_for_id|base_llm_for_terminal_view|AdminDisabled|feature_model_choice_for_team_uid` have zero matches under `app/`/`crates/`. No fork anchor to port onto.

## Verification

Run on `merge/upstream-2026-09-26` (with `CARGO_PROFILE_DEV_DEBUG=0`):

- Environment note: the Xcode 26.5 → 27.0 upgrade on this machine dropped the Metal Toolchain asset; `xcodebuild -downloadComponent MetalToolchain` was re-run before the first build (standing environment prereq, not a code regression). No `SDKROOT` pin needed anymore — Xcode 27's default SDK builds fine.
- `cargo nextest run -p warp -E 'test(vertical_tabs)'` — 95 passed.
- `cargo nextest run -p warp -E 'test(/group_name/)'` — 17 passed (the new group-name test family).
- `cargo check -p warp --all-targets --message-format short` — pass.
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — pass.
- `cargo build -p warp --all-targets --message-format short` — pass.

Deleted-surface scans over the merge diff (`app/src/workspace/view.rs`, `vertical_tabs.rs`, `vertical_tabs_tests.rs` only) show zero new hits.

`cargo clean` run after merge/tag/push.

## Notes

- The `9fb32eddd` rejection of `specs/GH14689/**` follows the standing rule that upstream spec/planning documents are not fork memory.
