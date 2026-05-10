# Change Map

This map explains the large fork baseline change at a path level.

## Replaced AI Architecture

| Path | Change | Merge rule |
| --- | --- | --- |
| `app/src/ai/acp/` | Added ACP backend, config options, event mapping, model, terminal/file capability plumbing, permission handling, thread state, and tests. | Preserve as the only agent backend. Port upstream ideas here only if they fit ACP. |
| `app/src/ai/blocklist/` | Retained AgentView shell and adapted it to ACP request flow and ACP-native output rendering. | Port generic UI fixes. Reject old cloud-agent controls and server-backed action execution. |
| `app/src/ai/agent/` | Simplified conversation/task data toward local ACP history and local transcript representation. | Preserve local persistence semantics. Do not restore server transcript APIs. |
| `app/src/ai/terminal_suggestions/` | Added OpenAI-compatible suggestions client/provider/tests. | Keep provider endpoint/model/key configurable. |
| `app/src/ai/predict/terminal_input_suggestions.rs` | Replaced hosted AI input suggestion request path for Next Command. | Port context improvements, not hosted API calls. |
| `app/src/ai/predict/terminal_prompt_suggestions.rs` | Added OpenAI-compatible prompt suggestions path. | Keep separate from ACP Agent. |
| `app/src/settings/ai.rs`, `app/src/settings_view/ai_page.rs` | AI settings now focus on ACP and terminal suggestions. | Reject account/billing/cloud-agent/privacy telemetry sections. |

## Removed Cloud And Platform Product Code

| Path or area | Removed purpose | Upstream merge decision |
| --- | --- | --- |
| `app/src/auth/` | Account auth, user identity, access tokens, login UI. | Reject. |
| `app/src/billing/` | Billing, usage, upgrade, referral gates. | Reject. |
| `app/src/autoupdate/` | App update/changelog infrastructure. | Reject unless this fork intentionally restores updater behavior. |
| `app/src/crash_reporting/` | Crash reporter/Sentry integration. | Reject. |
| `app/src/ai/agent_sdk/` | Old Warp Agent SDK, harnesses, cloud environment, scheduling, cloud tool execution. | Reject; reimplement useful behavior in ACP if needed. |
| `app/src/ai/agent_management/` | Cloud agent management UI. | Reject. |
| `app/src/ai/ambient_agents/` | Ambient/scheduled/cloud agents. | Reject. |
| `app/src/ai/cloud_agent_*`, `app/src/ai/cloud_environments/` | Cloud agent settings/environments. | Reject. |
| `crates/graphql/`, `crates/warp_graphql_schema/` | Warp cloud GraphQL API and schema. | Reject. |
| `crates/managed_secrets/`, `crates/managed_secrets_wasm/` | Cloud managed secrets. | Reject. |
| `crates/isolation_platform/` | Hosted/cloud isolation. | Reject. |
| `crates/onboarding/` | Product onboarding and marketing flows. | Reject. |
| `crates/voice_input/` | Voice input/transcription. | Reject. |
| `crates/warp_core/src/telemetry.rs`, `crates/warpui_core/src/telemetry/`, app focus telemetry | External telemetry/event queues. | Reject. |
| `script/sentry_create_release.sh`, `script/sentry_upload_dif.sh` | Sentry release/upload. | Reject. |

## Retained Generic Warp Code

| Area | Why retained | Merge rule |
| --- | --- | --- |
| Terminal emulator, blocks, shell integration | Core terminal product. | Accept compatible upstream fixes. |
| Input editor, slash commands, completions, NLD | Needed for terminal and ACP entry. | Accept or adapt. |
| AgentView shell and shortcuts | GUI shell for ACP conversations. | Adapt to ACP; keep generic conversation/code-review/help behavior. |
| Code review side panel | Generic Git diff UI, not old Warp Agent backend. | Accept local UI fixes. |
| Local MCP file config | Local extension/config mechanism. | Accept file-based local improvements. |
| Local object/persistence model | Needed by workflows, prompts, AI facts, MCP, conversation history. | Inspect carefully before accepting or removing. |
| `crates/remote_server/` | Remote terminal support. | Keep SSH/local behavior; reject Warp-account auth requirements. |
| OS launch-at-login | Operating-system setting, not account login. | Keep unless product scope changes. |

## Legacy Names Still Present

These names are not enough to decide merge behavior:

- `blocklist`: legacy name for AgentView and AI output code.
- `CloudObject`: can be local persisted object data after cloud sync removal.
- `server_id`: can be local schema compatibility.
- `warp_server_client`: still contains retained shared DTOs/identity/object types after cloud API removal.
- `remote_server`: remote terminal, not necessarily cloud account auth.

Always inspect call sites and data flow before making a merge decision.

## Required Audit Queries

Before finishing a major upstream merge, run:

```bash
rg -n "access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment" app crates script Cargo.toml
```

Every hit should be one of:

- Documentation.
- Local logging.
- Legacy local naming with no live cloud dependency.
- Retained remote-terminal code.
- Retained local object/persistence code.

Anything else should be removed or adapted to the fork contract.
