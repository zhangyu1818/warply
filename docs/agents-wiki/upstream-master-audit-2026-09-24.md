# Upstream Master Audit 2026-09-24

Range under review: `71088ba18d..upstream/master` (4 commits)

Previous audited upstream tip: `71088ba18 Report when a cloud agent run has computer use enabled but unavailable (#16114)`

Current upstream tip: `2120e3794 Forward typed query to zsh fzf Ctrl-R history search (#16122)`

Total upstream commits in this incremental range: 4

Status: triage complete. Three ports (one adapted completions fix, two shell-integration fzf Ctrl-R improvements building on the previously ported handoff stack); one commit rejected.

## Per-Commit Triage

### `e865a7437` — Honor Credits/Dollars setting in agent management costs (#16110)

Decision: **reject (no anchors)**. Agent-management run-cost display following the Credits/Dollars setting: `cost_in_cents` threading through `AgentConversationDisplayData` sourced from `usage_totals().charged_usage` and `server_conversation_metadata` usage totals, `AmbientAgentTask::cost_in_cents`, and shared `format_usage`/`usage_label`/`PricingTransparency` rendering across the management list and details pane. The primary consumers (`app/src/ai/agent_management/`, `app/src/ai/ambient_agents/`) are fork-absent, and the fork's persistence model has no `ChargedUsageTotals`/usage-tracking. The two shared files that do exist lack every anchor symbol (verified by search): the fork's `agent_conversations_model/entry.rs` is the minimal fork variant with no `request_usage`/`charged_usage`/`server_conversation_metadata` fields, and `conversation_details_panel.rs` has no `credits`/`charged_usage`/`PricingTransparency` members. Billing/credits surface with nothing separable.

### `d5fdbff50` — Fix flag completions after POSIX double dash (#16124)

Decision: **accept (adapted)**. Retained completions fix, entirely inside `crates/warp_completer/`: a completed standalone `--` ends option parsing for POSIX-compliant signatures (`options_terminated` on `CommandCallInfo`, terminator detection in the legacy parser gated on `flags_are_posix_noncompliant`, `TokenAction::EndOfOptions` stopping subcommand resolution in the signature registry walk, and engine changes so post-`--` dash-prefixed tokens classify as arguments while `cmd --` mid-edit still completes flags). PowerShell-style `flags_are_posix_noncompliant` signatures keep the old behavior.

Port: exact three-way patch for the seven existing files. Recorded adaptations and omissions:

- `signatures/legacy/registry_tests.rs` hunk remapped to the fork's singular `registry_test.rs` (path remap in the applied patch; the new `test_end_of_options_stops_subcommand_resolution` test applied with context intact).
- Import-block conflicts in `engine/test.rs`, `parsers/test.rs`, and `registry_test.rs` resolved keeping the fork's grouped `use crate::{...}` style while adding the upstream `add_content_signature`/`git_signature` testing imports.
- Post-port comparison with the upstream files shows only the standing fork divergences (import grouping, let-chain → nested-if rewrites, removed `cfg!(windows)` `.exe` trimming); all behavioral hunks match upstream.
- The `add_content_signature`/`git_signature` test helpers already existed in the fork's `signatures/testing/legacy.rs`.

Verified: the nine upstream-specified tests pass; full `warp_completer` suite 182 passed, 4 skipped.

### `b42435622` — Constrain external Ctrl-R fzf history picker layout (#16123)

Decision: **accept**. Retained shell-integration fix, `zsh_body.sh` only: the external Ctrl-R fzf history picker now applies fzf's own defaults — `__fzf_defaults "" "${FZF_CTRL_R_OPTS-}"` when the current fzf integration provides it, otherwise an equivalent `--height ${FZF_TMUX_HEIGHT:-40%}` + `FZF_DEFAULT_OPTS` + `FZF_CTRL_R_OPTS` fallback — delivered through `FZF_DEFAULT_OPTS` with `FZF_DEFAULT_OPTS_FILE=''`, keeping Warp's `--scheme=history --tiebreak=index +m` on the command line. Applied cleanly to the fork's fzf-history-widget case (ported 2026-09-03 with `bf2364bc9`); the remaining file diff vs upstream is the standing WSL/ConPTY removal divergence. `zsh -n` passes.

### `2120e3794` — Forward typed query to zsh fzf Ctrl-R history search (#16122)

Decision: **accept**. Retained shell-integration feature building on the two prior ports: `warp_run_external_ctrl_r_widget` now parses the `cursor:hexline` argument (`${1:-0:}` default) via `warp_hex_decode_string` and passes the left-of-cursor text to fzf as `--query="$query"`, and the ctrl-r handoff call site in `terminal/view.rs` flips `capture_cursor` to `true` so the host sends the buffer/cursor snapshot (the fork's `trigger_external_shell_widget_handoff` Replace-mode plumbing from the 2026-09-03/2026-09-23 ports already threads it). Both hunks applied cleanly; `zsh -n` passes.

## Verification

Run on `merge/upstream-2026-09-24` (with `SDKROOT` pinned to the Xcode 26.5 SDK per the standing CLT 27 toolchain prereq, `CARGO_PROFILE_DEV_DEBUG=0`):

- `cargo nextest run -p warp_completer -E '<the nine upstream-specified end-of-options tests>'` — 9 passed.
- `cargo nextest run -p warp_completer` — 182 passed, 4 skipped.
- `zsh -n app/assets/bundled/bootstrap/zsh_body.sh` — pass.
- `cargo nextest run -p warp -E 'test(ctrl_t) | test(external) | test(shell_widget)'` — 46 passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- `cargo check -p warp --all-targets --message-format short` — pass (8 pre-existing lib warnings, same queued-prompts/dead-code categories as the 2026-09-23 audit).
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass.
- `cargo build -p warp --all-targets --message-format short` — pass.

Deleted-surface scans over the merge diff show no new hits; the standing `zsh_body.sh` ConPTY comment mentions predate this range (verified in the 2026-09-23 audit).

`cargo clean` run after merge/tag/push.

## Notes

- The `e865a7437` triage is the standing answer for the APP-6090 cost-display family: the fork's conversation model and details panel carry no usage/credits fields, so future cost/usage-formatting commits need fresh anchors before any port is considered.
