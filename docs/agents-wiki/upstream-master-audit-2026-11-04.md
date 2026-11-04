# Upstream Master Audit — 2026-11-04

Incremental audit for `eafc6d18d..upstream/master` (`1eca3e4ee`), 21 commits. The previous audit is `upstream-master-audit-2026-10-28.md`.

## Summary

Two ports: the Features-page Tab-key behavior detection fix (`3c5115418`, verbatim — `refresh_tab_behavior_state` compared an uppercased keystroke against `"Tab"` so the dropdown always resolved to `UserDefined`) and the pending-command bootstrap-precmd guard from the environment-checkout hanging fix (`5a55a2aca` — `Input::execute_pending_command` now returns early until `is_bootstrapping_precmd_done()`, with the upstream regression test adapted to this fork's fixtures). Nine commits are not applicable (upstream-only CI/release/docker infrastructure or fork-absent files with no anchors), and ten are rejected (cloud-agent Linux PATH restoration, bundled skills ×2, agent-sdk harness/managed-secrets/GraphQL, server-api preview retention, crates/mcp, shared-session reconnect, out-of-credits billing gate, agent-sdk ACP-harness sandbox resume/steering).

## Commit-by-Commit

### `f883a848f` Preserve Linux runtime commands after shell profiles reset PATH (#16650) — rejected

Adds post-profile PATH restoration for `WARP_CLOUD_AGENT_FALLBACK_BIN` to the retained `bash_body.sh`/`zsh_body.sh`/`fish.sh` bootstrap assets, plus an integration test. Every shell hunk is gated on that environment variable, which is only produced by the warp-agent-docker container companion for bundled cloud-agent commands (ffmpeg/chromium sidecars) on direct non-root Linux containers. This fork removed cloud agents entirely and has no producer of the variable, so the change is dead cloud-agent plumbing inside retained files rather than a generic shell-integration fix; the dedup/prepend logic is inseparable from the variable check. No fork path executes a terminal inside a cloud-agent container.

### `23adf48cf` Update factory authoring skill to recommend automation scope filters (#16670) — rejected

Edits `resources/bundled/skills/factory-files/SKILL.md`. Bundled skills are a removed surface; `resources/bundled/skills/` does not exist here.

### `365e3c8b7` ci: mint GitHub App tokens instead of warpmachineuser PATs (#16655) — not applicable

Touches `cut_new_release_candidate.yml`, `cut_new_releases.yml`, `notify-docs-settings-changed.yml` — none exist in this fork. The fork's `.github/workflows/` contains only fork-owned `ci.yml` and `create_release.yml`.

### `6aeacb6ad` Exclude warp acp-bridge from TUI detection (#16675) — not applicable

Adds an `all_parsed_commands`-based check so a `warp acp-bridge` invocation is not treated as a TUI CLI agent run. The entire guard is gated on `matches!(self, CLIAgent::WarpTui)`, and this fork's `CLIAgent` enum has no `WarpTui` variant (no `warp_tui` crate; detection is the fork's singular `command_prefix` + `detect()` second-prefix idiom). The fork never detects `warp` invocations as a CLI agent, so there is no anchor and no behavior to fix. `all_parsed_commands` itself already exists in `crates/warp_completer`.

### `513a94bc0` bump trunk-io/analytics-uploader 2.1.5 → 2.1.6 (#16562) — not applicable

Upstream `ci.yml` only. The fork's `ci.yml` is a fork-owned rewrite with no trunk-io action.

### `f7f136ed8` bump taiki-e/install-action 2.87.0 → 2.87.26 (#16575) — not applicable

Same as above; the fork's `ci.yml` has no taiki-e action.

### `3c5115418` Fix Tab key behavior always detected as user defined (#16529) — accepted, ported verbatim

`FeaturesPageView::refresh_tab_behavior_state` uppercased the displayed keystroke before comparing it to `TAB_KEYSTROKE_STR` (`"Tab"`), so the comparison never matched and the "Tab key behavior" dropdown always resolved to `TabBehavior::UserDefined` after a page refresh, hiding the hint line. The fix drops the two `.to_uppercase()` calls, matching what `set_tab_behavior_setting` already does. Purely local settings-page state detection in `app/src/settings_view/features_page.rs` with no service dependency. Applied as the exact upstream patch (`git apply --3way`); the surrounding function in this fork was identical to upstream's pre-patch code, so the port is byte-for-byte the two upstream hunks. No test exists upstream or in the fork for this state function.

