# Warp ACP-Only macOS Fork Agent Guide

This is the repository-level entry point for agents working in this fork. It defines what must stay true after local changes or future upstream merges from Warp.

## Core Contract

This fork is a macOS terminal client with ACP-backed AI surfaces.

The fork's purpose is to remove Warp **server/cloud/account/telemetry dependencies** — not to freeze the feature set at the baseline date. Purely local upstream features (no Warp server API, cloud account, billing, auth, or telemetry dependency) are in scope and should be accepted after review, whether or not they existed at baseline.

- The maintained app target is macOS only.
- Warp-hosted Agent, Warp cloud accounts, and Warp server APIs are removed.
- AgentView remains the UI shell, but all agent backend behavior is ACP-only.
- Next Command and Prompt Suggestions remain OpenAI-compatible terminal suggestion flows, not Warp-hosted AI APIs.
- SSH remote terminal, SSH remote server, and Warpify are retained terminal features.
- MCP and skills belong to the ACP agent process, not the Warp app bundle or app settings.
- New upstream features remain in scope when they can run locally or through a retained provider without depending on Warp services.
- Do not add backward-compatibility shims for deleted product areas, deleted settings, old persisted formats, or removed platform branches.

Fork baseline:

```text
19659d12 refactor: create local ACP-only Warp fork
```

Use that baseline, plus the current `docs/agents-wiki/` records, when deciding whether future upstream changes should be accepted, adapted, or rejected.

Project memory lives in `docs/agents-wiki/`; do not recreate `llms.txt`, `llms-full.txt`, upstream `specs/**`, standalone `PRODUCT.md` or `TECH.md`, or `docs/superpowers/plans/**` as fork memory.

## Merge Discipline

Every upstream commit must be reviewed before it is applied.

- Inspect the commit and its touched paths.
- Classify each change as accept, adapt, reject, or not applicable.
- Treat retained areas as directions, not a closed allowlist. Judge new features by runtime ownership, data flow, and service dependencies rather than whether they existed at the fork baseline.
- If only part of a commit fits this fork, select and port the corresponding upstream code for that part.
- If a retained feature is involved, adapt it to the current fork architecture instead of restoring upstream dependency chains.
- Record meaningful merge decisions and cleanup rationale in `docs/agents-wiki/`.
- Do not treat passing tests as proof that product boundaries are preserved; inspect the actual code paths.
- Review every commit for both product behavior and cross-cutting engineering changes. Feature-oriented triage must not skip manifests, toolchains, formatter/lint configuration, build scripts, workspace metadata, or retained prerequisite infrastructure.

## Upstream Source Fidelity

For upstream merge work, the exact upstream source and commit history are the implementation authority.

- Every accepted or adapted change must begin from the actual upstream commit, patch, file, or hunk. Apply or copy that implementation first, then make the smallest changes required for this fork.
- Do not independently rewrite, approximate, or recreate upstream core behavior from descriptions, release notes, screenshots, observed behavior, or memory. Those materials may help locate or verify the source, but they are not implementation sources.
- A selective or manual port means selecting upstream code and modifying it for this fork; it never means writing an equivalent implementation from scratch.
- Preserve upstream structure, algorithms, tests, assets, and user-visible behavior wherever they are compatible with the fork contract.
- If the code cannot be applied cleanly, inspect upstream parents, call sites, history, and prerequisite commits. If the authoritative source cannot be inspected, do not reconstruct it by guessing.
- New handwritten code is limited to necessary fork integration glue and provider-boundary replacements around the copied upstream core implementation.

The mandatory port order for accepted or adapted changes is:

1. Inspect and classify the upstream commit and prerequisites.
2. Apply the exact upstream implementation to the worktree. Prefer `git cherry-pick --no-commit` for fully retained commits; for mixed commits, use an exact three-way patch or copy the exact upstream files and hunks.
3. Resolve conflicts on the applied upstream source, preserving its core behavior and making only the smallest fork-specific removals or provider-boundary adaptations.
4. Compare the result with the upstream patch and record every intentionally omitted path or meaningful hunk.

Do not start from a blank local implementation. A selective or manual port is still copied upstream code followed by conflict resolution and fork adaptation.

Workspace-wide engineering migrations are retained scope. Rust edition, minimum-toolchain, resolver, formatter, build-profile, and similar upstream migrations must be reviewed independently from product-service decisions. Do not drop a retained crate's migration because the same upstream commit also touches removed cloud, MCP, skills, or unsupported-platform code. Apply the authoritative upstream migration to retained paths first, then make the smallest source-faithful compatibility edits required by the fork.

For those migrations, inventory every retained workspace manifest and build path touched upstream. A commit is not fully reviewed until each retained crate is migrated or a concrete compiler/toolchain blocker is recorded. Do not preserve an older Rust dialect by rewriting upstream code when the fork can adopt the upstream edition.

## Retained Areas

These directions are part of the fork and should receive compatible upstream fixes and new features:

