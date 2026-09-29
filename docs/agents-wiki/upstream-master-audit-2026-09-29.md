# Upstream Master Audit 2026-09-29

Range under review: `cb2416204..upstream/master` (6 commits)

Previous audited upstream tip: `cb2416204 Fetch substituted benchmark branches without live default refs (#16172)`

Current upstream tip: `7aa4de12b Scope Bedrock token requests to the run team (#16133)`

Total upstream commits in this incremental range: 6

Status: triage complete. One adapted port from `8dfda6bbd` (remote-server abort-on-drop fix plus its client and daemon tests); five commits rejected or not applicable with no fork anchors.

## Per-Commit Triage

### `e7656eab3` — ci: skip full checks for documentation-only PRs (#16142)

Decision: **not applicable**. The only touched path is `.github/workflows/ci.yml`, which is fork-owned release CI diverged from upstream (no `Compute workflow parameters`/`Check CI results` aggregator, `--channel oss` bundle check instead). Continues the standing N/A family for upstream CI workflow commits.

### `8dfda6bbd` — Add Git identity diagnostics to clone failures (#16153)

Decision: **adapt** (remote-server portion) + **reject** (agent-SDK portion).

Rejected portion: `app/src/ai/agent_sdk/driver/environment.rs` and `environment_tests.rs` add Git author/credential-username diagnostics to cloud-agent environment clone failures. `app/src/ai/agent_sdk/` is a Removed System and does not exist in this fork; the whole diagnostic flow (silent-command abstraction, forge-host credential lookups, structured identity data on failures) lives inside it.

Ported portion — the retained remote-server fix this commit carries:

- `crates/remote_server/src/client/mod.rs`: new `PendingRequestGuard` RAII guard on `RemoteServerClient`. Any dropped `send_request` future (caller task cancelled, not only the timeout arm) now removes the pending request and sends `Abort` to the daemon, instead of leaving credential helpers running on the host. Applied via exact three-way patch; the timeout arm's manual remove+abort is removed upstream-side and the guard covers it. The fork's `send_request` is upstream's `send_request_internal` (the fork dropped the telemetry wrapper that fires `RequestFailedEvent`), so the hunks landed on the fork's single correlation function. Verified the fork's `reader_task` removes the pending entry before resolving the oneshot, so success-path guard drops send no spurious aborts.
- `crates/remote_server/src/client_tests.rs`: new `timed_out_run_command_removes_pending_request_and_sends_abort` plus the `Arc` import cleanup, applied from the exact upstream test and adapted to the fork: 2-tuple `RemoteServerClient::new` (fork has no `_failure_rx`/`_host_rx` channels) and flat `client_message::Message::RunCommand`/`::Abort` matching instead of upstream's `unwrap_session_scoped`/`unwrap_notification` helpers, which unwrap the `SessionScoped`/`notification` oneof wrappers that the fork's flat `remote_server.proto` envelope does not have.
- `app/src/remote_server/server_model.rs`: `#[cfg(test)] #[path = "server_model_tests.rs"] mod tests;` wiring, matching upstream.
- `app/src/remote_server/server_model_tests.rs` (new, fork-adapted): upstream's `test_model_with_diff_states`/`test_model_handle` helpers over the fork's ServerModel field set (no `auth_state`, `host_scoped_requests`, `git_status_models`, `github_repo_models`, `bundled_skills`, or remote-agent-context-snapshot fields), plus the `#[cfg(unix)] #[path = "server_model_unix_tests.rs"] mod unix;` include. Upstream's `test_model` helper and the entire non-unix test body are omitted: their subjects are fork-removed surfaces (remote Agent Mode context snapshot broadcast, bundled/home skill protos, host-scoped failover with `AuthState`), never ported.
- `app/src/remote_server/server_model_unix_tests.rs` (new): the daemon-side real-process cancellation test copied verbatim from upstream except the three `ClientMessage` constructions, adapted from upstream's `ClientMessage::notification(...)`/`ClientMessage::session_scoped(...)` wrapper constructors to the fork's flat struct-literal envelope. The test verifies the retained end-to-end contract the commit message describes: on `Abort`, the daemon cancels the tracked `RunCommand`, and the dropped `SpawnedChildCleanup` guard SIGKILLs the command's process group. The fork's `LocalCommandExecutor` already carries that guard; the test locks it in.
- `crates/remote_server/Cargo.toml`: `time` added to the tokio dev-dependency features. Upstream's identical manifest relies on feature unification through removed dependencies to expose `tokio::time::timeout`; the fork's trimmed dependency graph needs the explicit feature. Minimal fork build glue.