### `5d26d9bb6` bump warpdotdev/oz-agent-action 1.0.32 → 1.0.36 (#16518) — not applicable

Touches `changelog_draft.yml` (absent) and upstream `create_release.yml` (fork-owned rewrite with no oz-agent action).

### `811b70b8b` Align agent dev image Go linter with server CI (#16672) — not applicable

`docker/agent-dev/Dockerfile` only; the fork has no `docker/` directory.

### `e7bb41fc4` Release workflow: federate to GCP and Octo STS instead of stored keys (#16674) — not applicable

Upstream release infrastructure (`prepare_environment`, `.github/chainguard/`, `cut_new_*`, upstream `create_release.yml`). This fork's `prepare_environment` action is a fork-owned macOS build-environment rewrite (LFS, rust-cache, cargo-binstall, protoc) with no GCP/Octo STS or stored keys, and its `create_release.yml` is the Warply Sparkle release workflow.

### `5a55a2aca` Fix environment checkout hanging during shell bootstrap (#16661) — accepted (terminal hunks), ported with adaptations

Upstream motivation: the agent-sdk environment checkout executes a command during the window between the `Bootstrapped` hook and the final bootstrap precmd; that precmd replaces the active block, so a pending command executed inside the window could hang. The generic fix is retained terminal behavior: `Input::execute_pending_command` now returns early while `!block_list().is_bootstrapping_precmd_done()`, so anything routed through `execute_command_or_set_pending` (launch-config tab commands, queued setup commands) waits for the final bootstrap precmd and then flushes on the existing `ModelEvent::BootstrapPrecmdDone` → `TerminalView::execute_pending_command` path. This protects the same window in this fork, where `TerminalView::execute_command_or_set_pending` and the `BootstrapPrecmdDone` flush already exist.

Ported hunks and intentional divergences from the upstream patch:

- `app/src/terminal/input.rs` guard hunk: applied verbatim via `git apply --3way`; the fork's `execute_pending_command` was byte-identical to upstream's pre-patch code.
- Regression test `pending_command_waits_for_final_bootstrap_precmd`: upstream lives in `input_tests.rs`; this fork's mounted test file is `input_test.rs` (`#[path]` mount), so the test and the `subscribe_to_execute_command_events` helper are copied there, next to the handoff fixtures as upstream places them. Fixture adaptations only:
  - `model.command_finished(CommandFinishedValue { completion_metadata, session_id })` → `CommandFinishedValue { session_id: Some(session_id.as_u64()), ..Default::default() }` — the fork's pre-rework `CommandFinishedValue` has `exit_code`/`next_block_id`/`session_id` fields, not upstream's `completion_metadata`.
  - `model.precmd_with_completion_metadata(PrecmdValue { completion_metadata, prompt_metadata })` → `model.precmd(PrecmdValue { session_id: Some(session_id.as_u64()), ..Default::default() })` — the fork has no `precmd_with_completion_metadata` and its `PrecmdValue` carries the legacy shell-integration fields.
  - Imports: `CommandFinishedValue` added to the existing `model::ansi` import; `crate::terminal::model_events::ModelEvent` imported for the dispatcher emission, which is used verbatim (`terminal.model_event_dispatcher().update(ctx, |_, ctx| ctx.emit(ModelEvent::BootstrapPrecmdDone))` — the idiom already used by `ModelEventDispatcher::request_remote_server_block`).
  - The helper itself is upstream verbatim over fork types (`TerminalViewEvent::ExecuteCommand` → `(event.command, event.source)`).
- Verified as a true regression guard: with the guard hunk temporarily stashed, the test fails at `assert!(input.has_pending_command())` because the pending command executes immediately during the Bootstrapped→PostBootstrapPrecmd window; with the guard restored it passes.
- `app/src/ai/agent_sdk/driver/environment.rs` and `environment_tests.rs` hunks: not applicable — the fork has no `app/src/ai/agent_sdk/`; the environment-checkout producer of these pending commands was removed with the agent SDK. The input-layer fix stands alone for the fork's retained pending-command paths.

