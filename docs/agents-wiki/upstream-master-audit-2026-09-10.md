# Upstream Master Audit 2026-09-10

Range under review: `5879ee2d0..upstream/master` (13 commits)

Previous audited upstream tip: `5879ee2d0 Scope managed secrets and harness credentials by team (#15862)`

Current upstream tip detected: `d732aa5fc Fail Sentry DIF uploads loudly after bounded retries (#15819)`

Total upstream commits in this incremental range: 13

Status: triage complete. Zero-port cycle. No commit in the range is applicable to this fork: seven are `.github/` CI dependency bumps or removed-crate lockfiles against workflow files and action references the fork does not carry, one is the upstream issue-triage bot taxonomy for upstream's own GitHub repository, and the five product commits live entirely in removed surfaces (`agent_sdk` harness/ambient-agent stack, server API credential classification, GraphQL/isolation-platform error plumbing, cloud shared-session steering, Sentry upload scripts) with no anchor symbols in retained trees.

## Per-Commit Triage

### `6db7fb734` — Update triage taxonomy with agent: primary types and agent:priority-high (#15868)

Decision: **not applicable**. Adds `agent:bug`/`agent:feature`/`agent:security`/`agent:documentation`/`agent:priority-high` label definitions to `.github/issue-triage/config.json` (absent in this fork) and one taxonomy bullet to `.agents/skills/triage-issue-local/SKILL.md`. The fork's copy of that skill is a fork-owned variant that already diverged (the `specializes_source: warpdotdev/oz-for-oss` frontmatter and the parent-skill install prerequisite were removed), and the labels themselves are applied by the external oz-for-oss triage workflow described in the upstream PR description, not by anything in this repository. No `agent:*` label exists anywhere in the fork. Same rule as the `.agents/skills/review-pr-local` treatments recorded in the 2026-08-12 and 2026-08-20 audits.

### `86bcf038f` — Classify credential dependency failures as Warp errors (#15236)

Decision: **reject** (removed surfaces, no anchors). Every touched path is deleted at the fork baseline or by contract: `app/src/ai/agent_sdk/**` (old Warp Agent SDK), `app/src/server/retry_strategies.rs` + `server_api/ai.rs` (fork has no `app/src/server/`), `crates/graphql/**` + `crates/warp_graphql_schema/**` (removed), `crates/isolation_platform/**` (removed). The `Cargo.lock` +1 line adds a dependency for `isolation_platform` only. Zero fork hits for `retry_strategies|RetryStrategy` under `app/src`.

### `8865d29b9` — Queue 3p follow-ups until setup done, never create a native conversation for CLI-harness ambient tasks (#15801)

Decision: **reject** (removed harness/ambient stack, no anchors). The feature is built on the removed `agent_sdk/driver/harness/**` (Claude Code/Codex/Gemini harnesses), `app/src/ai/ambient_agents/task.rs`, `app/src/ai/agent/api.rs`, `app/src/server/server_api/harness_support.rs`, `app/src/ai/blocklist/local_agent_task_sync_model.rs`, `app/src/ai/blocklist/controller/shared_session.rs`, and `app/src/terminal/local_tty/terminal_view_adaptor.rs` (the fork's `local_tty/` tree has no adaptor file). The new `blocklist/pending_cli_harness_prompt_queue.rs` and its registrations in `blocklist/mod.rs`, `lib.rs`, `pane_group/mod_tests.rs`, `test_util/terminal.rs` all key off `LocalAgentTaskSyncModel`/`PendingCliHarnessPromptQueue`/`OrchestrationEventStreamer`, none of which exist in this fork (`rg 'LocalAgentTaskSyncModel|pending_cli_harness|CliHarnessPrompt|AmbientAgentTaskId' app/src` → zero hits). The fork's third-party CLI agents run through `cli_agent_sessions` in the terminal directly with no ambient-task or native-conversation creation path, so the bug being fixed cannot occur here. `app/src/tui_test_support.rs` is also absent.

### `ff9dbf463` — Allow sharer-attributed shared-session control actions (steering interrupt) (#15896)

Decision: **reject** (cloud shared-session control semantics). The only touched file is `app/src/terminal/local_tty/terminal_view_adaptor.rs` (absent in the fork), and the change threads `shared_session_presence_manager().sharer_id()`/`viewer_role()` through the server conversation steering API — exactly the shared-session viewer action-sync / remote action mirroring surface the fork contract removes. No `sharer|steering|shared_session` anchors exist under the fork's `app/src/terminal/`.

### `d732aa5fc` — Fail Sentry DIF uploads loudly after bounded retries (#15819)

Decision: **not applicable**. `script/sentry_upload_dif.sh` does not exist in this fork; Sentry release/upload infrastructure is removed.

### CI dependency bumps and removed-crate lockfiles

All **not applicable** — `.github/` CI is fork-owned and none of the bumped actions are referenced by the fork's workflow files (`rg 'setup-go|oz-agent-action|build-push-action|install-action|login-action' .github/workflows/` → zero hits; the fork's `create_release.yml` uses hash-pinned fork-specific actions):

- `a4e8122ca` Bump actions/setup-go 6.5.0 → 7.0.0 (`create_release.yml`).
- `fb6ae52ed` Bump warpdotdev/oz-agent-action 1.0.12 → 1.0.26 (`changelog_draft.yml`, `create_release.yml`; both refs absent).
- `d534e008e` Bump warpdotdev/oz-agent-action 1.0.26 → 1.0.32 (same absent files).
- `3dfad452f` Bump docker/build-push-action 6.19.2 → 7.3.0 (`publish-agent-dev-image.yml`, absent).
- `ccd263199` Bump taiki-e/install-action 2.82.7 → 2.85.13 (`ci.yml`; the fork's `ci.yml` does not use it).
- `82d33160e` Bump docker/login-action 3.7.0 → 4.6.0 (`publish-agent-dev-image.yml`, absent).
- `4b4833833` Bump browserslist in `/crates/warp_graphql_schema/yarn.lock` (crate removed at baseline).

## Verification

No code was ported in this cycle; the only tree changes are these audit docs. The unchanged tree was re-verified directly:

- `cargo fmt -- --check` — pass (workspace).
- `cargo check -p warp --all-targets --message-format short` — pass (`CARGO_PROFILE_DEV_DEBUG=0`, 1m 46s; pre-existing unused-test-helper warnings only).
- `cargo build -p warp --all-targets --message-format short` — pass (`CARGO_PROFILE_DEV_DEBUG=0`, 1m 18s), followed by `cargo clean`.
- Deleted-surface scans re-run with only the same allowed hits recorded in the 2026-09-08 audit (weak-handle `upgrade()` calls, doc-comment wording, the pre-existing `Warp Drive` doc-comment in `crates/warp_util/src/sync.rs`, retained SSH `ForwardX11=no`/ConPTY explanatory strings, and `#[cfg(windows)]`-gated `warp_util` tests); MCP/skills pattern — zero hits.

## Notes

- Fifth zero-port cycle after 2026-08-09, 2026-08-29, 2026-09-07, and 2026-09-08. The range continues the REV-2383/multi-team + KTLO pattern: upstream repo automation, CI bumps, and fixes confined to the removed agent-sdk harness / ambient-agent / server-API / shared-session / Sentry stacks.
- Deferred ports unchanged: `9921300b7` (Ctrl-C harness cancel, waiting on upstream local-keystroke wiring) and the mermaid toggle from `#10431` architecture work.