### `1b99853db` — Include clone command output in environment errors (#16115)

Decision: **reject / not applicable**. All seven touched paths are under `app/src/ai/agent_sdk/` (`driver.rs`, `driver/environment.rs`, `driver/failure_output.rs` and their test files, `driver/harness/mod.rs`), the Removed System absent from this fork. The 4-KiB secret-redacted clone-output attachment has no local terminal or ACP consumer here.

### `0e5949c92` — Answer the agent wake with AgentWake (#16152)

Decision: **reject**. Server-coupled client half of QUALITY-2088 (warp-server spec + warp-proto-apis#385):

- Requires the `warp_multi_agent_api` proto bump (`AgentWake` user input, `AgentMessageWake` query origin); the fork has no `warp_multi_agent_api` dependency at all.
- The blocklist/agent hunks all anchor on fork-absent symbols: `AIAgentInput::AgentWake` next to `OrchestrationConfigUpdate` variants (the fork's `AIAgentInput` enum diverged with no orchestration arms), `BaseUserQuery::is_agent_message_wake`, `send_agent_wake`, `QueuedQueryModel::is_dispatch_blocked` + `handle_pending_events_ready` (orchestration native-setup barrier), `OrchestrationEventService` echo-confirmation, `message_hydrator` `delivered_at` agent-message delivery tracking.
- Shared-session steered-path hunks and `app/src/server/telemetry/events.rs` are Removed Systems.

There is no server in this fork to wake a finished run and no multi-agent message routing, so no separable local bug fix exists in the diff.

### `2f5447c23` — Detect process OOMs in shutdown reports (#15998)

Decision: **reject / not applicable**. Cloud-agent harness shutdown reporting: `app/src/ai/agent_sdk/harness_support.rs` + new tests, `crates/warp_cli/src/harness_support.rs` + `lib_tests.rs`. The fork's `agent_sdk` directory is absent and `crates/warp_cli/src/` carries only `completions.rs`, `config_file.rs`, `json_filter*.rs`, and `lib.rs` — no harness-support surface, no `report-shutdown` control-plane path.

### `7aa4de12b` — Scope Bedrock token requests to the run team (#16133)

Decision: **reject**. Multi-team Bedrock identity-token minting/refresh: every touched path is a Removed System with no fork anchor — `app/src/ai/agent_sdk/**` (absent), `app/src/ai/{aws,bedrock,geap}_credentials*.rs` (absent), `app/src/server/{iap_identity_minter,server_api/managed_secrets}` (absent), `app/src/workspaces/user_workspaces/**` (absent), `app/src/tracing/cloud_agent_auth.rs` (absent), `app/src/settings_view/warp_agent_page.rs` (absent), `crates/managed_secrets/**` (absent), `crates/ai/src/api_keys.rs` (absent). Server-coupled via warp-server#18173.

## Verification

Run on `merge/upstream-2026-09-29` (with `CARGO_PROFILE_DEV_DEBUG=0`):

- `cargo check -p remote_server --all-targets --message-format short` — pass (pre-existing `client_event_kind` unused warning verified present on clean `main`).
- `cargo nextest run -p remote_server` — 55 passed, including the new `client::tests::timed_out_run_command_removes_pending_request_and_sends_abort`.
- `cargo check -p warp --all-targets --message-format short` — pass.
- `cargo nextest run -p warp -E 'test(remote_server::server_model)'` — 1 passed: `server_model::tests::unix::aborting_remote_run_command_kills_process_and_clears_in_progress_request`.
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- `cargo build -p warp --all-targets --message-format short` — pass.

Deleted-surface scans clean: scan 1 hits are `Weak::upgrade` false positives; scan 2 has no MCP/skills hits; scan 3 hits are retained SSH `ForwardX11=no` options and retained shell-bootstrap ConPTY commentary. `cargo clean` runs after merge/tag/push.