### `01614bfac` Prefer WARP_WORKSPACE_ROOT in factory-deferred-repositories skill (#16693) — rejected

Edits `resources/bundled/skills/factory-deferred-repositories/SKILL.md`; bundled skills are removed and the directory does not exist here.

### `b17a2ed02` Support managed LLM provider API keys in the harness driver (#16470) — rejected

Managed-secrets envelope/manager/secret-value changes plus `agent_sdk` harness (codex/pi) provider-key plumbing, `crates/graphql` task-secrets, `warp_graphql_schema`, `auth_secret_types.rs`, and `local_harness_launch.rs`. Every touched path is a removed surface in this fork (verified absent); there is no anchor to port onto, and the feature's core is the removed managed-secrets + server secret store.

### `89de3f5c9` feat: preview url idle retention (#16649) — rejected

New `agent_sdk/driver/preview_retention.rs` plus `server/server_api/ai.rs` changes to keep preview URLs of finished runs alive. Lives entirely on the removed agent SDK and server API surfaces (verified absent); the fork's ACP agents own their own preview/process lifecycle.

### `5acc8e7e4` Fix premature MCP protocol-version header during initialize (#16698) — not applicable

`crates/mcp/src/runtime.rs` only. The fork removed app-managed MCP and has no `crates/mcp`; MCP protocol handling belongs to the ACP agent process here.

### `d57d9760d` Reset reconnects with no progress if there are no new events to send (#16696) — rejected

`app/src/terminal/shared_session/sharer/network.rs` — cloud shared-session sharer reconnect accounting. The fork has no `shared_session` directory; session sharing was removed.

### `ffc5dc884` fix: let a configured custom inference endpoint pass the out-of-credits gate (#16493) — not applicable

`app/src/ai/request_usage_model.rs` gates AI requests on Warp-account credits; both the file and the billing/credits gate are removed in this fork. Terminal suggestions go through their own OpenAI-compatible request path with no credits gate, and ACP model choice is user-configured.

### `0eca80f4c` Send the warp-channel-config token as a URL-scoped git header (#16701) — not applicable

`prepare_environment` (fork-owned rewrite without channel-config) and `script/install_channel_config` (absent). No channel-config generator exists here.

### `77a4fc222` Read the Azure signing tenant and client IDs from repository variables (#16712) — not applicable

Upstream `create_release.yml` Azure signing variables. The fork's release workflow has no Azure signing path.

### `3dca8f995` Fix Linux signing passphrase handling (#16713) — not applicable

Upstream `create_release.yml` Linux signing. macOS-only fork; the release workflow has no Linux signing.

### `1eca3e4ee` ACP: resume the adapter session on new sandboxes, and steer mid-turn follow-ups (#16720) — rejected

Adds `agent_sdk/driver/harness/acp/resume.rs` and threads sandbox-recreation session resume plus mid-turn steering through the upstream agent-SDK ACP harness (`turn_driver`, `connection`, `mapping`, `protocol`, `test_support`). This fork's ACP client is `app/src/ai/acp/`, a separate implementation; `app/src/ai/agent_sdk/` does not exist here, and the sandbox-recreation and steering semantics are the cloud-agent orchestration runtime this fork removed. No fork anchor exists to port onto, and the fork's ACP sessions are local process sessions that are not recreated by a cloud sandbox manager.

## Verification

- `cargo check -p warp --all-targets --message-format short` — passed (SDKROOT pointed at the Xcode MacOSX SDK per the CLT tapi prerequisite); remaining warnings are pre-existing dead-code lints, none in the touched files.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo fmt -- --check` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 tests passed, 0 failed.
- `cargo nextest run -p warp -E 'test(pending_command_waits_for_final_bootstrap_precmd)'` — passes with the guard; fails against the temporarily stashed guard at the first pending assertion (regression guard verified, then guard restored).
- `cargo build -p warp --all-targets --message-format short` — passed; `cargo clean` run after the push.
- Restored-surface scans over the diff against `main` (three files): zero hits on all three scan patterns. Repo-wide scans return only the pre-existing allowed hits (documentation, tests, retained SSH/remote-terminal paths, shell-integration comments), unchanged from the 2026-10-28 audit baseline.

## Tag

`v2026.11.04` on `main` (merge commit of `merge/upstream-2026-11-04`).
