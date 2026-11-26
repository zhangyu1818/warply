# Upstream Master Audit — 2026-11-26

Incremental audit for `e1fe9eea3..upstream/master` (`e3cc805ec`, 7 commits). The previous audit is `upstream-master-audit-2026-11-25.md`.

## Summary

One partial port. Upstream's GraphQL error-alias commit (`421aa4573`) carries a lockfile bump of the shared transitive `hashbrown` (0.14.1 → 0.14.5) beneath the retained `dashmap` and `lasso` consumers; those three `Cargo.lock` hunks were applied verbatim, while the commit's GraphQL source and its `apollo-compiler` dev-dependency tree stay rejected with the fork-absent `crates/graphql` crate. The other six commits live on fork-absent surfaces: the Arch package builder, the `warp_multi_agent_api` proto pin, the dogfood block-completion telemetry event, the agent-sdk ACP harness's MCP environment forwarding and failed-tool-call recording, and the shared-session occurrence-time reporting. Every fork-present shared-file hunk was verified anchorless before rejection. No removed product surface was reintroduced; the only change is the three-hunk lockfile bump.

## Commit-by-Commit

### `e3525c726` Revert full system upgrade in Arch package builder (#16986) — not applicable

One-line Dockerfile change (`.github/actions/bundle_arch_package/Dockerfile`, `pacman -Syu` → `-Sy`) in the Arch Linux package builder. Linux packaging is a removed area and the file is fork-absent (same classification as the Arch packaging commits in the 2026-11-17 audit).

### `81a00ab35` Update multi-agent API to retain wait run IDs (#16987) — not applicable

