---
name: merge-upstream-master
description: Use when porting commits or changes from upstream Warp master into this ACP-only macOS fork, including merge, cherry-pick, rebase, restack, sync, or manual apply requests.
---

# Merge Upstream Master

## Purpose

Port upstream Warp changes into this fork without restoring removed product areas. Every upstream commit must be reviewed before it is applied.

The fork's purpose is to remove Warp **server/cloud/account/telemetry dependencies** — not to freeze the feature set at the baseline date. New upstream features that are purely local (no Warp server API, cloud account, billing, auth, or telemetry dependency) are in scope and should be accepted after review. Tab grouping, keyboard shortcuts, editor improvements, and other client-only features belong to the fork's terminal product direction as much as the features that existed at baseline.

## Required Reading

Read these before touching code for upstream merge work:

- `AGENTS.md`
- `docs/agents-wiki/README.md`
- `docs/agents-wiki/fork-contract.md`
- `docs/agents-wiki/upstream-merge-guide.md`
- `docs/agents-wiki/change-map.md`
- The latest `docs/agents-wiki/upstream-master-audit-*.md` file for the upstream range being reviewed

## Core Contract

This fork is a macOS terminal client with ACP-backed AI surfaces.

- The maintained app target is macOS only.
- Warp-hosted Agent, Warp cloud accounts, and Warp server APIs are removed.
- AgentView remains the UI shell, but agent backend behavior is ACP-only.
- Next Command and Prompt Suggestions remain OpenAI-compatible terminal suggestion flows.
- SSH remote terminal, SSH remote server, ControlMaster transport, remote file tree, remote command execution, and Warpify remain terminal features.
- MCP and skills belong to the ACP agent process, not the Warp app bundle or app settings.
- Do not add backward-compatibility shims for deleted product areas, deleted settings, old persisted formats, or removed platform branches.

## Merge Workflow

1. Inspect the upstream commit before applying it.

```bash
git show --stat <commit>
git show --name-only <commit>
git show <commit> -- <path>
```

2. Classify every touched area as `accept`, `adapt`, `reject`, or `not applicable`.

- `accept`: The change improves retained or **new purely-local** terminal/macOS/UI behavior and does not restore deleted systems or add any Warp server/account/telemetry dependency.
- `adapt`: The feature is retained or new-local, but upstream code depends on removed auth/cloud/telemetry/MCP/skills/platform plumbing. Port only the useful part.
- `reject`: The change is for deleted product areas, deleted platform targets, specs, or compatibility shims.
- `not applicable`: The change touches code that no longer exists or only served removed systems.

### New local features

Upstream may add features after the fork baseline date that are purely local — no Warp server API, cloud account, billing, telemetry, or auth dependency. These are **in scope**. The fork removes Warp server dependencies, not local terminal features. Do not reject a feature merely because it did not exist at baseline.

Accept a new local feature when review confirms all of:

- No Warp server/cloud/account/billing/auth/telemetry dependency anywhere in the implementation.
- No rollout flag that gates on server-side state. If upstream gates the feature behind a `FeatureFlag`, strip the flag and run the feature directly (same as retained features).
- It runs on macOS with the fork's local architecture: ACP for agent surfaces, OpenAI-compatible endpoints for suggestions, local persistence, AppKit windowing.
- It does not depend on a removed module to function. If it does, adapt the dependency or defer with a recorded rationale.

Examples of features that qualify: tab grouping, tab color cycling, editor/viewer improvements, new local slash commands, keyboard shortcuts, pane/window management, completion enhancements — anything that is client-side only.

3. Apply only the accepted or adapted parts.

- Do not blindly cherry-pick a commit just because it builds.
- If only part of a commit fits, manually port that part.
- If a retained feature is involved, adapt it to the current fork architecture instead of restoring upstream dependency chains.
- Compare against master behavior when debugging regressions, then decide whether matching master or intentionally diverging better preserves the fork contract.

4. Record meaningful decisions in `docs/agents-wiki/`.

- Write short merge rationale for accepted/adapted/rejected areas.
- Use `docs/agents-wiki/change-map.md` for durable path-level decisions.
- Do not add upstream `specs/**`, standalone `PRODUCT.md` or `TECH.md`, or `docs/superpowers/plans/**` documents.
- Do not recreate `llms.txt` or `llms-full.txt` for fork memory. This repo uses `docs/agents-wiki/` as its local LLM wiki and `.agents/skills/` for task procedures.

