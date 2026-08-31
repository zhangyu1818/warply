# Upstream Master Audit 2026-08-31

## Scope

- Current fork before this audit: `3392688e0` (`main`, `v2026.08.29`).
- Upstream source reviewed: `066ec71b7..upstream/master` (5 commits, tip `86cfeb9006`).
- Result: one adapted port (`76cfd2c17`, LRC liveness signals); the other four commits rejected or not applicable.

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `76cfd2c17` | Collect LRC liveness signals so the agent stops killing healthy commands (#14777) | **Adapt (ported)** | See port record below. |
| `17f4ec80f` | Make computer-use Warp UI verification opt-in (#15672) | **Reject / not applicable** | Touches `resources/channel-gated-skills/dogfood/{test-warp-ui,verify-ui-change-in-cloud}/SKILL.md` (channel-gated skills are a removed surface that must remain absent) and `.agents/skills/review-pr-local/SKILL.md` (fork skill diverged; standing rejection from the 2026-08-12 audit). |
| `64659feae` | Replace release asset action with gh CLI (#15678) | **Not applicable** | Upstream `create_release.yml` upload steps (`softprops/action-gh-release` → `gh release upload`) have no anchors in the fork-owned Sparkle workflow, which already creates releases with `gh release create`. Standing rule: the fork's `create_release.yml` accepts no upstream release-job hunks. |
| `9c2879ab6` | Promote NativeShellCompletions from Dogfood to Preview (#15680) | **Not applicable** | Moves the flag from `DOGFOOD_FLAGS` to `PREVIEW_FLAGS`; the fork has already wired `NativeShellCompletions` default-on through the cargo-feature pipeline (2026-08-28 audit), which supersedes upstream's rollout-list movement. |
| `86cfeb900` | Upload only top-level GitHub release package files (#15681) | **Not applicable** | Fixes `packages_dir` glob expansion in upstream release jobs; the fork's workflow has no `packages_dir` upload steps. |

## Port record: `76cfd2c17` LRC liveness signals

### Runtime-ownership review

Long-running command (LRC) control transfer is a retained area. In this fork the
snapshot path is live end to end without the Warp server:

- `ShellCommandExecutor` runs `RequestCommandOutput`/`WriteToLongRunningShellCommand`/`ReadShellCommandOutput`/`TransferShellCommandControlToUser` actions locally, including for the retained CLI-subagent takeover path (`CLISubagentController`) and user-accepted requested commands (`handle_requested_command_accepted`).
- Finished snapshot results drain into `AIAgentInput::ActionResult` (`BlocklistAIController::send_follow_up_for_conversation`) and are serialized into the ACP prompt via `AIAgentInput`'s `Display`.
- The same serde types persist into local conversation history.

The judging agent in this fork is therefore the ACP/CLI agent reading the
follow-up prompt, so the feature's core (local process-tree evidence attached to
snapshots) runs fully locally with no Warp service dependency. Ported.

### Applied from the exact upstream source

- `crates/ai/src/agent/action_result/mod.rs`: `LrcActivity`, `LrcProcessActivity`, `LrcProcessState` types and the `activity: Option<LrcActivity>` field on the four snapshot variants (clean three-way apply).
- `app/src/ai/blocklist/action_model/execute/lrc_activity.rs`, `lrc_activity/sampler.rs`, `lrc_activity_tests.rs`: exact upstream copies (monitor accounting, `sysinfo`-based sampler, 19 tests).
- `app/src/ai/blocklist/action_model/execute/shell_command.rs`: `activity_monitor` field, `begin_monitoring`/`sample_activity`, `LrcMonitoringGuard`, `lrc_activity_signals_supported`, snapshot `activity`/`forget` wiring, `action_result_future` `ctx` parameter, `action_result_for_*` field threading, `ActionResult::Snapshot` field.
- `app/src/ai/blocklist/action_model/execute.rs`: `pub(super) mod lrc_activity;`.
- `app/src/terminal/model/terminal_model.rs`: `ShellProcessInfo` struct, `shell_process_info` field/accessors, cleared on exit.
- `app/src/terminal/local_tty/terminal_manager.rs`: un-gated `pty.get_pid()` feeding `set_shell_process_info` (resolved onto the fork's `on_shell_determined` method shape).
- `crates/warp_features/src/lib.rs`: `FeatureFlag::LrcActivitySignal` variant only.

### Fork integration glue (handwritten, provider boundary)

- `app/Cargo.toml` + `app/src/lib.rs`: `lrc_activity_signal` cargo feature in `default`, bridged to `FeatureFlag::LrcActivitySignal` — the fork's retained-feature pattern replacing upstream's `DOGFOOD_FLAGS` entry.
- `crates/ai/src/agent/action_result/mod.rs`: `impl Display for LrcActivity` plus activity rendering in the four snapshot `Display` arms. Upstream serializes `activity` to the server agent over proto (`action_result/convert.rs`, absent here); the fork's agent-visible serialization for these results is the `Display` text that flows into the ACP follow-up prompt, so the signals are surfaced there. Without this glue the sampler data would have no live reader in the fork.

### Intentionally omitted paths (with reasons)

- Root `Cargo.toml`/`Cargo.lock` proto re-pin (`warp-proto-apis` → `2bd5df2e3`): the fork has no proto dependency (Warp server APIs removed).
- `app/src/ai/agent/api/convert_conversation.rs`, `convert_to.rs` (+ tests): absent — server API conversion removed at fork creation.
- `crates/ai/src/agent/action_result/convert.rs` and the proto round-trip tests in `mod_tests.rs`: absent/`warp_multi_agent_api` dependency does not exist in the fork.
- `app/src/ai/agent/conversation.rs` (`narrow_token_count`, saturating accumulation) and `crates/persistence/src/model.rs` (`u64::from` proto conversions, `to_proto_combined` widening): the anchors (`footer_model_token_usage`, `stream_finished::ConversationUsageMetadata`, `ModelTokenUsage::to_proto*`) were removed with the server streaming path; the fork's persisted/displayed counters are already `u32` with no proto boundary to widen.
- `crates/warp_tui/src/tool_call_labels_tests.rs`: crate absent.
- `lrc_activity_signals_supported()` platform negations (`!cfg!(windows) && !cfg!(wasm)`): constants on the macOS-only fork; reduced to the flag check.
- `sampler.rs` wasm `cfg_if` branch and `#[cfg(not(unix))]` fallbacks: removed per the macOS-only host policy (no wasm targets, no Windows host).
- `#[cfg(unix)]` attributes on `ShellProcessInfo::pty_leader_fd` and the sampler's unix fns: dropped; always true on the maintained host.

## Verification

- `cargo fmt -- --check`: clean.
- `cargo check -p warp --all-targets --message-format short`: pass.
- `cargo check --workspace --all-targets --message-format short`: pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: 156 passed.
- `cargo nextest run -p warp -E 'test(lrc_activity)'`: 19 passed (upstream test file ported verbatim).
- `cargo nextest run -p ai`: 57 passed; `cargo nextest run -p warp -E 'test(shell_command)'`: 2 passed.
- `cargo build -p warp --all-targets --message-format short`: pass (8 pre-existing dead-code warnings in untouched files; no new warnings from this port). `cargo clean` after the release push.
- Deletion-surface scans: no hits in touched files; remaining workspace hits are the pre-existing allowed set (warpui_core doc/log wording, retained SSH/remote-terminal platform detection in `remote_server`/`remote_command_executor`/`zsh_body.sh`, retained remote-path tests).
- `CARGO_PROFILE_DEV_DEBUG=0` used for check/test/build as in prior audits.
