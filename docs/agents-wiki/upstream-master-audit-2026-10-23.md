# Upstream Master Audit — 2026-10-23

Incremental audit for `f8a9c719f..upstream/master` (`1e6c3ae46`), 7 commits. The previous audit is `upstream-master-audit-2026-10-22.md`.

## Summary

One accepted port: the CVE-2026-111943 bundled TLS certificate bump (`edf27fd58`, lockfile-only: rustls-webpki 0.103.15 → 0.103.16, webpki-root-certs 1.0.1 → 1.0.10, webpki-roots → 1.0.10). One rejected feature: the independent Interrupt / Steer prompting stack (`07c5e048d`) whose runtime is the old Warp Agent request-piggyback semantics with no ACP anchor. Five commits are not applicable: two `agent_sdk` stacks (git mirror cache, pre-warmed sandbox harness support), the pending-sharer local execution fix (session sharing removed), the `crates/mcp` legacy protocol fallback (crate absent), and the ACP harness MCP tool-call projection (harness + `warp_multi_agent_api` + `agent/api` conversion absent).

## Commit-by-Commit

### `a36b76b92` Speed up Git mirror refresh and cached checkouts; mirror only branches and tags (#16552) — not applicable

Rewrites `app/src/ai/agent_sdk/driver/environment_checkout.rs` (bare clone of branches+tags instead of `--mirror`, concurrent default-branch lookup, `checkout.workers=0`) plus its tests. The changelog entry itself scopes it to "Warp cloud agents". Both paths live under the removed `agent_sdk` driver; the fork has no `environment_checkout`, no git mirror cache, and zero anchors for the touched symbols.

### `07c5e048d` [APP-6429] Add independent Interrupt / Steer prompting (#16595) — rejected

Adds `PromptSubmissionMode::Steer`: while a local desktop Warp Agent conversation is mid-turn, unqueued prompts are filed as `QueuedQueryOrigin::LocalSteering` rows and piggyback a `UserQuery` onto the agent's next request (tool-result follow-up or event injection) instead of interrupting. The core runtime is `BlocklistAIController::steer_head_prompt_for_request` attaching the row's input to the in-flight Warp Agent request — the old backend request semantics this fork replaced with ACP. The fork's controller has no `steer_head_prompt_for_request`, `is_steering`, `QueuedPromptDeliveryMode`, `is_dispatch_blocked`, `dispatch_queued_warp_agent_prompt`, `OrchestrationEventService`, `conversation.driver()`, `startup_queue` module, `resolve_ai_query_routing`, or `maybe_queue_input_during_cloud_setup`; `send_follow_up_for_conversation` deliberately returns early while an ACP session is active (`AcpAgentModel::has_active_session_for_conversation`), so there is no request the row could ride on. Porting the behavior would require inventing mid-turn prompt injection in the ACP client — a from-scratch rewrite upstream does not implement.

Recorded per-hunk classification of the mixed paths:

- `app/src/ai/blocklist/queued_query.rs` — `LocalSteering` origin, `RequestRow`, `next_row_for_request`, and `is_primary_injected_followup` build on `SharedSessionPrompt` rows, `BaseUserQuery`, `warp_multi_agent_api::AgentType`, `QueuedPromptDeliveryMode`, and `is_steering`, none of which exist in the fork's de-clouded queue model. `effective_prompt_submission_mode` is a ternary-mode refactor of the fork's existing binary toggle logic (behavior-identical while only Interrupt/Queue exist). The `is_lrc_auto_queue_active` `== Interrupt` → `!= Queue` change is a semantic no-op with the fork's two variants.
- `app/src/settings/ai.rs` — the `Steer` variant, `display_name`, palette description, and `is_available` gate: a `Steer` setting has no ACP-path runtime owner in this fork, so exposing it would be a no-op legacy setting (contract: reject settings whose ownership cannot be mapped to a live ACP or local path).
- `app/src/settings_view/warp_agent_page.rs`, `app/src/settings_view/mod.rs`, `app/src/workspace/view.rs` — the fork has no `warp_agent_page.rs`; its equivalent widget lives in `ai_page.rs` with its own flag set. The LRC-hidden-when-Queue binding restructure only changes behavior for the third mode.
- `app/src/terminal/input.rs` — `maybe_steer_input_for_in_progress_conversation` depends on `resolve_ai_query_routing`, `ambient_agent_task_id`, and `emit_input_buffer_submitted_telemetry` (all fork-absent) and files rows no consumer can fire.
- `app/src/terminal/view/queued_prompts_panel.rs` — the row-suffix refactor renders the steering suffix; its `QueuedQueryOrigin` match also covers fork-absent variants (`LocalSteering`, `PendingLrcAutoQueue`, `InitialCloudMode`, `SharedSessionInjection`), and the fork's LRC-suffix rendering already exists in its own shape.
- `app/src/ai/blocklist/controller.rs`, `controller/startup_queue.rs` — steering dispatch/nudge, `conversation_accepts_local_steering`, and the startup-queue firing restructure: fork-absent symbols throughout; the fork has no `startup_queue.rs`.
- `app/Cargo.toml`, `app/src/features.rs`, `crates/warp_features/src/lib.rs`, `app/src/server/telemetry/events.rs` — `agent_prompt_steering` cargo feature, `FeatureFlag::AgentPromptSteering`, and the steering telemetry event gate a runtime that does not exist here; the telemetry events file is a removed surface.