Re-pins the `warp_multi_agent_api` git dependency (`warp-proto-apis` rev `9c9bad2f6` → `15e79fceb`) in the workspace `Cargo.toml` and `Cargo.lock`. The fork has no `warp_multi_agent_api` dependency (zero hits in both files), and the server-side wait-run retention (warp-server#22858) belongs to the removed orchestration surface.

### `37d682759` Remove dogfood-only block completion telemetry (#16993) — not applicable

Deletes `TelemetryEvent::BlockCompletedOnDogfoodOnly` from `app/src/server/telemetry/events.rs`, its `send_telemetry_from_ctx!` emission in `app/src/terminal/view.rs`, and the TUI emission. Two of the three paths are fork-absent; the fork-present `app/src/terminal/view.rs` has none of the hunk's anchors — `BlockCompletedOnDogfoodOnly`, `TelemetryEvent`, and `send_telemetry_from_ctx!` have zero hits repo-wide because external telemetry was removed at the fork baseline and the event never existed here. The fork's remaining `is_dogfood` references are the unrelated `warp_core` channel accessor and `safe_log!` macros.

### `ddd4fce41` Forward the computer use MCP server's upload environment to Codex over ACP (#16994) — rejected

All four paths are under the removed `app/src/ai/agent_sdk/` driver. The ACP harness gains `mcp_servers_for_acp` and `set_forwarded_env_vars` so a Codex run's stdio MCP servers receive `HarnessMcpSettings::forwarded_env_vars` by value, and `mcp_startup.rs` adds `WARP_WORKLOAD_TOKEN` and `OZ_RUN_ID` to `COMPUTER_USE_MCP_FORWARDED_ENV_VARS` so the Warp CLI's `artifact upload` can authenticate to warp-server (API key, IAP run id, workload token). This is app-managed MCP server configuration for cloud agent runs. Zero fork anchors (`forwarded_env_vars`, `HarnessMcpSettings`, `ResolvedMCPServers`, `mcp_server_for_acp`, `mcp_startup`, `warp-computer-use`): the fork's own ACP client launches a user-configured agent and never builds an MCP server list, because MCP belongs to the ACP agent process. The fork's `WARP_API_KEY`/`WARP_WORKLOAD_TOKEN` hits are the unrelated agent-hooks runner scrub list and secret-redaction pattern, and its `stop_recording` hits are the local terminal recorder and integration video recorder.

### `83143bb5d` Record failed MCP tool calls as errors in ACP conversations (#16995) — not applicable

Fixes the agent-sdk ACP harness's `ToolProjection` MCP branch and `mcp_projection::outcome` (an MCP result with `isError: true` becomes an error, as does a call ACP marked `failed` even when text can be extracted from its output), all under `app/src/ai/agent_sdk/driver/harness/acp/`. That projection layer was rejected with the earlier ACP-harness commits (2026-10-23, 2026-11-11) and has zero anchors (`mcp_projection`, `ToolProjection`). The fork's `app/src/ai/acp/mapping.rs` forwards `SessionUpdate::ToolCallUpdate` protocol-natively as `AcpEvent::ToolCallUpdated { update }`, and `app/src/ai/agent/conversation.rs` upserts the raw `ToolCallUpdate` (status included) into `AIAgentOutputMessageType::AcpToolCall`. There is no success/error reclassification step in which the upstream bug could exist, so nothing maps.

### `421aa4573` Fix GraphQL error aliases and validate native operations (#16997) — partially ported (lockfile only)

Source: aliases the scalar `error` field as `message` in five native operations, renames the matching `ServerApi` result accessors, and adds an `apollo-compiler`-backed `define_operation!` validation test. Every path is fork-absent (`crates/graphql/**`, `app/src/server/server_api/ai.rs`): Warp cloud GraphQL, rejected.

`Cargo.lock`: 19 of the 22 hunks add the `apollo-compiler` 1.33.1 dev-dependency tree to `warp_graphql` (apollo-parser, ariadne, concolor, concolor-query, countme, jsonpath-rust, rowan, serde_json_bytes, text-size, triomphe, `windows-sys 0.45.0`/`windows-targets 0.42.2` plus six arch crates, yansi) and disambiguate the `windows-targets` dependency names of the existing `windows-sys` entries. Not applicable: none of those packages exist in the fork, and its single `windows-targets` version keeps the unqualified names.

The remaining three hunks touch a retained path. Upstream's re-resolution moved the shared `hashbrown` 0.14 entry from 0.14.1 to 0.14.5 (checksum `e5274423…df41f1`) and its two consumers, `dashmap 6.1.0` and `lasso 0.7.3`, to it. The fork's lock had exactly that pre-bump state — same `dashmap`/`lasso` versions, same two consumers, same old checksum — and both are retained direct dependencies of `app`, `warp_core`, `remote_server`, `warpui_core`, `warpui`, and `local_object_model`. Because feature-oriented triage must not skip manifests, those three hunks were applied as a plain patch extracted from the exact upstream lock diff; the resulting `+`/`-` lines are content-identical to upstream's. hashbrown 0.14.5 carries the `clone_from_impl` panic-guard index fix (#511) on top of the 0.14.2/0.14.3 changes (Miri and nightly `ptr::invalid_mut` avoidance, the `HashTable` type, the iterator `fold` specialization).

Intentionally omitted: all nine source/manifest paths of the fork-absent GraphQL crate and server API, and the 19 apollo-tree/`windows-targets` lock hunks.

### `e3cc805ec` Preserve shared-session occurrence time in client task reports (#16998) — rejected

Captures the shared-session startup occurrence time before releasing startup waiters and threads it through the agent-sdk drivers (`driver.rs`, `wake_driver.rs`, `mcp_startup.rs`, `terminal.rs`), `local_agent_task_sync_model`, the orchestration `restored_children`, and the `UpdateAgentTask` GraphQL mutation and schema. Sixteen of the seventeen paths are fork-absent. The one fork-present file, `app/src/pane_group/mod_tests.rs`, gets only a closure-arity edit inside `restored_local_child_subtree_setup_failure` (a `client.expect_update_agent_task()` mock over `warp_graphql::ai::AgentTaskState`), and the fork's file has none of those anchors (zero hits for `expect_update_agent_task`, `restored_local_child_subtree_setup_failure`, `update_agent_task`, `warp_graphql`). The behavior is the removed shared-session/orchestration server-reporting surface.

## Restored-Surface Scans

The three skill scans were run with the literal `rg` commands against the changed tree: scan 1 returns 234 hits, scan 2 returns 7, scan 3 returns 16 — identical to the 2026-11-24/25 documented baselines. A `git grep` baseline-delta of `main` against the working tree (`app crates script Cargo.toml`), with binary-file match lines and output ordering excluded, shows zero text-hit differences in all three scans. The only changed file is `Cargo.lock`, which is outside the scan pathspec.

## Verification

The tree was post-`cargo clean`, so the suite compiled from scratch. All commands ran with `CARGO_PROFILE_DEV_DEBUG=0`; the two check commands additionally ran with `--locked` (as `.github/workflows/ci.yml` does) because this cycle edits `Cargo.lock`, so cargo had to accept the lockfile exactly as patched:

- `cargo metadata --locked` — resolved graph contains `hashbrown 0.14.5` in place of 0.14.1, no `apollo-compiler`/`rowan`/`ariadne`; `Cargo.lock` left untouched.
- `cargo fmt -- --check` — passed repo-wide.
- `cargo check -p warp --all-targets --locked --message-format short` — passed (1m04s; pre-existing dead-code and unused-variable warnings only).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 157 passed, 3075 skipped.
- `cargo check --workspace --all-targets --locked --message-format short` — passed (53s).
- `cargo build -p warp --all-targets --message-format short` — passed (fresh no-op on top of the test-binary build, which compiled `hashbrown 0.14.5`), followed immediately by `cargo clean` (removed 24691 files, 12.7GiB).

## Tag

`v2026.11.26` on `main` (one partial lockfile port; full branch shape — `merge/upstream-2026-11-26` merged `--no-ff`).
