# Upstream Master Audit — 2026-11-20

Incremental audit for `a1ade81fc..upstream/master` (`cf9434a18`, 4 commits), two of them the agent-hooks stack's ended-turn stops, ported as two adapted port commits on `merge/upstream-2026-11-20`. The previous audit is `upstream-master-audit-2026-11-19.md`.

## Summary

Two commits ported (turn.stop for cancelled and failed turns with the controller status-change subscription, and the hold_ended_turn recording hardening from the live-check fixes); two rejected — one cloud team-model validation and one WindowsACP PowerShell change, both confined to fork-absent `agent_sdk`/`server_api` paths. No removed product surface was restored.

Port commits: `fffb01a23` (`0356d03ba`), `d26bb86b5` (`2c01ad402`).

## Commit-by-Commit

### `0356d03ba` hooks: fire turn.stop for cancelled and failed turns, and hold the run exit for it (#16903) — adapted port

A turn that ends as cancelled or failed now fires the `turn.stop` boundary with `status: Cancelled/Error`: `hold_ended_turn` records the stop from a conversation status change (a change from a not-finished status to `Cancelled`/`Error`), `start_ended_turn_stops` starts it once the stream that ended the turn is cleaned up, and `finish_ended_turn_stop` reports a follow-up, verdict, or `halt_turn` the hooks asked for as warning diagnostics, none of which a stop after a turn that did not complete can act on. One stop per turn is enforced through `TurnStops::stopped_at` (the conversation's exchange count when its last stop started), `pending_stop_for` is the shared piece between the completed hold and the ended hold, and `forget` cancels running ended-turn stops. The controller subscribes to the history model's `UpdatedConversationStatus` — the one place every path that ends a turn as cancelled or failed reaches — and the renames `start_held_turn_stop` → `start_held_turn_stop_hooks` and `settle_turn_after_stop` → `settle_turn_after_stop_hooks` follow upstream.

Fork adaptations: `app/src/ai/agent_sdk/driver.rs` and `driver_tests.rs` are omitted with the removed driver — the fork has no run exit to hold, so `ended_turn_stops_settled`, `ENDED_TURN_STOP_EXIT_GRACE`, the `EndedTurnStop` `timeout`/`finished`/`done` oneshot bookkeeping, the `EndedTurnStopOutcome._finished` field, and their wasm cfg_attrs are omitted too (their only consumer was the driver's exit wait). `ended_turn_status` drops the `TransientError`/`WaitingForEvents` arms absent from the fork's `ConversationStatus`. The controller subscription uses the fork's `terminal_view_id` event/field naming and the fork's three-argument model subscription closure; the `AcpEvent::Completed` handler is the fork's stream end that also starts waiting stops (upstream's `AfterStreamFinished` handler). In the fork's ACP flow the exchange is marked non-streaming before the status change inside the same history update, so the ended stops normally start straight from the subscription. Tests: the four upstream tests are adapted to the fork's library-level `hook_test_app` fixtures (direct `hold_ended_turn`/`start_ended_turn_stops`/`finish_ended_turn_stop` calls; the second turn is created with the fork's `append_reassigned_exchange`/`streaming_exchange` fixture); the controller-driven e2e variants are omitted with the absent MAA/ControllerHarness fixtures, and the pure status-rule test drops the absent-variant asserts. Omitted: `specs/agent-hooks/**`.

### `2c01ad402` hooks: keep the Oz final answer whole and wait for a failed turn's stop hooks (live-check fixes) (#16924) — adapted port (turn.rs hunk only)

`hold_ended_turn` now checks `HookRuntime::is_installed` and reads the recorded `stopped_at` exchange count before building the envelope, so a stop already recorded (the same status change reported again, or a second subscriber seeing one) costs no envelope build and fires nothing; the already-recorded log moves to debug with the "was already recorded" wording.

Fork omissions: the stream observer's `Tool::Server` transparency and its test are omitted — the fork's `hooks/stream.rs` is the ACP-adapted observer over `AcpStreamDelta`, with no `warp_multi_agent_api` server tool calls. The controller's `stop_ended_turn` widening to `pub(crate)` (with its driver-calling doc comment) is omitted — only the removed `agent_sdk` driver calls it. `driver.rs`/`driver_tests.rs` stay out with the removed driver. Omitted: `specs/agent-hooks/**`.

### `0827149c7` Validate a cloud execution's model against the run's team (#16934) — rejected

Cloud (Oz) execution model validation against the owning team's catalog: `agent_sdk/common.rs`, `common_tests.rs`, `mod.rs`, and `server/server_api/testdata/prewarmed_assignment_fresh_run.json` — all fork-absent (`app/src/ai/agent_sdk/` and `app/src/server/` do not exist); `validate_execution_model_id`/`HeadlessTeamScope`/`build_execution_task_and_options` have no fork anchors.

### `cf9434a18` [WindowsACP] Make PowerShell wait for the ACP bridge and checkout helper on Windows (#16753) — rejected

PowerShell startup ordering for the agent_sdk ACP harness's bridge and checkout helper on Windows: `agent_sdk/driver/environment.rs`, `environment_tests.rs`, `driver/harness/acp/bridge.rs`, `bridge_tests.rs` — the removed Oz harness stack, not the fork's ACP client (`app/src/ai/acp/`), and Windows-only platform behavior.

## Restored-Surface Scans

All three skill scans re-run and compared against the pre-change `main` worktree: scan 1 returns the same 234 hits, scan 2 the same 7, scan 3 the same 16 — no new hits, the sets remain the documented inert/local ones.

## Verification

All commands run with `SDKROOT` pointed at the full Xcode SDK and `CARGO_PROFILE_DEV_DEBUG=0`:

- `cargo fmt -- --check` — passed repo-wide.
- `cargo check -p warp --all-targets` — passed after each port.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo nextest run -p warp -E 'test(hooks::turn)'` — 11 passed (including the four new adapted tests).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 157 passed.
- `cargo nextest run -p warp -E 'test(hooks) | test(prompt) | test(session) | test(turn) | test(envelope) | test(pre_tool) | test(action_model) | test(queued_query) | test(stream) | test(conversation)'` — 576 passed.
- `cargo nextest run -p ai` — 313 passed.
- `cargo build -p warp --all-targets --message-format short` — passed, `cargo clean` to follow after the push.

Known gaps, recorded honestly: the ended-turn stop path is verified at the library level (hold/start/finish, one-stop-per-turn, refused follow-up diagnostic, status rule); the controller-driven cancel/fail e2e coverage stays omitted with the absent ControllerHarness/MAA fixtures, and no run-exit wait exists in this fork to cover.

## Tag

`v2026.11.20` on `main` (two adapted port commits plus this audit, merged with `--no-ff`).
