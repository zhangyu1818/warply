# Upstream Master Audit 2026-08-23

## Scope

- Current fork before this audit: `5c741026c` (`main`, `v2026.08.22`).
- Upstream source reviewed: `9e8ba7341..upstream/master` (8 commits, tip `dc1077845f`).
- Result: 2 commits accepted or adapted (four latent shell-integration fixes, warpui_core update/spawn monomorphization reduction), 6 rejected or not applicable (four multi-team/Teams commits, cloud-agent env clone flags, TUI team statusline, CodeForge forward-compat deserialization).

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `e722ebeda` | Fix four latent shell-integration bugs (split out from native shell completions) (#15428) | **Adapt (ported)** | See provenance below. All four fixes are retained terminal/shell-integration/completions behavior. |
| `e2a080210` | [multi-team P0] TeamContext / TeamContextForOperation infrastructure (#15439) | **Reject** | Teams foundation: only touches `app/src/workspaces/user_workspaces.rs` (+tests), absent from the fork. The `TeamScope`/`TeamContext`/`TeamContextForOperation` machinery exists to resolve per-team effective settings server-side; no local or ACP path consumes it. |
| `19548aec6` | [multi-team P11] Resolve the AI blocklist's team scope from its surface's current window (#15441) | **Reject** | Its purpose is resolving which team's policy applies to a blocklist surface (`team_context`/`TeamContextForOperation` threading through `BlocklistAIController`). The fork's blocklist controller has no team scope; `user_workspaces.rs` and `crates/warp_tui/` anchors are absent. Retained local AgentView/ACP request flow is unchanged by the omission. |
| `61be7e3c3` | [multi-team] /team switcher so the agent CLI has a team scope (#15448) | **Reject** | Adds a `/team` slash command and team-switcher dropdown so the agent CLI resolves a team scope; Teams surface with no local runtime owner. |
| `2c1f0cdfb` | Cloud agent env clones: use --filter=blob:none instead of --filter=tree:0 (#15281) | **Reject** | Only touches `app/src/ai/agent_sdk/driver/environment.rs` (+tests) — removed cloud-agent SDK surface. |
| `702aa106e` | [multi-team] Show the active team in the TUI statusline (#15452) | **Not applicable** | `crates/warp_tui/` absent. The `app/src/settings/ai.rs` hunk only adds a `Team` variant to `TuiStatuslineItem`, which does not exist in the fork (grep confirmed zero hits; the TUI statusline settings went with the TUI crate). |
| `9424410b2` | Make client CodeForge deserialization forward-compatible (fixes repo-less run "environment not found") (#15456) | **Not applicable** | Only touches `app/src/ai/agent_sdk/driver/environment.rs` and `crates/cloud_object_models/` (+tests); both crates/paths are absent from the fork (agent SDK and cloud-environment surfaces removed). |
| `dc1077845` | Reduce monomorphization in warpui_core update and spawn paths (#15453) | **Adapt (ported)** | See provenance below. Retained GPUI/Warp UI framework core; mechanical compile-time refactor with identical semantics. |

## Provenance: `e722ebeda` port detail

Upstream fixes ported verbatim in structure:

1. `warp_completer` `Span::slice` clamps to valid UTF-8 char boundaries (`floor_char_boundary` helper, `start.min(len)`/`end.min(len)` clamps, `end.max(start)`) — applied cleanly to `crates/warp_completer/src/meta.rs`; all 5 new tests copied into the fork's `meta_test.rs` (upstream filename `meta_tests.rs`; fork keeps the singular test filename; file content is otherwise byte-identical to upstream post-change).
2. `pty_controller` `split_kill_buffer_write` helper (PowerShell-only, prefix-validated chord split) + the two-write dispatch in `send_write_to_event_loop` — same placement as upstream (after `on_write_fn`, before the final single write).
3. bash `warp_hex_encode_string`: `echo "$1"` → `printf '%s' "$1"` — applied cleanly.
4. fish `warp_hex_encode_string`: `echo "$argv"` → `printf '%s' "$argv"`, and `warp_preexec` generator-job kill fix (`if not string match -q ... -- (string trim -- $argv[1])`, `kill -9 $pid`) — applied cleanly; all 4 new `split_kill_buffer_write` tests applied cleanly to `pty_controller_command_bytes_tests.rs`.

Fork adaptations (conflict resolution on the applied upstream source):

- `send_write_to_event_loop` in the fork destructures a 4-tuple carrying the fork's `raw_tmux_command` (fork-owned `PtyWrite::TmuxCommand` / `TmuxControlMode` support, which upstream master does not have). The upstream `shell_type_for_split` was added as a 5th element: `Command` arms pass `Some(shell_type)`, every other arm (including `TmuxCommand`) passes `None`.
- The fork's `send_write_to_event_loop` returns `()` (upstream returns `bool`), so the split branch uses `return;` instead of upstream's `return true;`.
- Runtime note: in the fork the split check runs after the tmux control-mode formatting pass. `split_kill_buffer_write` validates the kill-buffer prefix, so a tmux-wrapped payload (which no longer starts with the raw chord) falls back to the whole-write path — conservative and never mis-split, per upstream's own prefix-validation contract.
- `fish -n` was not run locally (fish not installed on this machine); the fish.sh hunks are byte-identical to upstream and upstream CI exercised them. `bash -n` passed.

## Provenance: `dc1077845` port detail

- `crates/warpui_core/src/core/app.rs`: ported verbatim — `impl AppContext` block with the four non-generic bookkeeping helpers (`take_model_for_update`, `finish_model_update`, `take_view_for_update`, `finish_view_update`) and the two entity-generic downcast helpers (`downcast_model_mut`, `downcast_view_mut`); `update_model`/`update_view` now call the helpers with the same panic messages, panic conditions, and flush ordering as upstream.
- `model/context.rs` and `view/context.rs`: ported verbatim — `SpawnResolveCallback`/`SpawnAbortCallback` type aliases, `spawn`/`spawn_abortable` boxing their arguments at the entry points, and the type-erased `spawn_abortable_boxed<O>` body containing the unchanged oneshot + `Abortable` + `spawn_local` chain.
- Fork adaptations:
  - The three view-side helpers use `Box<dyn AnyView>` where upstream uses `StoredView`: upstream's `StoredView` enum carries a `Tui` variant for the upstream TUI target, which this fork removed, so the fork's `Window.views` map stores `Box<dyn AnyView>` directly. No upstream behavior is otherwise changed; the fork has no `StoredView` import to restore (upstream's `use crate::core::{ActionType, StoredView, Window}` line is not fork state).
  - Import hunks resolved onto the fork's import groups: `BoxFuture` added to `r#async::{...}` groups in both context files; the fork's divergent import layout (no `warp_errors::report_error` import) is preserved.
  - Upstream's `#[cfg(not(target_family = "wasm"))]` context lines around imports do not exist in the fork (wasm gates removed at baseline); nothing to port there.

## Verification

- `cargo fmt -- --check`: clean after `cargo fmt` normalization of the 5-tuple match.
- `cargo check -p warp -p warp_completer -p warpui_core --all-targets --message-format short`: pass (pre-existing warnings only, in untouched files).
- `cargo check --workspace --all-targets --message-format short`: pass.
- Focused tests: `test(split_kill_buffer_write)` 4/4; `warp_completer` 168/168 (incl. the 5 new slice-clamp tests); `warpui_core` 322/322 + `warpui` 39/39 (upstream's spawn/update coverage suites); standard suite `test(slash_command) | test(acp) | test(terminal_suggestions)` 156/156.
- `bash -n` on `bash_body.sh`: pass. `fish -n` skipped (fish not installed locally).
- `cargo build -p warp --all-targets --message-format short`: succeeded; `cargo clean` after the release push.
- Deletion-surface scans over the nine touched files: removed-area scan hits are only `Weak*Handle::upgrade()` weak-handle methods and pre-existing comments; MCP/skills and platform scans return zero hits.
- Disk note: `CARGO_PROFILE_DEV_DEBUG=0` used for check/test/build as in prior audits.
