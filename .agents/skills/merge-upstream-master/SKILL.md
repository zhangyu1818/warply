---
name: merge-upstream-master
description: Reviews and ports commits or changes from upstream Warp master into this ACP-only macOS fork. Use for merges, cherry-picks, rebases, restacks, syncs, or selective upstream ports.
---

# Merge Upstream Master

## Purpose

Continuously bring compatible upstream terminal and macOS product improvements into this fork without restoring Warp-owned services. This fork changes service ownership and supported platforms; it is not frozen at the fork baseline. Every upstream commit must be reviewed before it is applied.

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
- The app must not require Warp-hosted Agent, Warp cloud accounts, Warp server APIs, Warp telemetry, or tracking.
- AgentView remains the UI shell, but agent backend behavior is ACP-only.
- Next Command and Prompt Suggestions remain OpenAI-compatible terminal suggestion flows.
- SSH remote terminal, SSH remote server, ControlMaster transport, remote file tree, remote command execution, and Warpify remain terminal features.
- MCP and skills belong to the ACP agent process, not the Warp app bundle or app settings.
- New upstream features are in scope when they can run locally or through a retained provider without depending on Warp services.
- Do not add backward-compatibility shims for deleted product areas, deleted settings, old persisted formats, or removed platform branches.

## Feature Preservation Rules

- Treat upstream as the source of truth for Warp's local terminal and macOS product behavior. The retained-area list is illustrative, not a closed allowlist.
- Decide from runtime ownership, data flow, and dependencies, not from whether the feature existed at the fork baseline or is already named in fork documentation.
- For every user-visible change, determine whether its core behavior still works with all Warp-owned endpoints, identities, hosted execution, telemetry, and tracking removed. If it does, accept it or adapt away only its incidental dependencies, including when the feature is new or absent from existing fork inventories.
- Adapt mixed changes by keeping their local UI, state model, persistence, and behavior while removing Warp service, account, sync, marketing, telemetry, and tracking integration. A removable reference to deleted plumbing is not a reason to drop the whole feature.
- For AgentView or AI features, retain upstream UI and local behavior only when execution can use ACP or an existing configurable provider boundary. Never restore Warp-hosted execution.
- Reject a feature only when its essential behavior depends on a removed Warp service or unsupported local platform and cannot operate through a retained local or provider-backed architecture.
- Never silently omit a user-visible local feature. Investigate its upstream dependencies and record the concrete reason for any rejected portion.

## Upstream Source Fidelity

- Treat the exact upstream source and commit history as the implementation authority for every accepted or adapted change.
- Begin by applying the original upstream commit or patch, or by copying its exact files and hunks. Only then make the smallest adaptations required by the fork contract.
- Do not independently rewrite, approximate, or recreate upstream core behavior from descriptions, release notes, screenshots, observed behavior, or memory. Use those materials only to locate or verify the authoritative source.
- A selective or manual port means selecting upstream code and modifying it for this fork; it never means writing an equivalent implementation from scratch.
- Preserve upstream structure, algorithms, tests, assets, and user-visible behavior wherever compatible.
- If the code does not apply cleanly, inspect upstream parents, call sites, history, and prerequisite commits. If the authoritative source cannot be inspected, do not reconstruct it by guessing.
- Limit new handwritten code to necessary fork integration glue and provider-boundary replacements around the copied upstream core implementation.

## Cross-Cutting Toolchain and Edition Changes

- Treat upstream Rust edition, minimum-toolchain, resolver, formatter, build-profile, and other workspace-wide engineering migrations as first-class merge subjects, even when they do not add a user-visible feature.
- Review these migrations independently from product-area decisions. Removing cloud, account, MCP, skills, or unsupported-platform paths does not justify dropping the migration for retained crates.
- Port the migration from the exact upstream commit for the retained workspace paths, then resolve edition or toolchain diagnostics with the smallest source-faithful changes. Do not hide a language-version mismatch by rewriting retained upstream behavior into an older dialect when the fork can adopt the upstream edition.
- If the complete migration cannot be adopted in one safe change, record the retained paths, blocked paths, and concrete compiler failures; do not mark the upstream commit fully reviewed while silently omitting retained workspace crates.

## Merge Workflow

1. Inspect the upstream commit, its feature context, and any dependent upstream commits before applying it.

```bash
git show --stat --summary <commit>
git show --name-status <commit>
git diff <commit>^ <commit> -- <path>
git log --oneline --reverse <base>..<commit>
```

2. Classify every touched area as `accept`, `adapt`, `reject`, or `not applicable`.

- `accept`: The change improves local macOS, terminal, or provider-backed behavior and does not restore deleted systems.
- `adapt`: The feature is retained, but upstream code also depends on removed auth, cloud, telemetry, tracking, MCP, skills, or platform plumbing. Port the upstream feature and remove or replace only those dependencies.
- `reject`: The change is for deleted product areas, deleted platform targets, specs, or compatibility shims.
- `not applicable`: The change touches code that no longer exists or only served removed systems.

For workspace-wide engineering commits, make a separate classification for each retained crate or build path. A large diff is not a reason to classify the whole commit as rejected.

3. Port accepted or adapted behavior according to Upstream Source Fidelity.

- Do not blindly cherry-pick a commit just because it builds.
- Remove Warp service and tracking hooks from otherwise retained features instead of rejecting the local functionality with them.
- Compare the result with current upstream behavior and account explicitly for every intentionally omitted hunk or behavior.

4. Record meaningful decisions and upstream provenance in `docs/agents-wiki/`.

- Record source commits and paths, plus short rationale for accepted, adapted, rejected, and intentionally omitted areas.
- Use `docs/agents-wiki/change-map.md` for durable path-level decisions.
- Do not add upstream `specs/**`, standalone `PRODUCT.md` or `TECH.md`, or `docs/superpowers/plans/**` documents.
- Do not recreate `llms.txt` or `llms-full.txt` for fork memory. This repo uses `docs/agents-wiki/` as its local LLM wiki and `.agents/skills/` for task procedures.

## Retained Areas

Usually accept or adapt compatible upstream fixes and new features for:

- Local-first terminal interaction, organization, and productivity behavior whose state and execution remain on-device.
- Terminal emulator, shell integration, PTY/session handling, input editor, completions.
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
