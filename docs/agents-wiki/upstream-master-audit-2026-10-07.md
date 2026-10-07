# Upstream Master Audit — 2026-10-07

Incremental audit for `c75f182d1..upstream/master` (`f571865ca`), 8 commits, completing the currently fetched upstream master tip. The previous audit is `upstream-master-audit-2026-10-06.md`.

## Summary

Three adapted ports (the CLI-agent `agent_needs_input` parse layer, the Docker-sandbox `ShellLaunchData` mapping fix, and the `TerminalBusy` busy-terminal requested-command fix), five zero-port commits (cloud run-cloud help, MCP OAuth issuer, build-cache diagnostics, CI timeout, runner-enum tolerance, plus the rejected shared-session injection core). No removed product surface was restored.

## Commit-by-Commit

### `88afc88fe` fix(cli): correct run-cloud attachment help — rejected (fork-absent)

Only touches `crates/warp_cli/src/agent.rs`, correcting `warp agent run-cloud` attachment help text from 5 to 25 files. The fork's `warp_cli` is the trimmed local lib (`completions.rs`, `config_file.rs`, `json_filter*.rs`, `lib.rs`) with no `agent.rs`; run-cloud is the removed cloud-agent CLI surface. Runtime behavior upstream is unchanged anyway (help string only).

### `9ccb4e3e3` Handle CLI agent needs-input in unattended runs — adapted port (parse layer only)

Upstream parses the new `agent_needs_input` plugin notification (claude-code-warp 2.3.0) as `CLIAgentEventType::NeedsInput`, maps it to `Blocked` in autonomous execution modes only (SDK/daemon), adds `BlockedSource` to `CLIAgentSessionStatus::Blocked`, and skips the typed `/exit` ladder for needs-input-blocked sessions. Upstream's own description states the desktop app and TUI keep ignoring the event, so interactive behavior is unchanged — and this fork is desktop-only with no autonomous run modes (`warp_core::execution_mode` has only `App`/`Headless`, no `is_autonomous`).

