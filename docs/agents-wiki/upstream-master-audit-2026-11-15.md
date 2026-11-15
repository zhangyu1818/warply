# Upstream Master Audit — 2026-11-15

Incremental audit for `7a74a2df8..upstream/master` (`c6f07bb45`), 4 commits. The previous audit is `upstream-master-audit-2026-11-14.md`.

## Summary

Zero-port audit. All four commits live on fork-absent surfaces: the GitHub commit-attribution orchestration/request-metadata stack (`warp_multi_agent_api`, `app/src/ai/agent/api/`, orchestration/agent-sdk/server-api), the agent-sdk restricted git-credentials refresh with its GraphQL schema changes, the managed-MCP identity fix inside the same absent request-conversion layer, and an LRC runtime-flag promotion already superseded by the fork's default-on cargo feature. Every shared-file anchor was verified absent in the fork. No removed product surface was reintroduced; no code changed.

## Commit-by-Commit

### `c20d137ae` Carry human commit credit through remote agent dispatch (#16820) — rejected

Adds `GitHubCommitAttribution`/`delegated_github_coauthor_user_uid` to `RunAgentsRequest`, `SendMessage`, and remote-child dispatch so warp-server can stamp the triggering human UID onto commits made by delegated agents; also handles `ToolCallResultType::DeniedByHook` in conversation conversion/YAML. Depends on `warp_multi_agent_api` (commit bumps the proto rev; the fork has no such dependency — verified zero hits in `Cargo.toml`/`Cargo.lock`) and the `app/src/ai/agent/api/` request-conversion layer (fork-absent directory; the APP-5720 request-metadata stack was never ported). Orchestration (`remote_child.rs`, `run_agents.rs`, `start_agent.rs`, `send_message.rs`), `handoff/pipeline.rs`, `run_agents_card_view.rs`, `agent_sdk/`, `ambient_agents/`, `server_api/`, and `warp_tui` orchestration blocks are all fork-absent (verified: no `app/src/ai/orchestration`, no `handoff` dir, no `RunAgentsRequest`/`github_commit_attribution`/`DeniedByHook`/`delegated_github` anchors). Same identity/attribution series as the 2026-11-14-rejected `d69768ec6`/`7a74a2df8`.

### `10ea335c2` Refresh task credentials on independent provider schedules (#16831) — rejected

Reworks `agent_sdk/driver/git_credentials/restricted.rs` to refresh task-scoped Git credentials per provider schedule, adds `restricted_provider_tests.rs`, and extends `server_api/ai.rs`, `crates/graphql/`, and `warp_graphql_schema/api/schema.graphql`. Every touched path is fork-absent (no `agent_sdk` directory, no `app/src/server`, no `crates/graphql`, no `crates/warp_graphql_schema`). Continues the bot Git-credentials series rejected 2026-11-11/2026-11-12.

### `7e52e0d49` Preserve GitHub managed MCP identity for native tool calls (#16845) — rejected

Special-cases the reserved `github` managed-server `warp_id` in `mcp_server_identity()` inside `app/src/ai/agent/api/convert_to.rs` so warp-server can attribute GitHub MCP tool calls to the human for commit credit. The entire change lives in the fork-absent request-conversion layer (verified: no `app/src/ai/agent/api/`, zero `mcp_server_identity`/`McpServerIdentity` anchors). App-managed MCP identity and server-side attribution are removed surfaces; nothing separable.

### `c6f07bb45` Enable LRC activity signals for Preview and Dogfood (#16578) — not applicable

Moves `FeatureFlag::LrcActivitySignal` from `DOGFOOD_FLAGS` to `PREVIEW_FLAGS` in `crates/warp_features/src/lib.rs`. The fork already ships LRC activity signals enabled in every build through its cargo-feature pipeline: `lrc_activity_signal` is in the `default` list of `app/Cargo.toml` and bridged via `#[cfg(feature = "lrc_activity_signal")]` at `app/src/lib.rs` (the fork's `warp_features` flag lists do not gate LRC at runtime). The upstream promotion is a strict subset of the fork's existing default-on state.

## Verification

No code changed, so the tree is identical to `v2026.11.14`. All commands run with `SDKROOT` pointed at the full Xcode SDK and `CARGO_PROFILE_DEV_DEBUG=0`:

- `cargo fmt -- --check` — passed repo-wide.
- `cargo check -p warp --all-targets --message-format short` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo build -p warp --all-targets --message-format short` — passed, followed immediately by `cargo clean`.
- Restored-surface scans re-run with unchanged results against the 2026-11-14 baseline (only ONNX tokenizer vocabulary data and SSH bootstrap ConPTY comments).

## Tag

`v2026.11.15` on `main` (zero-port audit; direct doc commit per the 2026-11-07/2026-11-10/2026-11-12/2026-11-14 precedent).