- Local-first terminal interaction, organization, and productivity behavior whose state and execution remain on-device.
- Terminal emulator, blocks, shell integration, PTY/session handling, input editor, completions.
- Natural language detection and input classification.
- AgentView shell, conversation navigation, help shortcuts, code review side panel, context chips, local attachments.
- ACP implementation under `app/src/ai/acp/`.
- OpenAI-compatible Next Command and Prompt Suggestions under `app/src/ai/terminal_suggestions/` and `app/src/ai/predict/terminal_*`.
- ACP protocol rendering for assistant text, reasoning, plans, permissions, commands, diffs, and generic tool-call updates.
- Local persistence for conversations, workflows, prompts, AI facts, terminal sessions, and retained local object data.
- macOS host integration, AppKit/windowing, local preferences, secure storage, signing, launch-at-login.
- SSH remote terminal, SSH remote server, ControlMaster transport, remote file tree, remote command execution, and Warpify for subshell and SSH sessions.

SSH/Warpify must not be removed just because remote hosts can be Linux or Windows. Remote host platform checks are retained when they support SSH terminal capability detection or remote-server setup.

## Removed Areas

Do not restore these systems from upstream:

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

## Upstream Merge Decision Table

| Upstream change area | Decision |
| --- | --- |
| Terminal, shell integration, PTY, blocks, editor, completions | Usually accept or port directly after review. |
| New local or provider-backed product behavior | Accept or adapt after reviewing runtime ownership and data flow. Remove rollout/service plumbing as needed; do not reject behavior merely because it postdates the baseline. |
| GPUI/Warp UI framework, macOS windowing, rendering | Usually accept if it does not depend on removed product surfaces. |
| ACP, AgentView, `blocklist`, conversation history, AI settings | Selectively port the upstream implementation and preserve ACP-only backend behavior. |
| Next Command, Prompt Suggestions, prediction APIs | Port UI/context improvements only; keep the OpenAI-compatible provider. |
| SSH, remote terminal, remote server, Warpify | Keep when it supports terminal behavior without Warp account auth. Reject Warp-hosted downloads, token auth, and cloud remote-control semantics. |
| Persistence and migrations | Accept only for retained local data. Reject account/team/billing/cloud/MCP/skills compatibility migrations. |
| Rust edition, toolchain, resolver, formatter, build profile, workspace metadata | Apply the upstream migration to every retained crate and build path, independently of removed product paths in the same commit. Resolve diagnostics with minimal source-faithful edits. |
| `CloudObject`, `server_id`, `local_object_model` naming | Inspect data flow before deciding. Some names remain for local schema compatibility, not cloud behavior. |
| MCP or skills | Reject app-side config, discovery, capability probing, invocation, or bundled resources. Keep only ACP protocol event rendering and ACP client capabilities for retained host handlers. |
| Linux/Windows/Web platform code | Reject local client platform implementation and packaging code. Keep remote-host detection only for retained SSH/remote-server behavior. |
| Upstream specs, product/tech plans, process planning docs | Reject by default. Keep fork-owned merge records in `docs/agents-wiki/` instead. |
| Auth, billing, Teams, Warp Drive cloud, GraphQL, managed secrets, telemetry, crash reporting, onboarding | Reject unless the change can be reduced to a retained local utility without restoring the product area. |
| Cargo dependencies | Keep removals unless a retained macOS or SSH/remote-terminal feature needs the crate. Be skeptical of reintroduced cloud/API/reporting crates. |

## Ambiguous Names

Names alone are not enough to decide merge behavior.

- `remote` may mean retained SSH remote terminal or removed cloud remote-control.
- `server` may mean retained SSH remote-server daemon or removed Warp server API.
- `CloudObject`, `server_id`, and `SyncId` can be legacy local schema names.
- `MCP capability` in ACP code can mean retained ACP client capabilities; app-side MCP probing is still rejected.
- `skills` in upstream code usually means app-managed or bundled skills and should be rejected.
- `Linux` or `Windows` may be retained remote host metadata, not local client platform support.

Inspect call sites, persistence usage, settings, and runtime ownership before accepting or deleting code.

## Required Reading For Merge Work

- `docs/agents-wiki/README.md`
- `docs/agents-wiki/fork-contract.md`
- `docs/agents-wiki/upstream-merge-guide.md`
- `docs/agents-wiki/change-map.md`
- The latest `docs/agents-wiki/upstream-master-audit-*.md` file for the upstream range being reviewed.

## Verification

For merge or code changes, run focused checks first, then expand as needed. Every independently ported feature must pass `cargo build -p warp --all-targets --message-format short`; immediately run `cargo clean` after the build succeeds before moving to the next feature. Edition, toolchain, resolver, formatter, build-profile, or other workspace-wide migrations must instead pass `cargo build --workspace --all-targets --message-format short`, followed immediately by `cargo clean`.

```bash
cargo check -p warp --all-targets --message-format short
cargo check --workspace --all-targets --message-format short
cargo fmt -- --check
cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'
```

Before finishing a major upstream merge, also scan for restored deleted surfaces:

```bash
rg -n "access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment" app crates script Cargo.toml
rg -n "mcp.*capab|capab.*mcp|mcp_server|mcpServers|bundled skills|channel-gated-skills|ReadSkill|InvokeSkill" app crates
rg -n "target_os = \"linux\"|target_os = \"windows\"|cfg\\(windows\\)|WSL|MSYS2|ConPTY|Wayland|X11|winreg|x11rb" app crates Cargo.toml
```

Allowed hits must be documentation, tests, retained remote-terminal behavior, retained local-object schema names, or retained ACP protocol code.
