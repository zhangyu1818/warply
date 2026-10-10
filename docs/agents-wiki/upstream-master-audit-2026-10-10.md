# Upstream Master Audit — 2026-10-10

Incremental zero-port audit for `9ec105c94..upstream/master` (`721449fde`), 11 commits, completing the currently fetched upstream master tip. The previous audit is `upstream-master-audit-2026-10-09.md`.

## Summary

All 11 commits belong to two upstream workstreams — the agent-sdk ACP harness stack (PRs #16330, #16355, #16331, #16332, #16364 plus the #16304/#16289 cloud-execution/config prerequisites) and the cloud Oz ambient-session workstream (#16148 shell recovery, #16391 handoff-checkpoint promotion). Every primary surface is fork-absent (`app/src/ai/agent_sdk/`, `app/src/terminal/shared_session/`, `crates/build_cache/`, `crates/graphql`, `crates/warp_tui`, `app/src/features.rs`) or a removed product area (cloud agent execution config, ambient sessions, handoff checkpoints). Shared-file hunks in fork-present files were each inspected and verified anchorless on symbols this fork never received (the three `AIConversation` driver booleans, `session_computer_use`/`computer_use_llm_override` models, `ShellProcessExited` payload, `CloudAgentShellRespawn`, `PeriodicHandoffCheckpoints`). No removed product surface was restored and no code was ported.

## Commit-by-Commit

### `81e553227` Agent driver: run an ACP harness inside the terminal session over a socket bridge (#16330) — rejected (removed surface)

PR 2/4 of the upstream ACP harness stack. Adds `agent_sdk/driver/harness/acp/bridge.rs` (`BridgeListener`, one-time-token launch file, Unix-socket/loopback-TCP transport) and the hidden `WorkerCommand::AcpBridge` subcommand. All implementation paths live under the fork-absent `app/src/ai/agent_sdk/`; the two shared-file hunks (`app/src/lib.rs`, `crates/warp_cli/src/lib.rs`) only add the `AcpBridge` dispatch arm and clap variant that call `agent_sdk::driver::harness::acp::run_bridge`, which does not exist here. Same classification as the `HarnessTransport::Acp` commit rejected on 2026-10-09: the direction (driving harnesses over ACP) matches this fork's ACP-only ownership, but the implementation is owned end-to-end by the removed upstream driver.

### `b61e9ca31` Load cloud agent execution configuration from the server (#16304) — rejected (removed surface)

Fetches cloud agent execution config (repos, setup commands, model choice, secrets, attachments) via a new GraphQL query instead of CLI flags, gated on `--execution-id`/`WARP_EXECUTION_ID` and `CloudAgentExecutionConfig`. All primary surfaces fork-absent (`crates/graphql`, `agent_sdk/**`, `server_api/managed_secrets`, `warp_cli/agent.rs`, `pane_group/pane/local_harness_launch.rs`, `shared_session`). The fork-present hunks are dead-code unlocks for models this fork never received:

- `app/src/ai/blocklist/permissions.rs` + `permissions_tests.rs`: `get_computer_use_setting(terminal_view_id, ..)` grows `team_autonomy_settings().computer_use_setting` and `AIExecutionProfilesModel::session_computer_use` lookups — the fork's copy still has the older `get_computer_use_setting_for_profile(ctx, profile_id)` shape (no terminal-scoped variant; the computer-use preference-model commit was rejected on 2026-09 as fork-absent).
- `app/src/ai/execution_profiles/profiles.rs`: removes `#[cfg_attr(not(test), expect(dead_code))]` from `SessionComputerUse` and its three accessors; the fork has no `session_computer_use` field or accessors.
- `app/src/ai/llms.rs`: same dead-code-marker removal for `set_computer_use_llm_override`/`clear_computer_use_llm_override`; the fork has no `computer_use_llm_for_terminal_view`.
- `app/src/terminal/view/docker_sandbox/mod.rs` + `shared_session/sharer/network_tests.rs` hunks are tests for those rejected models / fork-absent sharing surfaces.

### `b113a6c95` Cache structured environment checkouts with full Git mirrors (#16289) — rejected (removed surface)

Cloud-agent environment-checkout infrastructure: bare git mirrors in the persistent build cache, `agent_sdk/driver/environment_checkout*.rs`, hidden `CliCommand::EnvironmentCheckout`, and the `GitMirrorCache` feature flag. All implementation paths fork-absent; the shared-file hunks only wire `ai::agent_sdk::run_environment_checkout` (nonexistent here) and would add a permanently dead flag.

### `fc4b06fb4` Use snake_case for Namespace cache metadata (#16379) — not applicable

Only touches `crates/build_cache/src/metadata*.rs` and `lib_tests.rs`. `crates/build_cache/` is fork-absent (build-cache permissions commit previously rejected).

### `b2b7357dd` Recover cloud agent shells after process exit (#16148) — rejected (removed surface)

REMOTE-2243: when an agent-requested command terminates the persistent Bash shell of a shared ambient (cloud Oz) session, spawn a replacement shell in the same sandbox and resolve the interrupted tool call with `ShellRecovered`. Recovery is gated on `FeatureFlag::CloudAgentShellRespawn` and only runs for an active sharer of a shared ambient session past bootstrap; upstream explicitly excludes local runs, so no local-terminal behavior is being missed. The fork has no `app/src/terminal/shared_session/`, no ambient sessions, no `CloudAgentShellRespawn`, and its `ExitReason::ShellProcessExited` (`app/src/terminal/model/terminal_model.rs`) is still the unit variant — the payload/`ObservedExitStatus` wire stack, `ShellStarter::replacement()`, `mio_channel` receiver re-registration, `recover_cloud_shell`, the `shell_recovery` poll state in `blocklist/action_model/execute/shell_command.rs`, the `ShellRecovered` action-result variants through `crates/ai/src/agent/action_result/*` and `agent/{mod.rs, redaction.rs}`, and the `cloud_agent_shell_respawn` Cargo feature + `prost` dep all serve that one trigger and would land as permanently dead code. Also touches fork-absent `warp_tui`, `app/src/features.rs`, and `crates/integration` test surfaces.

### `760d01e53` Agent Mode: model who drives a conversation as a ConversationDriver enum (#16355) — rejected (premise absent)

Pure refactor consolidating `is_viewing_shared_session`/`is_cli_agent_transcript`/`is_remote_child` into `ConversationDriver` with predicate methods, as prep for the harness stack. The fork's `AIConversation` (`app/src/ai/agent/conversation.rs`) has none of the three booleans — they arrive with the unported shared-session/cloud lineage — so the enum has no premise here, and its only consumer (the external-harness driver, below) is not ported.

### `f386a4cc8` Agent Mode: let an external harness drive a native conversation turn (#16331) — rejected (removed surface)

PR 3/4 of the harness stack: `ConversationDriver::ExternalHarness`, `controller/external_harness.rs` (`begin_external_harness_turn`, `apply_external_harness_event`, prompt sink), and the `apply_response_event` extraction from `handle_response_stream_event`. Builds directly on the rejected `ConversationDriver` enum; the fork-present hunks are anchorless — `startup_queue.rs` diverts shared-session injections to the sink using `ambient_agent_task_id` (ambient field absent here), and `history_model.rs` adds `set_driver_for_conversation` requiring `ConversationDriver`. The refactor's only behavior deltas are keyed on driver predicates that are invariant for the fork's all-native conversations, so porting the code motion alone would be a no-op plus dead sink plumbing.

### `5c84259e2` REMOTE-2111: promote periodic handoff checkpoints to stable (#16391) — rejected (removed surface)

Flag-only promotion of `PeriodicHandoffCheckpoints` from `PREVIEW_FLAGS` to `RELEASE_FLAGS`. The flag does not exist in this fork (cloud agent handoff orchestration removed); nothing to promote.

### `6f120ee52` Give two Windows CI tests more shutdown time (#16395) — not applicable

Adds a Windows-only `[[profile.ci.overrides]]` slow-timeout for `ai::agent_sdk::driver::mcp_startup::tests::configured_and_profile_mcp_servers_wait_for_environment_setup` and `terminal::view::tests::revoking_remote_session_ai_takes_effect_without_a_new_terminal_session`. The fork is macOS-only, both named tests are fork-absent (agent_sdk; remote-session AI revocation), and the existing `update_manager` Windows override in the fork's `.config/nextest.toml` is untouched by the hunk.

### `02668f9be` Agent driver: ACP client, MAA mapping, and the ACP harness runner (#16332) — rejected (removed surface)

PR 4/4 of the harness stack: the agent_sdk ACP connection/mapping/policy/protocol modules and the runner that drives `external_harness` turns. The fork-present hunks are dead-code plumbing only: `blocklist/mod.rs` re-exports `controller::external_harness::{ExternalHarnessPrompt, ExternalHarnessTurn}` (rejected above) and widens `permissions::*` visibility for the agent_sdk mapping, and `agent/api.rs`/`api/convert_to.rs` make `serde_json_to_prost` `pub(crate)` for the same mapping — with no agent_sdk consumer, the re-exports would be unused-import dead code.

### `721449fde` [REMOTE-3428] Debounce third-party harness transcript saves (#16364) — not applicable

Debounces `agent_sdk/driver/harness/save_coordinator.rs` transcript saves. All five touched paths are under the fork-absent `app/src/ai/agent_sdk/`.

## Verification

Zero-port audit: no source changes, so verification re-confirms the unchanged tree.

- `cargo fmt -- --check`: clean.
- `cargo check -p warp --all-targets --message-format short`: clean.
- `cargo check --workspace --all-targets --message-format short`: clean.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: passed.
- `cargo build -p warp --all-targets --message-format short`: succeeded, followed immediately by `cargo clean`.
- Restored-deleted-surface scans (`rg` over `app crates script Cargo.toml`): no new hits beyond the previously documented allowances.
