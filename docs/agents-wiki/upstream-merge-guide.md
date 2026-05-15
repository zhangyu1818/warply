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
- macOS host platform, packaging, signing, local secure storage, user preferences, launch-at-login, and AppKit/windowing fixes.
- Local settings infrastructure.
- Dependency security updates that do not re-add removed cloud/API/reporting crates.
- Remote terminal fixes that do not require Warp account tokens.
- Warpify fixes for subshell and SSH sessions, including tmux wrapper behavior.
- ACP protocol rendering, including generic tool-call display.

## Adapt

Port the behavior manually when upstream touches retained but forked areas:

| Area | Adaptation rule |
| --- | --- |
| `app/src/ai/blocklist/` | Keep AgentView shell/UI improvements, but route backend behavior through ACP. |
| `app/src/ai/agent/` and conversation history | Preserve local ACP transcript/history semantics. Do not restore server transcript APIs. |
| `app/src/ai/acp/` | Preserve ACP protocol-native state and event mapping. |
| `app/src/ai/predict/` | Keep Next Command and Prompt Suggestions context/UI improvements, but use the OpenAI-compatible suggestions provider. |
| `app/src/ai/terminal_suggestions/` | Keep OpenAI-compatible endpoint/model/API key flow for Next Command and Prompt Suggestions. |
| `app/src/settings/ai.rs` and `app/src/settings_view/ai_page.rs` | Keep only ACP and terminal suggestions settings. |
| `app/src/terminal/input*` | Keep NLD and `/agent` behavior entering ACP AgentView. |
| AgentView conversation list/navigation | Keep as retained local ACP UI. Do not restore rollout flags that make the local conversation list optional. |
| Local context references | Keep `<plan:...>`, `<block:...>`, `<change:...>`, file, and diff context wiring when it becomes ACP prompt context. Do not reinterpret this as app-managed skills or MCP. |
| `app/src/cloud_object/` and `crates/local_object_model/` | Inspect whether the data supports local workflows/prompts/facts/history before accepting. |
| `crates/persistence/` | Accept migrations/schema only for retained local data. |
| `crates/remote_server/` | Keep local/SSH terminal behavior; reject account-auth token requirements. |
| SSH/subshell Warpify | Preserve command blocks, completions, input editor behavior, syntax highlighting, file tree/code diff integration, and other full Warp terminal features inside nested or remote sessions. Do not remove Warpify as cloud cleanup. Do not restore Warp-hosted portable tmux download/install paths; remote package-manager install scripts are the retained Linux path. |
| Web/network-backed PTY (`remote_tty`, `ssh-proxy-server`, websocket PTY client) | Reject. This fork keeps SSH remote server and Warpify, not the old Warp-on-Web remote PTY path. |

## Reject

Reject or strip these upstream changes unless they can be reduced to a retained local utility:

- `app/src/auth/` or any access-token acquisition path.
- Billing, credits, usage, referrals, upgrade, invite, Teams, workspace discovery.
- Unused cloud/billing/team/referral icons and assets. Do not remove retained generic upload/publish icons merely because their filename contains `cloud`.
- Cloud Warp Drive sharing/sync/import/export UI.
- `app/src/ai/agent_sdk/`, `agent_management`, `ambient_agents`, scheduled/cloud agents.
- Old server-backed AI APIs, Warp-hosted suggestion APIs, cloud orchestration, handoff, remote-control cloud controls.
- Ambient/cloud agent icons or menu entries. Keep the retained `agentmode.svg` entrypoint only when it routes to ACP AgentView.
- `crates/graphql/`, `crates/warp_graphql_schema/`, cloud GraphQL generated queries/mutations.
- Managed secrets, cloud environments, hosted isolation.
- Voice input/transcription.
- Onboarding/marketing UI and assets, including `app/assets/async/png/onboarding/`.
- Internal plugin/template installers or debug bindings that write Warp-owned GitHub plugin entries into user tool configs.
- App-bundled skills, channel-gated skills, bundled MCP skill directories, local skill scanners/managers, `/skills`, `/open-skill`, `ReadSkill`, `InvokeSkill`, tab-config skill CTAs, or CLI skill spec parsing. ACP agents own their own skills and MCP configuration; OpenAI-compatible terminal suggestions do not use skill bundles.
- Inert skill resource directories are still rejected. Do not keep `resources/bundled/skills/`, `resources/bundled/mcp_skills/`, or `resources/channel-gated-skills/` just because runtime code no longer references them.
- Agent shared-session viewer action sync, cloud session sharing, remote action execution mirroring, view-only action-result replay, shared-session resize sync, or input peer-edit sync. Keep local transcript viewing and ACP read-only diff rendering only when they do not restore cloud session services.
- Shared-block cloud sharing actions, title-generation settings, keymap flags, dialogs, or user-facing copy. Retained local terminal blocks may keep local layout helpers, but not Warp cloud sharing surfaces.
- WASM-only cloud conversation web-viewer contexts and shortcut gates. Keep local conversation transcript viewing only when it stays local and ACP-compatible.
- Telemetry, crash reporting, Sentry release/upload, event stores.
- Linux/Windows native host platform implementations, WSL/MSYS2 local host executors, Linux packaging, Windows packaging, and Linux/Windows-only build dependencies.
- Web/WASM app, CLI, search/menu, code review, terminal, plugin-host, and remote-server compile branches or no-op stubs. This fork keeps a native macOS host path; remote OS detection may remain only when required by retained SSH/remote-server behavior.
- Shell bootstrap hooks that only support Warp app/package updates on non-macOS hosts, such as `FinishUpdate` and Linux apt-source repair helpers.

