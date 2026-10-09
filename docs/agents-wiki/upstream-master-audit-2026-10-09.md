# Upstream Master Audit — 2026-10-09

Incremental audit for `8c6a00c82..upstream/master` (`9ec105c94`), 6 commits. The previous audit is `upstream-master-audit-2026-10-08.md`.

## Summary

Two adapted ports landed: the LRC continuation-prompt snapshot fix (`325d4d470`, blocklist paths only) and the CLI-agent pending-background-work status fix (`42e2a7d75`, all four retained `cli_agent_sessions` paths). Four commits rejected or not applicable on fork-absent surfaces (agent_sdk driver/harness, build-cache, computer-use preference models, session-sharing reconnect). No removed product surface was restored.

## Commit-by-Commit

### `325d4d470` [QUALITY-826] Fix LRC agent hang on shell continuation prompts (#12934) — adapted (blocklist paths ported)

Root cause: with unbalanced quotes the shell shows a continuation prompt while the block stays in `BeforeExecution`, where all PTY output is routed to `prompt_and_command_grid()` instead of the output grid, so LRC snapshots polled empty content forever. Ported both `ShellCommandExecutor` snapshot sites (polling and control-handback) reading `prompt_and_command_grid()` in `BeforeExecution` plus both regression tests, applied with `git apply --3way` cleanly.

Omitted:

- `app/src/ai/agent_sdk/driver/terminal.rs` + `terminal_tests.rs` (stall-detector `block_output_plaintext` grid selection and the plaintext-until-preexec test): `agent_sdk` is fork-removed; the fork's LRC stall visibility flows through the blocklist snapshot path and `lrc_activity`, which the ported hunks cover.

Fork-divergence note: the test file keeps the fork's import ordering and lacks upstream's `RecordingController` singleton registration, `set_active_conversation_context` displacement block, and `injection_interrupt_keeps_completion_waiter_until_normal_precmd` test — all from unported upstream lineage (the injection-interrupt stack was rejected with the shared-session injection core on 2026-10-07), not parts of this commit.

### `662c7bf26` Repair build-cache permissions and Namespace metadata writes (#16360) — rejected (fork-absent surface)

Touches only `app/src/ai/agent_sdk/driver/cache_setup.rs` and `crates/build_cache/**`. Both are fork-absent (agent SDK removed at baseline; no `build_cache` crate — confirmed absent previously on 2026-10-06 and re-verified now). Nothing separable.

### `08a845f45` Agent driver: accept an ACP transport mode for third-party harnesses (#16329) — rejected (removed driver surface + dead flag)

Adds `HarnessTransport::Acp` to the upstream agent-sdk driver (`app/src/ai/agent_sdk/driver/**`), `local_harness_launch.rs`, `warp_cli` `oz agent` plumbing, and the `AcpHarness` feature flag. All consumers are fork-absent: the fork has no `agent_sdk` driver, no `app/src/pane_group/pane/local_harness_launch.rs`, and `warp_cli` has no `agent.rs`. The only shared file (`crates/warp_features/src/lib.rs`) would gain a permanently dead flag gating a nonexistent `--harness-transport acp` CLI surface. Although the upstream direction (driving harnesses over ACP) matches this fork's ACP-only ownership, the implementation lives entirely in the removed upstream driver; the fork's ACP client under `app/src/ai/acp/` already owns agent execution and gains nothing from this patch. Not ported.

### `42e2a7d75` Keep CLI harness sessions in progress while background work is pending (#16367) — adapted

A `stop` event carrying `background_task_count`/`session_cron_count` now updates session context but leaves status untouched (no `Success`) until a `Stop` with nothing pending arrives, so Claude Code runs babysitting background tasks no longer flap Success/InProgress per turn. Ported the payload field + `has_pending_background_work()`, the v1 parse summation, the `Stop` guard in `CLIAgentSession::apply_event`, and both regression tests (adapted to the fork's inline `CLIAgentSession` construction, since the fork's test file has no `cli_agent_session()` helper; `input_state: Closed` matches the upstream helper).

Omitted:

- `crates/warp_core/src/cli_agent_protocol.rs`: fork-absent (the fork's pre-plugin_manager lineage keeps a local `RawEvent` in `event/v1.rs`); the two serde fields were added to that local struct instead — the parse logic itself is verbatim upstream.
- `app/src/terminal/cli_agent_sessions/plugin_manager/claude.rs` `MINIMUM_PLUGIN_VERSION` 2.4.0 bump: the fork has no `plugin_manager/` (no Warp-managed plugin install); the fork's `CLIAgentEventPayload` also keeps no `plugin_version`/`error_type` fields or `StopFailure` arm, consistent with prior lineage decisions.
- Upstream consumers `LocalAgentTaskSyncModel` reports, notifiers, usage charging, and the idle-exit timer are fork-absent; the retained effect in this fork is the local session/conversation status staying `InProgress` (no premature `ConversationStatus::Success`).

### `fae855ad5` Add per-session computer-use overrides to agent preference models (#16359) — rejected (removed surface)

`session_computer_use` map and accessors on `AIExecutionProfilesModel` are `dead_code` outside tests and serve only the computer-use stack this fork removed; `app/src/auth/mod.rs` is fork-absent; `app/src/ai/llms.rs` is the one-line `LLMId` re-export stub with no `LLMPreferences` anchors. Re-confirmed all anchors absent.

### `9ec105c94` Prevent poison messages from causing session-sharing reconnect loops (#16071) — rejected (removed surface)

Session-sharing sharer network reconnect caps and oversized-message handling in `app/src/terminal/shared_session/**` and `app/src/ai/agent_sdk/**` — both fork-absent, as is `app/src/terminal/local_tty/terminal_view_adaptor.rs`. The one shared-file hunk (`app/src/terminal/view.rs`) only adds a `SharedSessionFailed` event variant whose sole producers/consumers are the removed sharing stack; porting it would add dead code for a removed surface.

## Verification

- `cargo check -p warp --all-targets --message-format short` — pass (pre-existing warnings only).
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass.
- `cargo nextest run -p warp -E 'test(cli_agent_sessions) | test(shell_command)'` — 55 passed incl. all four new regression tests; `test(shell_command)` alone — 5 passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- `cargo build -p warp --all-targets --message-format short` — pass, followed by `cargo clean`.
- Removed-surface scans over `app crates script Cargo.toml`: only pre-existing allowed hits (fork documentation comments, tokenizer vocabulary, retained SSH `ForwardX11`, bootstrap ConPTY comments, warp_util windows test gates); zero MCP/skills hits; no hit inside any file touched by this merge.
