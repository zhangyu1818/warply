# Upstream Master Audit — 2026-11-24

Incremental audit for `f8223cef2..upstream/master` (`4540015ee`, 3 commits). The previous audit is `upstream-master-audit-2026-11-22.md`.

## Summary

Zero-port audit. All three commits live on fork-absent surfaces: the removed upstream autoupdate bundle-swap flow, the removed agent-sdk/ambient-agent/shared-session failure-retention stack with its REMOTE-2661 setup-failure-debug routing, and the removed staging-banner/Warp-server toast. Every fork-present shared-file hunk was verified anchorless before rejection. No removed product surface was reintroduced; no code changed.

## Commit-by-Commit

### `75f8808a1` Keep the running app bundle intact when applying a macOS autoupdate (#16511) — rejected

Rewrites `app/src/autoupdate/mac.rs` so applying an autoupdate moves the whole old `.app` bundle to `state_dir()/old.app` (instead of only the running executable to `state_dir()/old`), keeping the running process inside its signed bundle so macOS TCC attribution survives; `remove_old_executable` also cleans the legacy `old` file, and `StagedBundle::drop` skips removal when the staged path was already moved away. The fork removed `app/src/autoupdate/` wholesale (contract: upstream Warp update infrastructure replaced by the Sparkle 2 updater under `app/src/updater/`), so the touched file does not exist and the `renamex_np(RENAME_SWAP)` + executable-move flow has no fork counterpart. Not applicable rather than a cherry-pick candidate; the TCC-bundle-integrity insight belongs to a flow this fork does not run.

### `4cf398a13` Limit failure session retention to setup errors (#16832) — rejected

Reworks failure-session retention across the removed cloud-agent stack: runtime errors exit immediately for native/terminal-CLI/ACP harnesses while setup failures keep the shared session open for debugging, cancelled task-backed runs reject prompts before queue/attachment/harness side effects, authorized setup-failure follow-ups start separate debug conversations, the disconnected sender-side HTTP debug fallback is removed, and server-delivered failed-run follow-ups pair with warp-server#21834. Of its 19 paths, 14 are fork-absent outright: `app/src/ai/agent_sdk/driver*.rs` (agent_sdk removed), `controller/shared_session.rs`, `controller/startup_queue*.rs`, `orchestration_events.rs`, `terminal/local_tty/terminal_view_adaptor*.rs`, `terminal/view/ambient_agent/model.rs`, `cloud_conversation_continuation*.rs`, `terminal/input/slash_commands/data_source/gui.rs`, and `crates/warp_cli/src/agent.rs` (fork's minimal `warp_cli` has no agent surface). The five fork-present files carry only anchorless hunks:

- `app/src/ai/blocklist/controller.rs` — extends the queued-prompt eligibility gate `can_dispatch_queued_warp_agent_prompt` (adds `ConversationStatus::Blocked` and a `task_sync_mode().suppresses_task_lifecycle_updates()` arm). The fork's controller has no such function and none of its anchors (`QueuedQueryModel::is_dispatch_blocked`, `has_active_subagent`, any `is_awaiting_follow_up` call site — the fork defines `hooks::turn::is_awaiting_follow_up` but has zero consumers). The fork's `ConversationStatus::Blocked { blocked_action }` variant exists in the ACP conversation model, not in any queued-prompt dispatch gate.
- `app/src/terminal/input.rs` — deletes the REMOTE-2661 `SubmitSetupFailureDebugFollowup` event and `AIQueryRouting::RetainedSetupFailureDebug` arm; the fork has neither (`resolve_ai_query_routing`/`AIQueryRouting` do not exist).
- `app/src/terminal/view.rs` — deletes `try_submit_setup_failure_debug_followup` and its event handler; both depend on `ambient_agent_view_model`, fork-absent.
- `app/src/terminal/input/slash_commands/mod.rs` — replaces the REMOTE-2661 retained-setup-failure routing with a `LiveRemoteVm` attachment gate around `resolve_ai_query_routing`; fork-absent symbols.
- `crates/warp_cli/src/agent.rs` — doc-comment rewrite for `--keep-failed-session-open`-style retention flags; file absent.

Nothing separable: every retained-path hunk modifies code the fork never had, and the behavior itself (session retention for cloud agent runs) is the removed ambient-agent/agent-sdk product area.

### `4540015ee` Point staging API error toast to IAP account settings (#16962) — rejected

One-line copy change in the staging-banner toast in `app/src/workspace/view.rs` (`shown_staging_banner_count` block). The fork removed the staging-banner/Warp-server-access surface: no `Staging API` string or staging banner exists anywhere in the fork, and IAP/Account settings are a removed product area. Not applicable.

## Restored-Surface Scans

All three scans re-run against the unchanged tree: scan 1 returns the same 234 hits, scan 2 the same 7, scan 3 the same 16 as the 2026-11-20/21/22 documented baselines — no new hits; the sets remain the documented inert/local ones.

## Verification

No code changed, so the tree is identical to `v2026.11.22`. All commands run with `SDKROOT` pointed at the full Xcode SDK and `CARGO_PROFILE_DEV_DEBUG=0`:

- `cargo fmt -- --check` — passed repo-wide.
- `cargo check -p warp --all-targets --message-format short` — passed (1m05s; pre-existing unused match-binding warnings only).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 157 passed, 3075 skipped.
- `cargo check --workspace --all-targets --message-format short` — passed (53s).
- `cargo build -p warp --all-targets --message-format short` — passed, followed immediately by `cargo clean`.

## Tag

`v2026.11.24` on `main` (zero-port audit; direct doc commit per the 2026-11-07/2026-11-10/2026-11-12/2026-11-14/2026-11-15/2026-11-21 precedent).
