# Upstream Master Audit — 2026-11-16

Incremental audit for `c6f07bb45..upstream/master` (`3741d2b74`), 4 commits: the first four PRs of the upstream agent-hooks Phase 1 stack, ported as four adapted port commits on `merge/upstream-2026-11-16`. The previous audit is `upstream-master-audit-2026-11-15.md`.

## Summary

All four commits are the agent-hooks Phase 1 stack (canonical model, Cursor adapter, command runner). The feature is purely local — user-configured scripts fired at fixed points in the agent loop, no Warp service/account/billing/telemetry dependency — so it is in scope under the local/provider-backed feature rule. The interception core gates the retained blocklist action loop (`try_to_execute_action`, the same host-side executor that runs ACP-proposed terminal commands and LRC activity), and the observer boundaries attach to the fork's live ACP flow. Ported with the `AgentHooks` flag off by default, exactly matching upstream's off-on-every-channel Phase 1 state: `agent_hooks` cargo feature, not in `default`, bridged in `app/src/lib.rs`.

Port commits: `96fa77b2f` (scaffold, ffe91a0ef), `f5d1c1dae` (Cursor deny slice, b786054a4), `1cdc78e98` (complete Cursor adapter, 62c562545), `c870a0bec` (production runner, 3741d2b74), plus a small cleanup commit for an unused helper.

## Commit-by-Commit

### `ffe91a0ef` hooks: scaffold canonical model and interception points (#16577) — adapted port

`crates/ai/src/hooks/**` (model, dispatch, registry, permission, trust, notices, runner trait, test doubles) and `app/src/ai/blocklist/hooks/**` copied verbatim except the adaptations below. `NotExecutedReason::WaitingOnHooks` + requeue-at-index, `HookVerdictResolved` handling, `resume_action_after_hooks`, pre-tool deny via `HookPermissionPolicy::compose`, `finish_sync`, `tool_result::observe_terminal_result` on the async/sync/cancel funnels, `pre_tool::invalidate` on result finalize and edited-command acceptance, and `AIAgentActionResultType::DeniedByHook` (+ Display/description/is_failed/Markdown arms) all ported.

Fork adaptations and omissions:

- Omitted fork-absent paths: `app/src/ai/agent/api/convert_{conversation,to}.rs` (server request conversion, incl. the `denied_by_hook` proto member), `conversation_yaml.rs`, `agent_sdk/driver/output.rs`, `orchestration_event{,s,_streamer,_tests}.rs`, and the workspace `Cargo.toml` `warp_multi_agent_api` proto rev bump (fork has no warp-proto dependency).
- `warpui_core::AppContext::has_singleton_model` ported from upstream `6d8562c39` as a prerequisite (the video-recording commit itself remains rejected).
- Driver checks (`owns_turn_lifecycle`/`executes_tool_calls_locally`) reduce to conversation existence: the fork has no `ConversationDriver` split (no shared-session viewers, CLI transcripts, remote children, or external harnesses).
- Envelope builders drop `AuthStateProvider` (user_email is `None`; no accounts) and map surfaces without autonomous/TUI/Oz modes (`AppExecutionMode::allows_active_ai` → Gui, else CliRun; run metadata stays a local CliRun surface). `repository_root` uses the fork's direct `&Path` signature. `cancellation_reason_name` covers the fork's variant set (`OptimisticCLISubagentCompletion` → `command_finished`).
- The stream observer (`StreamObserverState` state machine) is copied verbatim, but its feed is fork glue: `AcpStreamDelta` observations produced at `AcpAgentModel::apply_event_to_history` (assistant text/thought deltas keyed by the history model's live `acp-{stream}-*` message ids, tool-call boundaries as `OtherMessage`, `AssistantThoughtFinished` at `AcpEvent::Completed` since ACP carries no reasoning duration), with `flush_on_stream_end` on Completed/Cancelled/Failed and `turn.stop(Completed)` on Completed and on the controller's all-actions-successful branch. Upstream's MAA `collect_observations`/`observe_message`/`MessageDelivery` are not portable.
- `prompt.submit` fires in the fork's `send_query` (same position as upstream: after cancelling the previous turn's actions, before the request); `session.start` is unconditional in `start_new_conversation` (no viewer/transcript flags).
- `queue_actions` ported without upstream's MAA preprocessing pipeline (the fork has no wire-driven actions; recorded `#[cfg(test)]` until a producer exists — the fork's action queue has no runtime producer since ACP tool calls render protocol-natively).
- Tests: hooks unit tests + `execute_tests`/`action_model_tests` hooks modules ported with harness adaptations (no `RecordingController`, no `UserWorkspaces` resolver, 6-arg action model `new`, `AIConversation::new()`); the MAA-driven `controller_tests` hooks module and the two RunAgents verdict-invalidation tests are omitted (fork-absent harness/paths); shared-session/viewer negative tests replaced with unknown-conversation equivalents; the proto wire-conversion assertion dropped.

