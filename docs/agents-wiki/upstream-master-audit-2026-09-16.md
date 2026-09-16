# Upstream Master Audit 2026-09-16

Range under review: `e3464f102..upstream/master` (8 commits)

Previous audited upstream tip: `e3464f102 Attach the ambient workload token to the built-in Factory MCP server (#15992)`

Current upstream tip detected: `0eefb7404 Skip the personal runs seed fetch for service-account principals (#16032)`

Total upstream commits in this incremental range: 8

Status: triage complete. **Zero-port audit** — every commit is rejected or not applicable. No fork code changes.

## Per-Commit Triage

### `acb96e6e8` — [APP-5720] Round-trip request metadata in the client (#15876)

Decision: **N/A (reject)**. Client-side plumbing for server-authored `Message.request_metadata` proto records from warp-proto-apis #377, including `*_cost_in_credits` billing fields and persistence back to the Warp server's task message list.

- `warp_multi_agent_api` git dep bump in `Cargo.toml`/`Cargo.lock`: dep is fork-absent.
- `crates/persistence/src/model.rs`: the hunks extend `ChargedUsageTotals` and `api::InferenceUsage`/`stream_finished` — the fork's model.rs (811 lines vs upstream 1847) has no `ChargedUsageTotals`, no proto `api` module, no usage/credits tracking at all.
- `app/src/ai/agent/api/convert_conversation.rs`, `convert_from.rs`, `conversation_yaml.rs`, `app/src/persistence/agent_tests.rs`, `crates/persistence/src/model_tests.rs`: fork-absent files.
- Shared-file hunks in `app/src/ai/agent/conversation.rs` (`request_charges: Option<stream_finished::RequestCharges>`) and `app/src/ai/blocklist/history_model.rs` (`warp_multi_agent_api::RequestCharges`, `ConversationUsageMetadata`, `TokenUsage`) lack every anchor symbol in the fork — verified by search.

### `5b8e6934e` — Pre-dismiss HOA onboarding for new users (#16010)

Decision: **reject**. New-user onboarding pre-dismissal wired into auth-complete handling. `app/src/workspace/one_time_modal_model.rs` is fork-absent; the `app/src/workspace/view.rs` hunks anchor on `AuthManager::handle(ctx)`/`set_user_onboarded`, which do not exist in the fork's view.rs (verified). Onboarding and account auth are removed areas with no local or ACP data flow.

### `0ea952d31` — Report only the repos that actually failed in parallel clone errors (#16006)

Decision: **N/A**. `app/src/ai/agent_sdk/driver/environment.rs` and `environment_tests.rs` are fork-absent (agent_sdk removed at baseline).

### `af7d930a5` — Queue follow-up injections for warp agent, using new steering mode (#15921)

Decision: **reject**. The feature is steering-mode queued-prompt delivery for Oz ambient-agent-driven conversations plus shared-session prompt injection. Core machinery is fork-absent:

- `app/src/ai/agent_sdk/driver.rs`/`driver_tests.rs`, `app/src/server/telemetry/events.rs`, `app/src/terminal/local_tty/terminal_view_adaptor.rs`, `app/src/terminal/view/ambient_agent/model.rs`, `crates/warp_tui/`, `app/src/ai/blocklist/controller/shared_session.rs`: all fork-absent.
- No `session_sharing_protocol` crate, no `ParticipantId`/`AgentAttachment` types in the fork.
- Retained-file additions all serve the removed path: `queued_query.rs` gains `QueuedQueryOrigin::SharedSessionInjection` + `QueuedQueryKind::SharedSessionPrompt` + `QueuedPromptDeliveryMode` (steering only set by `bind_native_prompt_conversation` for Oz conversations); `controller.rs` gains `can_dispatch_queued_warp_agent_prompt`/`steer_head_prompt_for_request`; the new `controller/startup_queue.rs` hosts `dispatch_queued_warp_agent_prompt`/`route_native_startup_injection`.
- `queued_prompts_panel.rs` hunks extend `cloud_setup_in_progress`/`DispatchStateChanged` handling — the fork's panel has no `InitialCloudMode` variant or `DispatchStateChanged` event (fork's `QueuedQueryOrigin` enum verified: QueueSlashCommand/AutoQueueToggle/LrcAutoQueue/CompactAndSlashCommand/ForkAndCompactSlashCommand only).
- `terminal/input.rs` "Send now" routing targets `shared_session_prompt()` rows only; `pending_user_query.rs` hunks are `FeatureFlag::QueuedPromptsV2`-gated logging — flag fork-absent.
- Upstream keeps local Queueing mode (`/queue`, auto-queue, LRC auto-queue — the fork's entire retained surface) semantics unchanged, so there is no separable local fix.

### `5b0d0088d` — Adopt speed-first agent validation guidance (#16025)

Decision: **reject**. Upstream-owned process guidance (`AGENTS.md`, `.agents/skills/promote-feature|remove-feature-flag|rust-unit-tests|tui-testing`). The fork's `AGENTS.md` (416 diff lines vs upstream) is the fork contract and merge discipline with its own mandated verification sequence; the retained skill copies are fork-diverged (33–178 diff lines) and `tui-testing` is absent. Upstream workflow/process docs are rejected by default in favor of fork-owned `docs/agents-wiki/` records.

### `102144753` — [REMOTE-3187] Forward harness shutdown exit codes (#16008)

Decision: **N/A**. `app/src/ai/agent_sdk/harness_support.rs`, `app/src/server/server_api/harness_support*`, `agent_sdk/driver/*_tests.rs` are fork-absent; the fork's retained `crates/warp_cli` is the minimal completions/config/json_filter lib with no `harness_support.rs` and no bins.

### `7b7f4f7c2` — Turn panel UI for per-request usage metadata (APP-5720) (#15931)

Decision: **N/A (reject)**. Display side of the APP-5720 stack; the new `app/src/ai/agent/request_metadata.rs` self-describes as "the client's read-only view of the server-authored `Message.RequestMetadata` records" and imports `warp_multi_agent_api` + `crate::persistence::model::ChargedUsageTotals` (fork-absent). Behind the `PricingTransparency` feature flag (fork-absent) with Credits/Dollars billing settings (removed area). ACP conversations produce no `RequestMetadata` records in this fork, so the panel would be dead code with no producer; the legacy-conversation fallback path also uses `ConversationUsageMetadata`/usage-pill machinery absent here. Depends on `acb96e6e8`, also rejected.

### `0eefb7404` — Skip the personal runs seed fetch for service-account principals (#16032)

Decision: **N/A**. Gates the `GET /api/v1/agent/runs` personal-runs seed request on `AuthStateProvider.is_service_account()` inside `fetch_ambient_agent_tasks_and_cloud_convo_metadata`. The fork's `app/src/ai/agent_conversations_model.rs` (234 lines) has no such function and no `AuthStateProvider`/`user_id`/ambient-fetch machinery — removed with auth, ambient agents, and the server API.

## Verification

Zero ports; verification confirms the released tree is healthy. Run on `merge/upstream-2026-09-16` (= `main` + this audit doc, with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`):

- `cargo check -p warp --all-targets --message-format short` — pass (pre-existing warnings only).
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — pass (156/156).
- `cargo build -p warp --all-targets --message-format short` — pass.

Deleted-surface scan not applicable (no code changes); the standing scans from the 2026-09-15 audit remain valid for this tree.

`cargo clean` run after merge/tag/push.

## Notes

- The two APP-5720 commits (`acb96e6e8`, `7b7f4f7c2`) establish a new server-authored per-request usage/cost metadata pipeline. If a future upstream change routes equivalent metadata through a local or ACP-visible source, re-triage the stack as a unit; until then every anchor (`warp_multi_agent_api`, `ChargedUsageTotals`, `PricingTransparency`, credits settings) is fork-absent.