Ported (fork's `cli_agent_sessions` is the retained live host for CLI-agent protocol events, so the vocabulary stays current with the shipping plugin):

- `app/src/terminal/cli_agent_sessions/event/mod.rs`: `NeedsInput` variant — verbatim.
- `app/src/terminal/cli_agent_sessions/event/v1.rs`: `"agent_needs_input" => CLIAgentEventType::NeedsInput` — verbatim.
- `app/src/terminal/cli_agent_sessions/mod.rs`: `CLIAgentEventType::NeedsInput => return None` arm in `apply_event`, placed after `QuestionAsked` as upstream does, carrying upstream's interactive-ignore comment adapted to note the fork has no unattended run modes.
- `app/src/terminal/cli_agent_sessions/mod_tests.rs`: `parse_agent_needs_input_notification` — verbatim.

Omitted:

- `BlockedSource`, the `Blocked { message, source }` reshaping, `is_blocked_on_needs_input`, and the autonomous-mode gate in `update_from_event`: their only consumers are the agent-SDK exit ladder and unattended-run status mapping, both fork-absent; in the fork the event is ignored exactly as before (previously as `Unknown`, now explicitly).
- `app/src/ai/agent_management/`, `app/src/ai/agent_sdk/driver*`, `app/src/ai/blocklist/local_agent_task_sync_model*`, and the `app/src/terminal/view.rs` Blocked-pattern hunk: fork-absent removed surfaces / dependent on the omitted reshaping.

### `617c3a996` fix(mcp): forward OAuth callback issuer to rmcp — rejected (removed surface)

App-managed MCP OAuth: `app/src/ai/mcp/templatable_manager*` and `crates/mcp/src/oauth*` are all removed/absent in this fork. MCP server configuration and OAuth belong to the ACP agent process.

### `f61ad219e` Improve build cache failure diagnostics — rejected (fork-absent)

Entirely inside `crates/build_cache/**`, absent in this fork (recorded in the 2026-09-26 and 2026-10-06 audits). No anchor symbols.

### `c03b5ed9e` Raise the CI tests job timeout from 25 to 35 minutes — not applicable (fork-owned CI)

Only `.github/workflows/ci.yml`. The fork's CI workflow is fork-owned and diverged; consistent with prior audits treating upstream CI-workflow commits as not applicable.

### `125940e41` Run harness plugin setup in the session's shell on Windows — adapted port (terminal_manager hunk)

The commit's driver/plugin-manager changes live in the removed agent-SDK harness stack, but its `app/src/terminal/local_tty/terminal_manager.rs` hunk is a retained local fix: `on_shell_determined` mapped a `ShellStarter::DockerSandbox` shell to `ShellLaunchData::Executable`, misclassifying retained local `sbx` Docker-sandbox sessions (persisted shell launch data, shell indicator, path conversion) as ordinary executable shells.

Ported:

- `app/src/terminal/local_tty/terminal_manager.rs`: `ShellStarter::DockerSandbox(docker_starter) => ShellLaunchData::DockerSandbox { sbx_path: docker_starter.logical_shell_path().to_owned(), base_image: docker_starter.base_image().map(str::to_owned) }` — byte-identical to the upstream hunk; the fork's `ShellLaunchData::DockerSandbox` variant and `DockerStarter::base_image()` already existed.

Omitted: `app/src/ai/agent_sdk/driver.rs`, `driver/terminal.rs`, `driver_tests.rs` — fork-absent harness/plugin-manager surface (the Windows WSL-bash resolution this commit chases only runs there).

### `0fbb5419a` Tolerate unrecognized runner OS, architecture and macOS version values — rejected (fork-absent)

`app/src/ai/agent_sdk/runner*.rs`, `app/src/ai/runner_display*.rs`, and `crates/graphql/**` are all removed/absent (cloud runner inventory over GraphQL). No fork anchors; remote-host platform detection for SSH does not flow through these enums.

### `f571865ca` Interrupt shell commands for shared-session follow-ups — adapted port (TerminalBusy piece only); injection core rejected

The commit's core feature — interrupting an agent-owned shell command with raw Ctrl-C for a primary-directed follow-up injected through shared session — depends on the removed session-sharing/steering stack (`session_sharing_protocol`, `warp_multi_agent_api`, `SharedSessionPrompt` row kinds, the server-side warp-server#18940 support, and a local proto override in `Cargo.toml`/`Cargo.lock` that must not be ported). Rejected for the same reasons as `ff9dbf463` and `1012545632`.

Its separable local improvement is ported: a requested command against a busy terminal now returns an honest recoverable `TerminalBusy` result instead of `CancelledBeforeExecution`, so the conversation is not falsely treated as cancelled and the follow-up request fires (`should_trigger_request_upon_completion()` is `!is_cancelled()`, and `TerminalBusy` is not cancelled).

Ported:

- `crates/ai/src/agent/action_result/mod.rs`: `TerminalBusy { command, block_id }` variant plus `is_successful`/`failed`/`Display` arms — byte-identical to upstream.
- `app/src/ai/blocklist/action_model/execute/shell_command.rs`: the busy-terminal check now returns `TerminalBusy { command, block_id }` — upstream hunk, kept in the fork's original position (the fork lacks the upstream displaced-conversation guard that this commit reordered around; that guard belongs to a different upstream commit and its `active_conversation_id()` block-list API is fork-absent).
- `app/src/ai/blocklist/inline_action/requested_command.rs`: `TerminalBusy` footer in `maybe_render_footer` and the non-expandable arm — upstream hunks (the upstream `has_finished_command_block` context is pre-existing upstream code the fork never received and is not part of this commit).
- `app/src/ai/agent/mod.rs`: `MarkdownActionResult` Display arm for `TerminalBusy` — byte-identical to upstream.
- `app/src/ai/blocklist/action_model/execute/shell_command_tests.rs`: `terminal_busy_does_not_write_or_cancel_the_running_command`, adapted: dropped the `RecordingController` registration (fork-absent computer-use recording controller) and the displaced-conversation segment (asserts the fork-absent displaced guard and `set_active_conversation_context` API); the rest is verbatim upstream.

Omitted:

- `app/src/ai/blocklist/controller.rs` (injection predicate, `commands_interrupted_for_injection`, grace period), `app/src/terminal/view.rs` (`InterruptForInjectedFollowup` raw-ETX handler), `shell_command.rs`'s `interrupt_for_injected_followup` method and event variant, `crates/ai/.../convert.rs` proto wire conversion, `crates/warp_tui/**`, `app/src/ai/agent/api/convert_conversation*`, `conversation_yaml.rs`, `base_user_query.rs`, and the `Cargo.toml`/`Cargo.lock` proto override: all shared-session/MAA/proto surfaces with no fork consumer or anchor.
- The `injection_interrupt_keeps_completion_waiter_until_normal_precmd` test (exercises the omitted interrupt method).

## Verification

Results after the ports settled:

- `cargo fmt -- --check`: clean.
- `cargo check -p warp -p ai --all-targets --message-format short`: clean (only pre-existing dead-code warnings in untouched files).
- `cargo check --workspace --all-targets --message-format short`: clean.
- `cargo nextest run -p warp -E 'test(parse_agent_needs_input_notification) | test(terminal_busy_does_not_write_or_cancel_the_running_command) | test(block_working_directory_updated_does_not_drain_finish_senders)'`: 3 passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions) | test(cli_agent_sessions)'`: 204 passed.
- `cargo build -p warp --all-targets --message-format short`: succeeded, followed immediately by `cargo clean`.
- Restored-deleted-surface scans (`rg` over `app crates script Cargo.toml`): no new hits beyond the previously documented allowances (ONNX tokenizer vocabulary tokens, SSH `ForwardX11=no`, retained bootstrap ConPTY comments).
