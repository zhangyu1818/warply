# Fork Contract

This contract defines what must remain true after local development or upstream merges.

## Maintained Product Surface

The fork is a local-first terminal application:

- Terminal GUI and shell workflow.
- ACP-backed agent conversations.
- OpenAI-compatible Next Command.
- OpenAI-compatible Prompt Suggestions.
- Local settings and local persistence.

No maintained feature should require Warp login, Warp billing, Warp cloud objects, Warp-hosted agent services, or external telemetry.

## Required AI Behavior

### Agent

- Natural language input and `/agent` enter the existing AgentView shell.
- The request backend is ACP-only.
- ACP session config is driven by AI settings and ACP adapter config options.
- ACP events are represented as ACP-native state, not old Warp Agent server actions.
- Assistant text, reasoning, tool calls, tool updates, plans, permission requests, available commands, session info, current mode, and config options must retain protocol structure.
- Permission UI answers ACP permission requests. It must not reuse old Warp auto-approve semantics as if they were ACP semantics.

### Suggestions

- Next Command and Prompt Suggestions are independent from ACP Agent.
- They use OpenAI-compatible endpoint/model/API key settings.
- They must not call Warp-hosted suggestion APIs.

### Settings

- The AI page should expose ACP Agent settings and terminal suggestion settings.
- It should not expose account, privacy telemetry, billing, Teams, Warp Drive cloud, usage credits, cloud BYOK, cloud agent, or old Warp agent settings.

## Removed Systems

These were deliberately removed. If upstream changes touch them, the default decision is reject:

- `app/src/auth/`: account auth, anonymous id, SSO, access tokens, login UI.
- `app/src/billing/`: billing, usage, upgrade, referral/invite gates.
- `app/src/autoupdate/`: upstream update/changelog behavior not needed for this fork.
- `app/src/crash_reporting/` and Sentry scripts.
- `app/src/ai/agent_sdk/`: old Warp-hosted Agent SDK and harnesses.
- `app/src/ai/agent_management/`: cloud agent management UI.
- `app/src/ai/ambient_agents/`: ambient/scheduled/cloud agents.
- `crates/graphql/` and `crates/warp_graphql_schema/`: Warp cloud GraphQL client/schema.
- `crates/managed_secrets/` and `crates/managed_secrets_wasm/`.
- `crates/isolation_platform/`: hosted/cloud isolation infrastructure.
- `crates/onboarding/`: onboarding and marketing flows.
- `crates/voice_input/`: voice input/transcription.
- External telemetry/event-store code and app focus telemetry.

## Retained Systems

These are still in scope and should receive compatible upstream fixes:

- Terminal emulator, blocks, PTY, shell integration, session restoration, input editor, completions.
- Natural language detection.
- AgentView shell, local conversation history, conversation navigation, help shortcuts, code review panel, context chips, local file attachments.
- ACP implementation and ACP UI rendering.
- OpenAI-compatible terminal suggestions.
- Local MCP file-based configuration.
- Local persisted objects used by workflows, prompts, AI facts, MCP, and local conversation data.
- OS launch-at-login settings.
- SSH/remote terminal behavior that does not depend on Warp account auth.

## Legacy Naming Policy

Names alone are not merge evidence.

- `blocklist` is legacy naming for the AgentView path.
- `CloudObject` may still represent local persisted object data.
- `server_id` may still exist for local schema compatibility.
- `remote_server` is remote terminal support, not account login by itself.

Before deleting or restoring code with these names, inspect call sites and persistence usage.
