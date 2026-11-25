# Upstream Master Audit — 2026-11-25

Incremental audit for `4540015ee..upstream/master` (`e1fe9eea3`, 8 commits). The previous audit is `upstream-master-audit-2026-11-24.md`.

## Summary

Zero-port audit. All eight commits live on fork-absent surfaces: the cloud-agent agent-inbox delivery through the agent-sdk ACP followup harness (landed, reverted, and reapplied), the agent-sdk environment-clone retry and handoff-snapshot exclusions, the IAP-header MCP transport to warp-server, the agent-sdk ACP-harness turn-failure semantics, and the auth sign-in banner for rejected anonymous sessions. Every fork-present shared-file hunk was verified anchorless before rejection. No removed product surface was reintroduced; no code changed.

## Commit-by-Commit

### `629b4a8fd` Deliver agent inbox batches through shared ACP followup turns (#16939) — rejected

Resolves typed cloud-agent `AgentWake` queries into undelivered inbox batches fetched over new GraphQL arguments (`fetch_agent_inbox`/ACK against warp-server#22527) and delivers them through the agent-sdk ACP harness turn driver, gated by a `SessionAgentMessages` client capability negotiated with the server and a `session_agent_messages` cargo feature. Of its 18 paths, 14 are fork-absent outright: `app/src/ai/agent_sdk/driver/harness/acp/**` (agent_sdk removed), `app/src/ai/blocklist/controller/external_harness.rs` + `external_harness_tests.rs` + `orchestration_event_streamer.rs` (never ported), `app/src/server/server_api*` (server API removed), `crates/graphql/**`, `crates/warp_graphql_schema/**`. The four fork-present files carry anchorless hunks:

- `app/Cargo.toml` — adds `session_agent_messages = []` beside `cloud_agent_execution_config`/`cloud_agent_runners`; the fork's feature list has no such cloud block, and the flag's only consumer (`FeatureFlag::SessionAgentMessages` in `crates/warp_features`) does not exist in the fork (verified zero hits).
- `app/src/features.rs` — upstream's `enabled_features()` bridge does not exist; the fork's file is the one-line `pub use warp_core::features::*;` re-export with flags wired through `app/src/lib.rs` + Cargo defaults.
- `crates/warp_features/src/lib.rs` — no `SessionAgentMessages`/`CloudAgentExecutionConfig` variants to extend.

The behavior itself (server-owned inbox batches for cloud agent runs) is the removed ambient/agent-sdk product area.

### `d0704206a` fix(agent-sdk): retry failed environment clones once (#16563) — rejected

Entirely inside `app/src/ai/agent_sdk/driver/environment*.rs` and `skill_references.rs` — the removed agent-sdk environment-checkout stack. `environment_checkout`/`skill_references` have zero hits in the fork. Not applicable.

### `928aef6d4` Send the Warp client's IAP headers on MCP connections to warp-server (#16965) — rejected

Adds a `HeaderSource` to `crates/mcp` `RefreshableHeaders` and selects warp-server-origin MCP servers by `http_client::is_warp_server_origin` so `ServerApi::iap_headers()` rides every Streamable HTTP request. `crates/mcp`, `crates/mcp/src/refreshable_headers*.rs`, and `app/src/ai/mcp/` are fork-absent (app-managed MCP removed; MCP belongs to the ACP agent process). The lone retained-crate hunk promotes `is_warp_server_origin` to `pub`, but the fork's `crates/http_client/src/lib.rs` has neither that function nor `ChannelState` server-URL machinery (verified zero hits), and `ServerApi::iap_headers()` is fork-absent. Not applicable.

### `e46cf5371` Revert "Deliver agent inbox batches…" (#16970) — not applicable

Revert of `629b4a8fd`, which was never ported. Nothing to revert.

### `eac4fe13d` [ACP] Keep a run's declared outcome when a later turn fails (#16971) — not applicable

Entirely inside the agent-sdk ACP harness (`turn_driver.rs`, `turn_failure.rs` adapter-convention table, `connection.rs` `errorKind`/session-not-found accessors, plus telemetry turn labels `error_turn_failed`/`error_session_lost`). The fix's core rule — the driver no longer decides whether a failure replaces the run's state because the server owns it (warp-server#22776) — is server-coupled. The fork's own ACP client under `app/src/ai/acp/` is a host/UI path with no Turn action, turn driver, harness telemetry, or adapter table: its `model.rs` publishes `AcpEvent::Failed`/`Cancelled`/`Completed` directly from the `session/prompt` result, and there is no server-owned run state for an adapter error to overwrite. Nothing maps; the insight (an errored `session/prompt` is not necessarily fatal) has no fork consumer to port onto and would require new handwritten code, which the source-first rule forbids.

### `e87535997` fix(agent-sdk): exclude temp-dir files from handoff snapshot declarations (#16981) — rejected

Entirely `app/src/ai/agent_sdk/driver/snapshot*.rs` — fork-absent (verified `handoff_snapshot` zero hits). Not applicable.

### `0125d2944` Show the sign-in banner when the server rejects an anonymous session (#16975) — rejected

Maps a new `GraphQLError::AnonymousSessionRejected` (401 + `X-Warp-Anonymous-Session-Rejected` header) to `AuthEvent::NeedsReauth`, reroutes anonymous sign-up/copy-link through `sign_in_url()`, and marks the variant permanent in the retry classifiers. Every path is a removed surface: `app/src/auth/` (absent), `crates/graphql/` and `crates/warp_server_client/` (absent), `app/src/server/` (directory absent — the only `RetryStrategy` symbol in the fork is warpui_core's unrelated view-update retry). Not applicable.

### `e1fe9eea3` Reapply "Deliver agent inbox batches…" (#16984) — rejected

Reapply of `629b4a8fd` with identical paths and hunks; same classification.

## Restored-Surface Scans

All three scans re-run against the unchanged tree: scan 1 returns 234 hits, scan 2 returns 7, scan 3 returns 16 — identical to the 2026-11-24 documented baselines; the sets remain the documented inert/local ones (tokenizer JSON tokens, doc comments, retained remote-terminal and local-object code).

## Verification

No code changed, so the tree is identical to `v2026.11.24`. All commands run with `SDKROOT` pointed at the full Xcode SDK (`MacOSX27.0.sdk`) and `CARGO_PROFILE_DEV_DEBUG=0`:

- `cargo fmt -- --check` — passed repo-wide.
- `cargo check -p warp --all-targets --message-format short` — passed (1m03s; pre-existing unused match-binding warnings only).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 157 passed, 3075 skipped.
- `cargo check --workspace --all-targets --message-format short` — passed (53s).
- `cargo build -p warp --all-targets --message-format short` — passed, followed immediately by `cargo clean` (removed 12.7GiB).

## Tag

`v2026.11.25` on `main` (zero-port audit; direct doc commit per the 2026-11-24 and earlier zero-port precedent).