### `3cee055a9` Allow pending session sharers to execute commands locally (#16544) — not applicable

One-line guard change in `app/src/terminal/input.rs` (`is_sharer_or_viewer() && !is_share_pending()`) so a sharer executes locally while a session share is pending. Session sharing is a removed surface: the fork has no `shared_session_status`, `is_sharer_or_viewer`, or `is_share_pending` anywhere in `app/src/terminal`. The motivating scenario (local session-sharing server + tunneled remote cloud-agent sandbox) is cloud-agent infrastructure.

### `d1d6200bf` Fix legacy MCP protocol-version fallback for server-error responses (#16620) — not applicable

Fixes `crates/mcp/src/runtime.rs` preflight so HTTP 400/-32600 synthetic `server-error` responses reach the legacy `initialize` fallback. The fork has no `crates/mcp`; app-managed MCP was removed and MCP belongs to the ACP agent process. This is the follow-up to #16574, whose preflight version negotiation (`9b499e029`) was already rejected on 2026-10-21.

### `edf27fd58` [REMOTE-3614] Update bundled TLS certificate verification and roots (#16622) — accepted, ported

Lockfile-only security bump for CVE-2026-111943 (compromised eMudhra intermediate keys): `rustls-webpki` 0.103.15 → 0.103.16 rejects the compromised intermediates, and both WebPKI bundles move to 1.0.10 (`webpki-root-certs` and `webpki-roots`). No manifest or source change; native macOS verification is unchanged. Retained scope: the fork's rustls consumers (OpenAI-compatible suggestions, remote/SSH transport, updater) get the patched bundles.

Applied via `cargo update --precise` to upstream's exact target versions; the resulting lockfile entries match upstream byte-for-byte, including checksums (`rustls-webpki` `4443b1b6…`, `webpki-root-certs` `58ec80e7…`, `webpki-roots` `1c1c8e20…`). The only structural difference from the upstream diff: the fork's `webpki-roots` resolved from 1.0.7 (not upstream's 1.0.1) and the fork has no `webpki-roots 0.26.11`/`rustls-platform-verifier` compat-wrapper entries to re-point, so those upstream hunks have no counterpart.

### `692f75fd9` [REMOTE-3650] Support Claude and Codex in pre-warmed sandboxes (#16624) — not applicable

Extends the pre-warmed standby sandbox stack (rejected/N-A 2026-10-16 through 2026-10-22) with harness-aware preparation and assignment installation. Every path is `app/src/ai/agent_sdk/**` (driver, agent_setup, prewarmed, standby, execution_config), all fork-absent.

### `1e6c3ae46` Project ACP MCP tool calls with their tool name, server identity, and late input (#16615) — not applicable

Splits `mcp__server__tool` names into tool + `McpServerIdentity`, carries the identity onto `CallMcpTool`, and threads late tool-call input through `UpdateTaskMessage` in the ACP harness mapper (`agent_sdk/driver/harness/acp/mapping.rs`), plus `mcp_startup` server-identity collection and harness constructor threading. The harness transport stack was never ported (rejected 2026-10-09/10-10/10-20/10-22); the fork has no `agent_sdk`, no `warp_multi_agent_api` dependency, and no `app/src/ai/agent/api/` (`convert_to.rs`/`mcp_server_identity` absent — the two shared-file hunks only re-export that symbol for the harness). The fork's `app/src/ai/acp/mapping.rs` renders ACP tool calls as protocol-native local events with no MCP identity projection to fix.

## Verification

- `cargo check -p warp --all-targets --message-format short` — passed (SDKROOT pointed at the Xcode MacOSX SDK per the CLT tapi prerequisite); remaining warnings are pre-existing dead-code lints.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo fmt -- --check` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 tests passed, 0 failed.
- `cargo build -p warp --all-targets --message-format short` — passed; `cargo clean` run after the push.
- Restored-surface scans over the diff against `main` (Cargo.lock only, 14 lines): MCP/skills scan zero hits; account/billing/telemetry and platform scans return only the pre-existing allowed hits (documentation, tests, retained SSH/remote-terminal paths).

## Tag

`v2026.10.23` on `main` (merge commit of `merge/upstream-2026-10-23`).
