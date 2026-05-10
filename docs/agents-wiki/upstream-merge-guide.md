# Upstream Merge Guide

Use this guide when pulling commits from original Warp into this fork.

## First Question

For every upstream commit, ask:

```text
Does this improve a retained local terminal/ACP/suggestions feature, or does it restore a removed Warp cloud product feature?
```

That answer decides the merge path.

## Decision Process

1. Inspect upstream changed paths.
2. Classify each path as accept, adapt, or reject.
3. Resolve conflicts according to that classification.
4. Run focused tests for touched areas.
5. Run workspace checks before considering the merge complete.

Prefer small upstream batches. Large upstream merges are likely to reintroduce deleted cloud modules.

## Accept

Accept or cherry-pick directly when the change is limited to retained generic behavior:

- Terminal rendering, blocks, PTY, shell integration, session restoration.
- Input editor, completions, command parsing, natural language detection.
- GPUI/Warp UI framework and platform windowing fixes.
- Local settings infrastructure.
- Local MCP file parsing.
- Dependency security updates that do not re-add removed cloud/API/reporting crates.
- Remote terminal fixes that do not require Warp account tokens.

## Adapt

Port the behavior manually when upstream touches retained but forked areas:

| Area | Adaptation rule |
| --- | --- |
| `app/src/ai/blocklist/` | Keep AgentView shell/UI improvements, but route backend behavior through ACP. |
| `app/src/ai/agent/` and conversation history | Preserve local ACP transcript/history semantics. Do not restore server transcript APIs. |
| `app/src/ai/acp/` | Preserve ACP protocol-native state and event mapping. |
| `app/src/ai/predict/` | Keep context/UI improvements, but use OpenAI-compatible suggestions provider. |
| `app/src/ai/terminal_suggestions/` | Keep OpenAI-compatible endpoint/model/key flow. |
| `app/src/settings/ai.rs` and `app/src/settings_view/ai_page.rs` | Keep only ACP and terminal suggestions settings. |
| `app/src/terminal/input*` | Keep NLD and `/agent` behavior entering ACP AgentView. |
| `app/src/cloud_object/` and `crates/warp_server_client/` | Inspect whether the data supports local workflows/prompts/facts/history before accepting. |
| `crates/persistence/` | Accept migrations/schema only for retained local data. |
| `crates/remote_server/` | Keep local/SSH terminal behavior; reject account-auth token requirements. |

## Reject

Reject or strip these upstream changes unless they can be reduced to a retained local utility:

- `app/src/auth/` or any access-token acquisition path.
- Billing, credits, usage, referrals, upgrade, invite, Teams, workspace discovery.
- Cloud Warp Drive sharing/sync/import/export UI.
- `app/src/ai/agent_sdk/`, `agent_management`, `ambient_agents`, scheduled/cloud agents.
- Old server-backed AI APIs, Warp-hosted suggestion APIs, cloud orchestration, handoff, remote-control cloud controls.
- `crates/graphql/`, `crates/warp_graphql_schema/`, cloud GraphQL generated queries/mutations.
- Managed secrets, cloud environments, hosted isolation.
- Voice input/transcription.
- Onboarding/marketing UI and assets.
- Telemetry, crash reporting, Sentry release/upload, event stores.

## Conflict Examples

### Upstream edits `app/src/ai/blocklist/agent_view`

Port layout, keyboard, scrolling, accessibility, or rendering fixes. Do not restore cloud agent controls, usage banners, remote-control cloud controls, or old model/profile selectors.

### Upstream edits old Warp Agent SDK files

Ignore the commit unless it contains a protocol/UI idea that should be reimplemented in `app/src/ai/acp/` or ACP rendering. Do not restore `app/src/ai/agent_sdk/`.

### Upstream edits prediction APIs

Keep terminal context collection or UI placement improvements. Replace network calls with the OpenAI-compatible terminal suggestions client.

### Upstream adds persistence migrations

Accept if the migration supports retained local conversations, workflows, prompts, AI facts, MCP, terminal sessions, or UI state.

Reject if it supports users, teams, billing, cloud refresh, cloud sharing, hosted agents, managed secrets, or usage tracking.

### Upstream changes `Cargo.toml` or `Cargo.lock`

Regenerate dependency state from retained manifests. Do not accept reintroduced GraphQL, Sentry, telemetry, managed-secret, onboarding, old-agent-SDK, or voice-input crates unless a retained local feature explicitly needs them.

## Post-Merge Audit

Run targeted searches and inspect every hit:

```bash
rg -n "access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment" app crates script Cargo.toml
```

Allowed hits should be documentation, local logs, legacy local naming, or retained remote-terminal/local-object code. Product code that restores removed systems should be removed again.

## Verification

Use focused checks for the touched area, then:

```bash
cargo check -p warp --all-targets --message-format short
cargo check --workspace --all-targets --message-format short
cargo fmt -- --check
cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'
```