## Retained Areas

This list covers features that existed at baseline. It is **not exhaustive** — new upstream features that are purely local (see "New local features" above) belong here too once accepted. Usually accept or adapt compatible upstream fixes for:

- Terminal emulator, blocks, shell integration, PTY/session handling, input editor, completions.
- Natural language detection and input classification.
- AgentView shell, conversation navigation, help shortcuts, code review side panel, context chips, local attachments.
- ACP implementation under `app/src/ai/acp/`.
- OpenAI-compatible Next Command and Prompt Suggestions under `app/src/ai/terminal_suggestions/` and `app/src/ai/predict/terminal_*`.
- ACP protocol rendering for assistant text, reasoning, plans, permissions, commands, diffs, and generic tool-call updates.
- Local persistence for conversations, workflows, prompts, AI facts, terminal sessions, and retained local object data.
- macOS host integration, AppKit/windowing, local preferences, secure storage, signing, launch-at-login.
- SSH remote terminal, SSH remote server, ControlMaster transport, remote file tree, remote command execution, and Warpify for subshell and SSH sessions.

SSH and Warpify must not be removed just because remote hosts can be Linux or Windows. Remote-host platform checks are retained when they support SSH terminal capability detection or remote-server setup.

## Removed Areas

Reject upstream changes that restore:

- Account auth, anonymous user creation, access token retrieval, SSO, paste-token login.
- Billing, usage credits, referrals, upgrade, invite, Teams, workspace discovery.
- Cloud Warp Drive sharing, sync, import, export, and cloud sharing UI.
- Old Warp Agent SDK, cloud agents, ambient agents, scheduled agents, orchestration, handoff, and cloud remote-control semantics.
- Warp cloud GraphQL clients/schema, server API clients, managed secrets, hosted isolation, cloud environments.
- App-managed MCP configuration, MCP capability probing, MCP server startup, MCP persistence, MCP permissions, or MCP settings panes.
- App-bundled skills, bundled MCP skills, channel-gated skills, local skill scanners/managers, `/skills`, `/open-skill`, `ReadSkill`, or `InvokeSkill`.
- Voice input and hosted transcription.
- Onboarding, cloud marketing surfaces, Oz/cloud-agent assets.
- External telemetry, crash reporting, Sentry release/upload scripts, app focus telemetry, event queues.
- Old Warp AI `/model` and `/profile` selector flows.
- Native Linux/Windows client platform code, WSL/MSYS2 local host executors, Linux packaging, Windows packaging, Web/WASM app targets, and platform no-op stubs.
- Upstream specs and planning docs such as `specs/**`, standalone `PRODUCT.md` or `TECH.md`, and `docs/superpowers/plans/**`.

## Ambiguous Names

Names alone are not enough to decide.

- `remote` may mean retained SSH remote terminal or removed cloud remote-control.
- `server` may mean retained SSH remote-server daemon or removed Warp server API.
- `CloudObject`, `server_id`, and `SyncId` can be legacy local schema names.
- `MCP capability` in ACP code can mean retained ACP client capabilities; app-side MCP probing is rejected.
- `skills` in upstream code usually means app-managed or bundled skills and should be rejected.
- `Linux` or `Windows` may be retained remote host metadata, not local client platform support.

Inspect call sites, persistence usage, settings, runtime ownership, and user-visible behavior before accepting or deleting code.

## Verification

Run focused checks first, then expand as needed:

```bash
cargo check -p warp --all-targets --message-format short
cargo check --workspace --all-targets --message-format short
cargo fmt -- --check
cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'
```

Before finishing major upstream merge work, scan for restored deleted surfaces:

```bash
rg -n "access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment" app crates script Cargo.toml
rg -n "mcp.*capab|capab.*mcp|mcp_server|mcpServers|bundled skills|channel-gated-skills|ReadSkill|InvokeSkill" app crates
rg -n "target_os = \"linux\"|target_os = \"windows\"|cfg\\(windows\\)|WSL|MSYS2|ConPTY|Wayland|X11|winreg|x11rb" app crates Cargo.toml
```

Allowed hits must be documentation, tests, retained remote-terminal behavior, retained local-object schema names, or retained ACP protocol code.