## Conflict Examples

### Upstream edits `app/src/ai/blocklist/agent_view`

Port layout, keyboard, scrolling, accessibility, or rendering fixes. Do not restore cloud agent controls, usage banners, remote-control cloud controls, or old model/profile selectors.

### Upstream edits old Warp Agent SDK files

Ignore the commit unless it contains a protocol/UI idea that should be reimplemented in `app/src/ai/acp/` or ACP rendering. Do not restore `app/src/ai/agent_sdk/`.

### Upstream edits prediction APIs

Keep terminal context collection or UI placement improvements. Replace network calls with the OpenAI-compatible terminal suggestions client.

### Upstream adds persistence migrations

Accept if the migration supports retained local conversations, workflows, prompts, AI facts, terminal sessions, or UI state.

Reject if it supports users, teams, billing, cloud refresh, cloud sharing, hosted agents, managed secrets, or usage tracking.

Reject migrations that only preserve Warp app-managed MCP server config, OAuth tokens, capability caches, or running-server state. MCP belongs to the ACP agent process in this fork.

Reject migrations or UI/action changes that preserve Warp app-managed local skills, skill file watches, bundled skill discovery, skill invocation history, or skill-specific editor buttons. Skills belong to the ACP agent process in this fork.

Do not confuse ACP client capabilities with MCP capability probing. ACP client capabilities may advertise retained Warp host handlers to the ACP process; Warp must not query MCP server capabilities, manage MCP config, or route OpenAI-compatible terminal suggestions through skills.

Reject UI/action changes that make retained local objects depend on network online status, Warp cloud sync state, sharing permissions, server-assigned IDs, or client-id to server-id backfill for local create/edit/delete flows.

When upstream changes retained local AgentView action rendering such as grep/file-glob display, port UI and local-result handling without restoring rollout gates or Warp-hosted tool execution. ACP tool calls should remain protocol-native UI events; app-managed MCP/skills and cloud agent tooling should not be reintroduced.

Keep local Docker sandbox terminal support when upstream changes `sbx` spawning, shell bootstrap, or terminal-session UI. Do not restore rollout gates, stale-setting fallbacks, hosted sandbox/cloud isolation, or old agent sandbox routing around that local terminal mode.

Reject Drive/local-object UI changes that restore cloud sync badges, cloud save progress, or cloud save-error indicators. Keep local pending state only when it supports local persistence or quit-warning behavior.

Reject changes that restore workflow or environment-variable collection runtime names and schema fields as `CloudWorkflow`, `WorkflowType::Cloud`, `WorkflowSource::PersonalCloud`, `CloudEnvVarCollection`, or `cloud_workflow_id`. Port them as saved/local object behavior without legacy compatibility aliases.

Reject saved workflow pane, saved environment-variable pane, or command-history link readers that fall back from client IDs to old server-style object hashes.

Reject local object reference parsing that guesses by trying server-style IDs first and client IDs second. When both retained `SyncId` forms are still necessary, parse by the stored prefix.

Reject workflow import/deserialization changes that silently default malformed or old argument data to text arguments. Current-format workflow arguments must include a valid `arg_type`.

Reject local settings migrations that keep deleted private setting keys alive only to respect older local preferences. Port retained settings to the current key only.

Reject UI-state readers that accept deleted old formats, such as bare ANSI tab colors instead of the current `SelectedTabColor` payload.

Reject AI-query runtime fields that revive old Warp planning-model selector state. ACP model metadata may be retained for local history display, but model routing belongs to ACP configuration.

Reject local-object metadata changes that restore online-only pending metadata, permission, untrash, or delete flags. Adapt them to direct local SQLite persistence when the object type is retained.

When touching retained `CloudModel`, `CloudObject`, `SyncId`, or `ServerId` code, keep behavior local. Do not add comments, schema descriptions, or UI copy that implies server ownership, cloud sync, cloud conflict resolution, or server-assigned identity beyond the legacy type names already present.

Reject changes that restore app-level cloud online/offline state, offline cloud indicators, debug network-status toggles, or Next Command/Prompt Suggestions gates based on Warp cloud connectivity. OpenAI-compatible providers should fail through their own request path.

### Upstream changes `Cargo.toml` or `Cargo.lock`

Regenerate dependency state from retained manifests. Do not accept reintroduced GraphQL, Sentry, telemetry, managed-secret, onboarding, old-agent-SDK, voice-input, Linux host, or Windows host crates unless a retained macOS or SSH/remote-terminal feature explicitly needs them.

### Upstream changes Linux or Windows platform paths

Reject Linux/Windows host application, packaging, single-instance, secure-storage, user-preferences, local PTY, WSL/MSYS2, and windowing changes. Do not reject SSH or remote terminal code merely because it can connect to Linux/Windows hosts; inspect whether the code runs on the macOS client or on a remote terminal path.

## Post-Merge Audit

Run targeted searches and inspect every hit:

```bash
rg -n "access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment" app crates script Cargo.toml
rg -n "target_os = \"linux\"|target_os = \"windows\"|cfg\\(windows\\)|WSL|MSYS2|x11|wayland|winreg|windows-registry|x11rb" app crates Cargo.toml
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
