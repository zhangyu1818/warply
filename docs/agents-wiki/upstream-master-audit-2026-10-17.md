# Upstream Master Audit — 2026-10-17

Incremental audit for `4c509880f..upstream/master` (`b80b4e3e0`), 5 commits. The previous audit is `upstream-master-audit-2026-10-16.md`.

## Summary

Zero ports: all five commits are not applicable or rejected. Four live entirely in fork-absent surfaces (`app/src/ai/agent_sdk/**` including its `driver/harness/acp` external-harness stack, `app/src/server/server_api/**`, `crates/graphql/**`/`crates/warp_graphql_schema/**`, `resources/bundled/skills/**`, `app/src/ai/ambient_agents/**`, orchestration pills/links/event streamer, `app/src/pane_group/child_agent/**`, `app/src/terminal/shared_session/**`, `app/src/ai/blocklist/controller/external_harness*.rs`), verified per file against the fork tree. The fifth (`b80b4e3e0`, lazy hydration of remote child agent panes) also touches retained files, but every hunk in them exists only to serve the fork-absent remote-child pane surface and has no fork anchor.

No product surface was restored; no code was changed in this batch. Verification re-ran on the unchanged tree: `cargo fmt -- --check` passed, `cargo check -p warp --all-targets --message-format short` passed, and the restored-surface scans match the 2026-10-16 baseline (remaining hits are documentation, tests, retained local-object schema names, retained SSH remote-terminal behavior, and the zsh bootstrap ConPTY comments).

## Commit-by-Commit

### `3e2b62975` Dogfood frozen deferred benchmark repository preparation (#16451) — not applicable

Follow-up to the server-coupled REMOTE-3490/REMOTE-2960 deferred-repository stack already rejected in the 2026-10-16 audit (`522481cd0`): frozen checkout policy, execution bootstrap metadata, `FactoryDeferredRepositories` gating, Cynic query-field omission, and the bundled `factory-deferred-repositories` skill. Every touched path is fork-absent: `app/src/ai/agent_sdk/**` (removed Agent SDK), `app/src/server/server_api/**` (no `app/src/server`), `crates/graphql/**` + `crates/warp_graphql_schema/**` (no graphql crates), `resources/bundled/skills/**` (no `resources/bundled`). Nothing separable.

### `2590c9b56` refactor(agent-sdk): make MCP startup helpers driver-independent (#16480) — not applicable

Moves MCP spec resolution, built-in server attachment, concurrent startup, and readiness waiting from `impl AgentDriver` into `driver::mcp_startup` free functions, as part 4 of the pre-warmed sandbox stack (REMOTE-3488). Only touches `app/src/ai/agent_sdk/driver.rs` and `app/src/ai/agent_sdk/driver/mcp_startup.rs`, both fork-absent. App-managed MCP startup is a removed surface (MCP belongs to the ACP agent process in this fork).

### `a38e4416d` Resolve ACP harness identity from warp-harness.json manifests (#16447) — not applicable

Strict version-1 JSON manifest resolution for ACP sidecar harness launch/setup identity (schema, `WARP_HARNESS_SIDECAR` selection, `${MANIFEST_DIR}` expansion, sidecar filename validation, per-harness `claude_code`/`codex`/`gemini`/`pi` updates). Entirely under `app/src/ai/agent_sdk/driver/harness/**` — the external-harness ACP stack rejected in the 2026-10-10 audit. The fork's ACP client lives in `app/src/ai/acp/` with its own registry (`app/src/ai/acp/registry.rs`) and shares no code with this stack; `find app/src -name "harness*"` returns nothing.

### `f90f90a97` ACP: a follow-up that steers a running turn no longer surfaces a Cancelled status (#16513) — not applicable

