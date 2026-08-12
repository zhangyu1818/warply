# Fork Contract

This contract defines what must remain true after local development or upstream merges.

## Fork Purpose

The fork removes Warp **server/cloud/account/telemetry dependencies**. It does not freeze the feature set at the baseline date. Upstream behavior is in scope when its core can run locally or through a retained provider without those dependencies, whether or not it existed at baseline.

## Maintained Product Surface

The fork is a local-first terminal application:

- Terminal GUI and shell workflow.
- Warpify for subshells and SSH sessions.
- ACP-backed agent conversations.
- OpenAI-compatible Next Command.
- OpenAI-compatible Prompt Suggestions.
- Local settings and local persistence.
- Sparkle 2 app updates from this fork's GitHub Release appcast.
- macOS as the only maintained native host platform.

No maintained feature should require Warp login, Warp billing, Warp cloud objects, Warp-hosted agent services, or external telemetry. Do not add backward-compatibility paths for deleted Warp cloud/account/agent data; migrate retained local data directly to the current local schema.

## Required AI Behavior

### Agent

- Natural language input and `/agent` enter the existing AgentView shell.
- The request backend is ACP-only.
- ACP session config is driven by AI settings and ACP adapter config options.
- Model/backend choice is driven by ACP adapter configuration. Do not restore old Warp `/model`, Codex modal, per-terminal model override, or execution-profile model selector flows.
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
- Execution profiles should expose local permission behavior only. Do not add compatibility code for stored model choices or context-window overrides.

## Removed Systems

These were deliberately removed. If upstream changes touch them, the default decision is reject:

- `app/src/auth/`: account auth, anonymous id, SSO, access tokens, login UI.
- `app/src/billing/`: billing, usage, upgrade, referral/invite gates.
- Unused cloud/billing/team/referral SVGs and icon variants, including `cloud-01.svg`, `cloud-filled.svg`, `cloud-off.svg`, `create-team.svg`, `credits.svg`, `referral-*`, `Icon::Cloud`, `Icon::CloudFilled`, and `Icon::CoinsStacked`.
- `app/src/autoupdate/`: upstream Warp update/changelog infrastructure. This fork uses its own Sparkle 2 updater under `app/src/updater/` instead.
- `app/src/crash_reporting/` and Sentry scripts.
- `app/src/ai/agent_sdk/`: old Warp-hosted Agent SDK and harnesses.
- `app/src/ai/agent_management/`: cloud agent management UI.
- `app/src/ai/ambient_agents/`: ambient/scheduled/cloud agents.
- `Icon::AmbientAgentMode` and `app/assets/bundled/svg/ambient-agent-mode.svg`: ambient/cloud agent icon surface.
- `crates/graphql/` and `crates/warp_graphql_schema/`: Warp cloud GraphQL client/schema.
- `crates/managed_secrets/` and `crates/managed_secrets_wasm/`.
- `crates/isolation_platform/`: hosted/cloud isolation infrastructure.
- `crates/onboarding/`: onboarding and marketing flows.
- `app/assets/async/png/onboarding/`: onboarding, agent intention, third-party toolbar/notification, and Warp Drive marketing screenshots.
- `crates/voice_input/`: voice input/transcription.
- Old Warp model/profile selector surfaces, Codex modal/deeplink handling, and per-terminal LLM override persistence.
- Bundled, channel-gated, MCP, or locally scanned skills managed by the Warp app. ACP agents own their own skill and MCP configuration.
- Agent shared-session viewer action sync, remote action mirroring, cloud session sharing, view-only action-result replay, shared-session resize sync, and input peer-edit sync.
- WASM-only cloud conversation web-viewer contexts such as `Workspace_CloudConversationWebViewer`. Local conversation transcript viewing may remain without cloud viewer gates.
- External telemetry/event-store code and app focus telemetry.
- Linux and Windows native host implementations, WSL/MSYS2 host executors, and Linux/Windows packaging/build-support paths.

## Retained Systems

These are still in scope and should receive compatible upstream fixes:

- Terminal emulator, blocks, PTY, shell integration, session restoration, input editor, completions.
- Warpify, including subshell bootstrap, SSH warpification, tmux wrapper checks, and related UI states.
- Natural language detection.
- AgentView shell, local conversation history, conversation navigation, help shortcuts, code review panel, context chips, local file attachments, and local plan/block/diff context references that are converted into ACP prompt context.
- AgentView conversation list/navigation is a retained local ACP UI path and should not be reintroduced as an optional rollout gate.
- Long-running command control transfer and CLI subagent takeover/handback UI when routed through AgentView and ACP.
- ACP implementation and ACP UI rendering.
- Generic read-only ACP tool-call diff rendering.
- OpenAI-compatible Next Command and Prompt Suggestions.
- ACP tool-call rendering only. MCP server configuration, capability probing, startup, execution, and MCP/skill instructions belong to the ACP agent process.
- ACP client capabilities are host capabilities exposed to the ACP process, not MCP server capability queries. Next Command and Prompt Suggestions do not use ACP skills, bundled skills, or app-managed MCP.
- Do not bundle or restore skills that teach agents to manage Warp-owned `.mcp.json` server config, a Warp MCP settings pane, provider-specific hosted agents, tab/settings helpers, app-distributed MCP workflows, local skill scanners, `/skills` or `/open-skill` UI, `ReadSkill`/`InvokeSkill` actions, or tab-config skill CTAs.
- Web/WASM and Linux/Windows host branches are not retained compatibility layers. Keep native macOS host behavior direct; keep remote host detection only as part of SSH/remote-server setup.
- Local persisted objects used by workflows, prompts, AI facts, and local conversation data.
- Retained local objects must not require cloud online status, cloud sync state, or Warp sharing permissions for local edit/delete behavior.
- Retained local object pending state may exist for local persistence and quit-warning accounting, but it must not render Warp cloud sync badges or cloud save/error UX.
- Environment-variable collection references to user-installed secret-manager CLIs such as 1Password or LastPass are retained local shell integrations. Do not restore Warp managed secrets, cloud secret storage, cloud credential discovery, or account-backed secret APIs around them.
- Local object metadata should not keep online-only pending metadata, permissions, untrash, or delete fields. Persist retained metadata/permissions directly to the local store.
- Local object metadata writes must support the current local object id column instead of assuming server-assigned identity.
- Local object create/update/delete events should carry the local `SyncId` directly. Do not restore client-id to server-id backfill flows after object creation.
- Persisted local UI references such as saved workflow panes, saved environment-variable panes, and command-history workflow links should use the current client-id form without server-style hash fallback.
- Persisted local UI state should use current formats only; do not restore old-format readers such as bare ANSI tab-color payloads.
- Retained AI history may store ACP model metadata for display, but must not restore old Warp planning-model selector state.
- Do not restore app-level cloud online/offline mode, offline cloud toolbar indicators, debug network-status toggles, or suggestion gates that depend on Warp cloud connectivity.
- OS launch-at-login settings.
- Sparkle 2 update checks and downloads for Warply releases, using the bundled Sparkle framework, standard Sparkle UI, GitHub Release DMG assets, and `appcast.xml` from GitHub Releases. Do not restore Warp `channel_versions`, Warp update server APIs, Linux package updaters, or old autoupdate product code around this.
- SSH/remote terminal behavior that does not depend on Warp account auth.
- macOS host platform integration, packaging, secure storage, user preferences, local PTY, windowing, menus, and launch-at-login behavior.

## macOS-Only Host Notes

- Local host code should assume macOS/POSIX behavior. Do not reintroduce local WSL/MSYS2, ConPTY, Windows PATH/cmd.exe, Linux windowing, Linux packaging, or Windows packaging branches.
- Host UI and shortcuts should follow macOS-only behavior: AppKit windowing, native macOS modals, Cmd-based shortcuts, macOS font/log conventions, and Unix path encoding. Do not add Linux/Windows keybinding alternatives or platform fallback branches.
- Terminal bootstrap scripts should stay on the POSIX DCS JSON path unless a retained macOS or SSH workflow proves otherwise.
- SSH is part of the retained terminal surface. Preserve macOS-to-remote workflows, remote shell/session metadata, remote command execution, and remote-server behavior when they do not require Warp account auth.

## Legacy Naming Policy

Names alone are not merge evidence.

- `blocklist` is legacy naming for the AgentView path.
- `subagent`, `handoff`, and `control transfer` may describe retained long-running command behavior if it routes through AgentView and ACP; remove only cloud handoff/orchestration/remote-control services.
- `CloudObject` may still represent local persisted object data.
- Saved workflows and environment-variable collections are local persisted objects. Do not restore `CloudWorkflow`, `WorkflowType::Cloud`, `WorkflowSource::PersonalCloud`, `CloudEnvVarCollection`, or command-history `cloud_workflow_id` compatibility paths.
- `ServerId` is legacy naming only if still present in retained local-object data. `stable_object_id` is the current SQLite column for that server-style local identifier form. Do not add new compatibility fallback around either.
- `remote_server` is remote terminal support, not account login by itself.
- References to Linux/Windows inside terminal protocol parsing, shell target metadata, or path conversion must be checked before removal. Preserve SSH/remote terminal behavior; remove only local host platform support unless the code is required by macOS-to-remote workflows.

Before deleting or restoring code with these names, inspect call sites and persistence usage.