### `b786054a4` hooks: deny a real shell command through a real Cursor hook (#16579) — adapted port

Minimal Cursor adapter (`beforeShellExecution`-only config/decode), the real-process `CommandHookRunner`, registry `install_explicit_source` file loading, `HookRuntime::install_command_runner` behind `local_tty` with login-shell PATH capture, adapter table registration, `e2e_slice_tests`, and the real-script-deny e2e test in `execute_tests` — all ported. `crates/ai` gains `async-process`/`command` deps without upstream's wasm target-family gate. `command_windows_tests.rs` omitted together with its `cfg(all(test, windows))` mod declaration (upstream deletes it in 3741d2b74); the runner's `#[cfg(windows)]` PowerShell arm stays verbatim and inert.

### `62c562545` hooks: complete the Cursor adapter for all 21 events (#16582) — adapted port

The full events table (all 21 Cursor event names, boundary projections, control effects, matcher targets), complete `hooks.json` parsing, matcher semantics, per-source state, `tool_names`, stdin encoding, output decoding rules, and the checked-in fixtures — ported. The fork's tool vocabulary lacks RunAgents/MCP/skills/recording/suggestion actions, so:

- `tool_names`/`tool_input`/`tool_output`/`facets` reduce to the fork's action and result variants; `FileEdit::Create` has no `allow_overwrite`, `ReadFilesResult::Success` has no `failed_files`, command results carry no timestamps (no duration in shell events, `ShellRecovered` views omitted), and computer-use views match the trimmed `computer_use::ActionResult`.
- MCP events parse and report but can never fire: the MCP facet keeps its upstream types but returns `None` (no `rmcp` dependency; MCP belongs to the ACP agent process). The MCP stdin fixtures stay on disk unexercised; the fixture round-trip test skips them; matcher tests assert the never-matching behavior.

### `3741d2b74` hooks: production command runner with process tree cleanup (#16583) — adapted port

Production `CommandHookRunner` (deadlines, whole-tree stop/grace/kill, bounded stdout/stderr tail, four-slot semaphore, one log line per invocation), the two-layer environment policy (`env.rs` scrubbed startup snapshot + protected names), `app/src/process_environment.rs` capture/registration, and the runner env argument in `install_command_runner` — ported. `crates/command` gains the unix process-group `JobGuard` (`new_in_job`/`spawn_in_job`/`killpg_says_gone` + `async_tests`); the Windows Job Object support (`windows.rs`, `crates/command` windows Cargo features) is omitted per the macOS-only host policy, while the verbatim `#[cfg(windows)]` arms in `async.rs`/`command.rs`/their tests stay inert. `async-lock` added to the workspace. Upstream `AGENTS.md`, `.agents/skills/rust-unit-tests/SKILL.md`, and `specs/agent-hooks/architecture.md` changes stay out (fork-diverged guidance / rejected specs).

## Restored-Surface Scans

Re-run with the three skill scans. New hits are all from the ported upstream library code and inert: `mcp_server_denylisted` is a permission-floor boolean inside `crates/ai/src/hooks/permission.rs` (copied upstream policy, not MCP server management), `#[cfg(windows)]` arms in `crates/command/src/async.rs`, `crates/ai/src/hooks/runner/command.rs`, and their cfg-gated windows test modules compile out on macOS, and scan-1 hits are `Weak::upgrade` word matches plus a pre-existing fork note. No auth/billing/GraphQL/Sentry/telemetry/MCP-management/skills surface was reintroduced.

## Verification

All commands run with `SDKROOT` pointed at the full Xcode SDK and `CARGO_PROFILE_DEV_DEBUG=0`:

- `cargo fmt -- --check` — passed repo-wide.
- `cargo check -p warp --all-targets --message-format short` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 157 passed.
- `cargo nextest run -p ai` — 250 passed (hooks library, Cursor adapter fixtures, e2e real-process deny and process-tree-kill).
- `cargo nextest run -p command` — 6 passed (JobGuard unix tests).
- `cargo nextest run -p warp -E 'test(hooks) | test(real_process)'` — 57 passed.
- `cargo check --workspace --all-targets --message-format short` — passed (after clearing a full data disk with `cargo clean`; target/ was 37G regenerable cache).
- `cargo build -p warp --all-targets --message-format short` — passed, followed immediately by `cargo clean`.

## Tag

`v2026.11.16` on `main` (four adapted port commits merged with `--no-ff`).