Fixes the external-harness ACP turn driver settling a steered turn via `finish_turn(Done)` + `mark_conversation_cancelled`, which mirrored `SUCCEEDED` → `CANCELLED` → `IN_PROGRESS` to warp-server and fired the Slack cancellation notice; replaced by a `TurnHost::cancel_turn(stream_id, TurnCancellation)` hook (`FollowUpSubmitted` → `KeepInProgress` vs `ManuallyCancelled`) mapped to `BlocklistAIController::cancel_external_harness_turn`. Touched files: `app/src/ai/agent_sdk/driver/harness/acp/{mod,turn_driver,turn_driver_tests}.rs` and `app/src/ai/blocklist/controller/external_harness{,_tests}.rs` — all fork-absent (fork `controller/` holds only `input_context.rs`, `response_stream.rs`, `slash_command.rs`).

Fork-relevance check: the fork's ACP run loop (`app/src/ai/acp/model.rs`) cannot exhibit the bug. Follow-up prompts are queued on `prompt_rx` and processed sequentially after the in-flight prompt completes; nothing cancels a running turn on follow-up submission. `cancel_session` is reachable only from user stop paths (status bar stop, `stop` command, cancel-progress), and the loop's `StopReason::Cancelled` → `AcpEvent::Cancelled` → `mark_response_stream_cancelled(ManuallyCancelled)` already matches the upstream Interrupt semantics. There is no warp-server status mirroring or Slack notice surface to fix.

### `b80b4e3e0` Lazily hydrate remote child agent panes (#16514) — not applicable

Defers remote/shared-session child pane hydration until explicit navigation (reveal/swap/open materialize one child; shared-session discovery stops emitting automatic pane requests; `EnsureUnifiedViewerChildPane` removed). Core mechanism files are fork-absent: `app/src/ai/ambient_agents/task.rs`, `app/src/ai/blocklist/agent_view/orchestration_{conversation_links,pill_bar}.rs`, `app/src/ai/blocklist/orchestration_event_streamer.rs`, `app/src/pane_group/child_agent/{hydration,restoration}.rs`, `app/src/terminal/shared_session/viewer/**`.

The retained-file hunks have no fork anchors and only serve that surface:

- `app/src/pane_group/mod.rs`, `pane/terminal_pane.rs`, `terminal/view.rs`: rename `restore_missing_child_agent_panes_*` → `restore_child_agents_*`, drop `EnsureUnifiedViewerChildPane` handling, and re-comment `pending_child_hydrations`. `rg` for `restore_missing_child_agent_panes|pending_ambient_agent_conversation_restorations|pending_child_hydrations|EnsureUnifiedViewerChildPane|AmbientAgentTask` across those three fork files returns zero matches — the child-agent pane surface was never ported.
- `app/src/ai/agent/conversation.rs`: adds `remote_run_updated_at` (+ accessors, constructor seeds, and a `last_modified_at` override) so remote-child placeholders sort by the run's server update time. The fork's `AIConversation` has no `task_sync_mode`, `use_warp_credits_instead_of_chatgpt`, or `ConversationDriver::RemoteChild` context, and `last_modified_at` keeps its exchange-based form; the field has no consumer without remote children.
- `app/src/ai/blocklist/history_model.rs`: adds `apply_remote_child_task_snapshot` (consumes fork-absent `AmbientAgentTask`) and a placeholder-merge passthrough inside the cloud-conversation merge path (fork has no `reassign_exchange_ids`/`server_metadata` merge path).
- `app/src/pane_group/mod_tests.rs`: expectations for lazy remote/viewer restoration — fork file has no child-pane tests to update.

## Verification

No code changed; the tree equals the 2026-10-16 verified state plus this document.

- `cargo fmt -- --check` — passed.
- `cargo check -p warp --all-targets --message-format short` — passed (cold build, `SDKROOT` pointed at the Xcode SDK per the CLT 27 tapi prerequisite).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — passed (156 run, 156 passed, 2970 skipped).
- Restored-surface scans — hit counts and files identical to the 2026-10-16 baseline; allowed categories unchanged (docs/tests/local-object schema names/SSH remote-terminal/zsh bootstrap comments).
- `cargo clean` — run after the check completed.

## Tag

`v2026.10.17` on `main` (merge commit of `merge/upstream-2026-10-17`).
