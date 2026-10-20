# Upstream Master Audit — 2026-10-20

Incremental zero-port audit for `ae60375ef..upstream/master` (`7082729a1`), 7 commits. The previous audit is `upstream-master-audit-2026-10-19.md`.

## Summary

All seven commits extend workstreams whose runtime surfaces are fork-absent: the pre-warmed standby sandbox stack (rejected 2026-10-16..19) gains credential isolation and two sandbox-lifecycle fixes; the removed Agent SDK's OpenTelemetry spans, terminal exit-waiter, and managed git credentials receive fixes; the removed server-backed agent-management fetch path gains a CLI-mode guard; and the `AcpHarness` rollout flag (whose only consumer is the removed Agent SDK) is promoted to dogfood. The one ACP-adjacent change (`1e1acb4d7`, Claude model env vars) lives entirely in the removed `agent_sdk` harness driver — the fork's ACP client has no Warp-side model selection: `AcpAgentLaunch` env is user-owned config (`app/src/ai/acp/backend.rs`), so there is no live anchor to port onto. Nothing was ported.

## Commit-by-Commit

### `d5667a308` Send sandbox credentials on standby GraphQL requests (#16542) — rejected (removed surface)

Credential isolation for the pre-warmed standby sandbox stack (REMOTE-3568, parts rejected as N.A. on 2026-10-16/17/18/19): standby GraphQL requests carry the sandbox ID and pre-warmed workload token instead of ambient run credentials, `--standby-id` selects the standby identity before authenticated background requests, and both identity header names are reserved against configured overrides. Touched paths: `app/src/lib.rs` (adds a `LaunchMode::CommandLine` arm for `CliCommand::Agent(AgentCommand::RunCloudStandby(args))` calling `server_api.set_prewarmed_sandbox_id`), `app/src/server/server_api/prewarmed_sandbox.rs`, `crates/warp_server_client/src/base_client*.rs`. The fork has no `app/src/server`, no `warp_server_client` crate, and `rg "RunCloudStandby|set_prewarmed_sandbox_id|prewarmed_sandbox|warp_server_client"` over `app crates Cargo.toml` returns zero anchors, so even the one fork-present file (`app/src/lib.rs`) has no live insertion point.

### `945201940` Fix Git mirror checkout span attributes (#16550) — rejected (removed surface)

OpenTelemetry span fix in `app/src/ai/agent_sdk/driver/environment_checkout.rs` (single file, ~600 lines reflowed): mutable booleans recorded as span attributes are declared `Empty` and written once after the inner checkout future completes so later records stop appending duplicate attributes that make Cloud Trace retain the initial `false`. Both the owning `agent_sdk` module and the telemetry/tracing pipeline are removed fork surfaces; `rg "environment_checkout"` over `app crates` returns zero anchors.

### `2511c797f` Skip filtered management run fetches in CLI mode (#16546) — not applicable

Adds an execution-mode guard (`AppExecutionMode::can_fetch_agent_runs_for_management()`) at the top of `AgentConversationsModel::fetch_tasks_for_filters` plus a regression test, stopping background management run fetches (e.g. `WindowTeamChanged`-triggered) in SDK/CLI mode. The fork retains `AppExecutionMode` as a general model, but its `agent_conversations_model.rs` is the minimal local-history version: `rg "fetch_tasks_for_filters|can_fetch_agent_runs|trigger_filter_fetch"` returns zero matches, and the model performs no server run fetches at all (`get_ai_client`/`fetch.*tasks` absent), so the guarded path does not exist here.

### `00c9593cd` Match terminal command exit to the block the command started in (#16548) — not applicable

Fixes the removed Agent SDK's `TerminalDriver` exit-waiter: `waiting_command` becomes a `WaitingCommand { exit_tx, started_block_id }`, the block ID published by the pending-start waiter is recorded, and `BlockCompleted` resolves the exit only when `started_block_id` matches, ignoring unrelated completions (bootstrap/Precmd blocks) with an info log. All changed logic lives in `app/src/ai/agent_sdk/driver/terminal.rs` + `terminal_tests.rs`; the fork retains the terminal-side primitive `execute_command_or_set_pending` (`app/src/terminal/view.rs:11119`) unchanged by this commit, and the fork's ACP terminal capability executes commands by spawning processes directly and reading their exit status (`app/src/ai/acp/model.rs`, `TerminalExitStatus`), so the block-based exit-waiter bug surface does not exist in this fork.

### `393e97a30` Promote AcpHarness to dogfood (#16554) — not applicable

Moves `FeatureFlag::AcpHarness` from `LOCAL_FLAGS` to `DOGFOOD_FLAGS` in `crates/warp_features/src/lib.rs` so Dev-channel sandbox agent binaries accept `--harness-transport`/`WARP_HARNESS_TRANSPORT`. The fork has no `AcpHarness` flag (`rg -l "AcpHarness" app crates` returns nothing); its only upstream consumer is the `run_agent` CLI gate in the removed `app/src/ai/agent_sdk/mod.rs`, and the fork has no cloud-agent sandbox transport selection path.

### `1e1acb4d7` Carry the Claude model into ACP sessions via `ANTHROPIC_MODEL` and `ANTHROPIC_CUSTOM_MODEL_OPTION` (#16556) — not applicable

For the Claude harness over ACP inside the removed Agent SDK, exports `ANTHROPIC_MODEL` and `ANTHROPIC_CUSTOM_MODEL_OPTION` through `harness_env_vars` (`app/src/ai/agent_sdk/driver/harness/mod.rs`) so the adapter's model picker advertises the requested full model id and the strict `session/set_model`-adjacent check in `model_config.rs` passes. Both touched files are fork-absent, and `rg "ANTHROPIC_MODEL|ANTHROPIC_CUSTOM_MODEL_OPTION|harness_env_vars|configure_model_and_thinking"` over `app crates` returns zero anchors. The fork's ACP client has no Warp-side model selection to fix: `app/src/ai/acp/backend.rs` builds the adapter command from the user-owned `AcpAgentLaunch` env map and command line, and fork contract assigns model/backend choice to that ACP config. A fork user who wants these vars sets them in their agent config; injecting them app-side would be new handwritten behavior with no upstream anchor on a fork-present path, which the source-first rules forbid.

### `7082729a1` Stop git from erasing managed credentials after a 401 (#16559) — rejected (removed surface)

Turns the Agent SDK's git credential helper (`app/src/ai/agent_sdk/driver/git_credentials.rs` + tests) into a `get`-only helper so git's `store`/`erase` operations cannot mutate the managed `~/.git-credentials` file shared across sandbox repositories. Managed credentials for cloud sandboxes are a removed surface, `agent_sdk` is fork-absent, and `rg "git_credentials|GitCredentialHelper|managed_credential"` over `app crates` returns zero anchors.

## Verification

Zero-port audit: no source changes, so verification re-confirms the unchanged tree (`git diff main` empty).

- `cargo fmt -- --check` — passed.
- `cargo check -p warp --all-targets --message-format short` — passed (SDKROOT pointed at the Xcode MacOSX SDK per the CLT tapi prerequisite).
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 tests passed, 0 failed.
- `cargo build -p warp --all-targets --message-format short` — passed; `cargo clean` run after the push.
- Restored-surface scans — zero delta against `main` for all three scan patterns (pre-existing allowed hits unchanged).

## Tag

`v2026.10.20` on `main` (merge commit of `merge/upstream-2026-10-20`).
