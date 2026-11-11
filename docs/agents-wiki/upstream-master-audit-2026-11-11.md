# Upstream Master Audit — 2026-11-11

Incremental audit for `8246c1e1b..upstream/master` (`619c82aa7`), 6 commits. The previous audit is `upstream-master-audit-2026-11-10.md`.

## Summary

Two commits ported (one as a prerequisite backfill), four rejected or not applicable. The ported pair carries upstream's size-based log-file rotation in the retained `crates/simple_logger` crate plus its test-race fix; the crate-side port supersedes the crate-side portion of the 2026-06-04 audit's blanket rejection of `92069590d` as an "app-managed MCP surface" — that commit is mixed, and under the path-level rule its `crates/simple_logger/**` hunks are a purely local, service-free, opt-in utility whose only upstream consumers (the MCP wiring hunks) stay rejected. The rejected four live entirely on fork-absent `agent_sdk`/`ambient_agents`/`server_api`/GraphQL orchestration surfaces, verified anchor-symbol by anchor-symbol.

## Commit-by-Commit

### `9d93abfd7` Carry typed child lifecycle context into parent-agent events (#14888) — rejected / not applicable

Replaces the free-form `event_type`/`detail` JSON on parent-LLM lifecycle events with a typed event-kind enum across `app/src/ai/agent_events/{driver,message_hydrator}*`, `app/src/ai/agent_sdk/ambient.rs`, `app/src/ai/agent_sdk/driver/harness/claude_code/parent_bridge.rs`, `app/src/ai/blocklist/orchestration_event_streamer*`/`orchestration_events*`, and `app/src/server/server_api/*` (new `agent_run_event_tests.rs`). Every touched directory is fork-absent: the fork has no `app/src/ai/agent_events/`, no `agent_sdk`, no orchestration streamer/events modules under `blocklist` (verified: no `orchestration*` entries in the fork's `app/src/ai/blocklist/`), and no `app/src/server/server_api/`. The `Cargo.toml`/`Cargo.lock` hunks only bump the `warp_multi_agent_api` git rev, a dependency the fork removed with the orchestration surface (absent from the fork manifest). The parent-agent lifecycle stream is the removed cloud orchestration event path; nothing is separable into the ACP client, which renders ACP-native session update events instead.

### `651109041` Use bot Git credentials with ordinary managed GitHub MCP transport (#16760) — rejected

Entirely inside removed surfaces: `app/src/ai/agent_sdk/driver{,/git_credentials,/prewarmed}` (restricted bot credential bootstrap/refresh/expiry), `app/src/ai/agent_sdk/driver/mcp_startup*` (managed GitHub MCP registration), `app/src/ai/agent_sdk/mod.rs`, `app/src/server/server_api/ai.rs`, `crates/graphql/**` (new `task_restricted_git_credentials` query), `crates/warp_graphql_schema/api/schema.graphql`, and `crates/warp_server_client/**`. All of `agent_sdk`, `server_api`, `crates/graphql`, `crates/warp_graphql_schema`, and `crates/warp_server_client` are fork-absent (verified by path listing), and `rg git_credentials|restricted_installation` over `app/` returns zero anchors. The feature's core is server-issued bot credentials for cloud agent runs — a removed Warp-server dependency chain.

### `4b790f5fa` Fix race in logger rotation test (#16791) — ported (with prerequisite backfill)

Test-only race fix in `crates/simple_logger/src/manager_tests.rs`: after `logger.close()`, the test now unwraps the writer `Arc` and `block_on`s the logging task before reading rotated paths or cleaning up files, replacing the `wait_for` poll that intermittently raced the background writer's rotation renames. The affected test is the rotation e2e test added by upstream `92069590d` (#10874), which the fork had never ported, so the fix is ported together with that commit's crate-side prerequisite (below). Applied as the exact upstream hunk; the fork's `SimpleLogger.writer: Arc<LogFileWriter>` and `LogFileWriter._logging_task: BackgroundTask` (a `Future`) already match upstream's pre-image, so the fix compiles verbatim.

### `92069590d` (backfilled prerequisite) fix(mcp): size-rotate MCP server log files to cap disk usage (#7723) (#10874) — crate side adapted, MCP wiring rejected

The 2026-06-04 audit rejected this commit wholesale as an app-managed MCP surface. That classification stands for `app/src/ai/mcp/logs.rs` (10 MiB × 5 rotation policy helper) and `app/src/ai/mcp/templatable_manager/native.rs` (the only `register_with_rotation` callers), but the commit's core — 547 lines in `crates/simple_logger` — is a generic local utility with zero service dependencies (the upstream description itself records "Server API dependencies: None"): `RotationConfig` with zero-maps-to-`None` construction, opt-in `LogManager::register_with_rotation` threading `Option<RotationConfig>` through the private `register_resolved_path`/`SimpleLogger::new` (existing callers unchanged, `None` default), writer-side byte counting with rotate-after-write, drop-handle-before-rename, `.N` shift with tolerated intermediate rename failures, append-mode recovery that preserves log content when the active-file rename fails, and `path_with_suffix` operating on the raw `OsString`. Ported source-first via `git diff 92069590d^ 92069590d -- crates/simple_logger/ | git apply --3way`, followed by the `4b790f5fa` hunk.

Intentionally omitted or kept fork-divergent:

- `app/src/ai/mcp/logs.rs`, `app/src/ai/mcp/templatable_manager/native.rs` — removed app-managed MCP surface; in this fork MCP servers belong to the ACP agent process, so no fork caller wires rotation yet and the API remains an available local capability for retained log consumers (e.g. LSP logs).
- `Cargo.lock` — regenerated from the fork manifest rather than copied; resolves to just `instant` + `tokio` added to the `simple_logger` package's dependency list (both already in the graph).
- Upstream's later never-ported crate edits are not swept in: import regrouping and the `safe_message` doc rewording (`4aea06734`-lineage), the `warpui` → `warpui_core` dependency rename (`a35bf3a47`-lineage; fork keeps `warpui`), edition let-chain rewrites (`abea51cd1`-lineage; the port keeps the nested-`if` form exactly as `92069590d` wrote it), and the `secure_state_dir`/`cfg!(windows)` `WARP_LOGS_DIR` branch in `log_directory_path` — which the fork's own `3dfa8418f` localization removed and which `92069590d` did not touch, so the three-way merge keeps the fork's macOS `state_dir()` form. None of those belong to either ported commit.
- `instant`/`tokio` dev-dependencies added as upstream specified; `wait_for` needs `instant` and the `perform_rotation` unit tests are `#[tokio::test]`.

### `cb91603d2` Restore native local agent subtrees after cloud-parent handoff (#16557) — rejected

Restores local Oz-harness child agents (and their descendants) when a cloud parent run resumes on a new virtual machine. The core new modules live on removed surfaces: `app/src/ai/ambient_agents/discovery*.rs`, `app/src/ai/agent_sdk/driver/restored_children*.rs`, `app/src/ai/blocklist/orchestration_event_streamer/restored_children.rs`, `app/src/pane_group/child_agent/restored_owners.rs` (the fork has no `child_agent` pane directory at all), and `app/src/server/server_api/ai.rs`. Every hunk in fork-present shared files is anchored on fork-absent symbols, verified individually: `app/src/ai/agent/conversation.rs` (anchors on `task_sync_mode`, `use_warp_credits_instead_of_chatgpt`, `is_remote_child`, `remote_run_updated_at` — all absent; the new `LocalChildRestorationStatus` machinery exists only to gate this restoration), `app/src/ai/blocklist/controller.rs` (`OrchestrationEventStreamerEvent`, `handle_dormant_claude_wake_ready` — absent), `app/src/ai/blocklist/history_model.rs` (`AmbientAgentTask`, `is_remote_child` — absent), `app/src/pane_group/mod.rs` (`OrchestrationEventStreamer` import, `AmbientAgentTaskId`, `ambient_pane_restoration`, `child_agent` module — all absent), and `app/src/ai/blocklist/local_agent_task_sync_model.rs`/`orchestration_child_tracker*` (files fork-absent). The `restore_local_subtree` cargo feature, `FeatureFlag::RestoreLocalSubtree` enum variant, and dogfood registration in `crates/warp_features/src/lib.rs` + `app/src/features.rs` are flag plumbing for this rejected feature and stay out. `specs/QUALITY-2186/TECH.md` is rejected as an upstream spec document. The retained pane-group/local-conversation surfaces keep their current behavior because the feature's runtime owner (cloud-parent handoff recovery) is the removed orchestration service.

### `be0d42bc6` Recognize MCP tool calls from every ACP adapter, not only Claude's (#16790) — not applicable

Adds `app/src/ai/agent_sdk/driver/harness/acp/mcp_projection.rs` with a harness-agnostic MCP tool-call shape table (`rawInput {server, tool, arguments}`, `mcp.<s>.<t>`, `mcp__<s>__<t>`, Pi proxies) and stamps `server_identity`/`server_id` so the Factory run view can brand Slack/Jira/Linear/Factory cards — because "ACP runs author their own history, so the driver has to produce the same shape" that warp-server stamps for Oz-native runs. Every touched path is under the removed `app/src/ai/agent_sdk/driver/harness/acp/` harness (`mapping.rs`, `protocol.rs`, `turn_driver_tests.rs`, `mcp_startup*`); the fork's retained `app/src/ai/acp/` client is a different codebase with no MCP-call classification layer (`rg "mcp__|mcp_projection|server_identity" app/src/ai/acp/` returns zero hits), no Factory run persistence, and no app-managed MCP server identity. Under fork ownership rules MCP servers are configured by the ACP agent process, so there is no host-side projection to fix.

### `619c82aa7` Support restricted GitHub credentials in third-party harnesses (#16785) — rejected

Seven-file follow-up to #16760 entirely under `app/src/ai/agent_sdk/driver{.rs,/git_credentials/restricted_tests.rs,/harness/acp/mod.rs,/harness/gemini*.rs,/mcp_startup*}`: removes the native-only launch rejection for restricted credentials, exports managed GitHub MCP through harness adapters, and writes resolved MCP servers into Gemini terminal settings. All paths are on the removed `agent_sdk` harness stack (verified absent); the restricted-credential infrastructure it builds on was rejected with `651109041` above, and managed/managed-MCP registration is app-managed MCP configuration the fork deliberately does not own.

## Verification

All commands run with `SDKROOT` pointed at the full Xcode SDK and `CARGO_PROFILE_DEV_DEBUG=0`.

- `cargo nextest run -p simple_logger` — 20 tests run, 20 passed (matches upstream's post-`4b790f5fa` count).
- `cargo fmt -- --check` — passed repo-wide (the port's import ordering normalized to the fork's rustfmt grouping).
- `cargo check -p warp --all-targets --message-format short` — passed; only pre-existing dead-code warnings in untouched blocklist/terminal files (`new_streaming`, queued-prompts-panel test helpers).
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — passed.
- `cargo build -p warp --all-targets --message-format short` — passed, followed immediately by `cargo clean`.
- Restored-surface scans re-run over the merged tree: the MCP/skills pattern scan returns zero hits, and the platform scan returns exactly the six known retained occurrences (SSH `ForwardX11=no` ×2, retained ConPTY comments in `zsh_body.sh` ×2, `#[cfg(windows)]`-gated `warp_util` tests ×2) — unchanged from the 2026-11-05/06/07/10 baselines. The cloud/auth/billing/telemetry pattern scan returns only the same pre-existing allowed hits as the prior audits.

## Tag

`v2026.11.11` on `main` via `merge/upstream-2026-11-11` (`--no-ff`).
