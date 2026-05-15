# Warp Local ACP Fork Guide

This file is the repository-level entry point for agents working on this fork. It documents the fork delta from upstream Warp and the rules for merging future upstream commits.

## Baseline

Fork baseline:

```text
19659d12 refactor: create local ACP-only Warp fork
```

Compare future upstream merges against that commit when deciding whether a change should be accepted, adapted, or rejected.

## Fork Delta

This fork keeps Warp as a local terminal GUI and changes the AI/product surface:

- Warp-hosted Agent is replaced by ACP.
- `/agent` and natural-language terminal input enter the existing AgentView shell, but the backend is ACP-only.
- ACP events are rendered in Warp UI as protocol-native assistant text, reasoning, tool calls, tool updates, plans, permissions, commands, session info, modes, and config options.
- Next Command and Prompt Suggestions use user-configured OpenAI-compatible endpoints.
- AI settings contain only ACP and terminal suggestions configuration.
- The app runs without Warp login or Warp access tokens.

## Deleted Product Areas

Do not restore these systems when merging upstream:

- Account auth, anonymous user creation, access token retrieval, SSO, paste-token login.
- Billing, usage credits, referrals, upgrade, invite, Teams, workspace discovery.
- Cloud Warp Drive sharing/sync/import/export UI.
- Old Warp Agent SDK, cloud agents, ambient agents, scheduled agents, orchestration, handoff, and cloud remote-control semantics.
- Warp cloud GraphQL client/schema, managed secrets, hosted isolation/cloud environments.
- Voice input and hosted transcription.
- Onboarding, cloud marketing surfaces, Oz/cloud-agent assets.
- External telemetry, crash reporting, Sentry release/upload scripts, app focus telemetry, event queues.
- Old Warp AI `/model` and `/profile` selector flows.

## Retained Local Areas

These remain part of the fork and should receive applicable upstream fixes:

- Terminal emulator, blocks, shell integration, PTY/session handling, input editor, completions, and Warpify for subshell/SSH sessions.
- Natural language detection and input classification.
- AgentView shell, conversation navigation, help shortcuts, code review side panel, context chips, local attachments.
- ACP implementation under `app/src/ai/acp/`.
- OpenAI-compatible Next Command and Prompt Suggestions under `app/src/ai/terminal_suggestions/` and `app/src/ai/predict/terminal_*`.
- ACP tool-call rendering. MCP server configuration belongs to the ACP agent process, not the Warp app.
- Local persistence for conversations, workflows, prompts, AI facts, and retained object data.
- OS launch-at-login settings.
- SSH/remote terminal behavior that does not require Warp account auth.

## Upstream Merge Decision Table

| Upstream change area | Decision |
| --- | --- |
| Terminal, shell integration, PTY, blocks, editor, completions | Usually accept or cherry-pick directly. |
| GPUI/Warp UI framework, platform windowing, rendering | Usually accept if it does not depend on removed product surfaces. |
| ACP, AgentView, `blocklist`, conversation history, AI settings | Port manually and preserve ACP-only backend behavior. |
| Next Command, Prompt Suggestions, prediction APIs | Port UI/context improvements only; keep OpenAI-compatible provider. |
| Persistence and migrations | Accept only if needed by retained local data. Reject account/team/billing/cloud-only migrations. |
| `CloudObject`, `server_id`, `local_object_model` naming | Inspect data flow before deciding. Some names remain for local schema compatibility. |
| Auth, billing, Teams, Warp Drive cloud, GraphQL, managed secrets, telemetry, crash reporting, onboarding | Reject unless the change can be reduced to a retained local utility without restoring the product area. |
| Cargo dependencies | Keep removals unless a retained local feature needs the crate. Be skeptical of reintroduced cloud/API/reporting crates. |

## Required Reading For Merge Work

- `llms.txt`
- `docs/agents-wiki/README.md`
- `docs/agents-wiki/fork-contract.md`
- `docs/agents-wiki/upstream-merge-guide.md`
- `docs/agents-wiki/change-map.md`

## Verification

For merge or code changes, run focused checks first, then expand as needed:

```bash
cargo check -p warp --all-targets --message-format short
cargo check --workspace --all-targets --message-format short
cargo fmt -- --check
cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'
```
