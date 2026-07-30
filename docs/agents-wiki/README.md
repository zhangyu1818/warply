# Agents / LLM Wiki

This wiki is the project-level LLM memory for the local ACP fork. Its purpose is to make future upstream merges from original Warp possible without accidentally restoring removed cloud product code.

This is not an `llms.txt` site index. `llms.txt` is a website-root convention for publishing a curated public content map to external LLM tools. This repository instead uses a local, versioned wiki under `docs/agents-wiki/` as the durable source of truth for fork decisions.

## Memory Model

- `AGENTS.md`: entry contract and high-level rules agents should read first.
- `.agents/skills/`: procedural workflows agents can load for specific tasks, such as upstream merge work.
- `docs/agents-wiki/`: durable project memory, merge decisions, retained/deleted surface boundaries, and path-level cleanup rationale.

Do not create a parallel `llms.txt`, `llms-full.txt`, or historical plan directory for fork memory. Keep long-lived context here, and keep action-oriented procedures in project skills.

The wiki is organized for quick agent lookup:

- What this fork changed.
- What this fork deleted.
- Which upstream changes should be accepted, adapted, or rejected.
- Which legacy names are still present but do not mean the old product surface should return.

## Maintenance Rules

- Prefer short, durable facts over narrative implementation logs.
- Record why a retained upstream change was accepted, adapted, or rejected when the reason is likely to matter in a future merge.
- Keep deleted upstream specs and planning docs out of the repo; summarize retained decisions here instead.
- Update `change-map.md` for path-level behavior changes, removed surfaces, and intentional divergences from master.
- Update `fork-contract.md` only when the product boundary itself changes.
- Update `upstream-merge-guide.md` when a new recurring merge pattern needs a rule.
- Do not duplicate the same rule in multiple files unless one copy is an entry-point summary and the wiki remains the detailed source.

## Baseline

```text
19659d12 refactor: create local ACP-only Warp fork
```

Treat that commit as the fork baseline. Future merge work should compare upstream changes against the code after this commit.

## Product Shape

The fork keeps:

- Warp terminal GUI.
- Local terminal sessions and retained remote terminal support.
- Warpify for subshell and SSH sessions.
- macOS as the only maintained native host platform.
- ACP agent conversations displayed through Warp AgentView.
- OpenAI-compatible Next Command and Prompt Suggestions.
- Local settings and local persistence.
- ACP protocol event rendering, including generic tool-call updates. MCP server setup is external to Warp and owned by the ACP agent process.

The fork removes:

- Warp account login and access tokens.
- Billing, usage credits, referrals, Teams, cloud workspace discovery.
- Warp Drive cloud sync/sharing UI.
- Warp-hosted Agent SDK/cloud/ambient/scheduled agents.
- Cloud GraphQL APIs, managed secrets, hosted isolation.
- Telemetry, crash reporting, Sentry release upload.
- Onboarding, marketing surfaces, voice input/transcription.
- Linux/Windows native host implementations and packaging.
- Warp app-managed MCP file config, server startup, capability probing, tool/resource execution, permissions, panes, CLI, slash commands, and persistence.
- Warp app-managed bundled/local skills, skill scanners, `/skills` or `/open-skill` UI, and skill-specific actions. ACP agents own skills.

## Current Cleanup Notes

- The app should not keep legacy serde aliases or old wire-format fallbacks for deleted Warp service paths.
- Retained local objects should not regain cloud online/offline gates, sharing permissions, or sync-status UI for local edit/delete behavior.
- Local object pending state can remain for persistence and quit-warning behavior, but cloud sync badges should not return.
- Local object metadata should not keep online-only pending metadata, permissions, untrash, or delete flags.
- Legacy `CloudModel`/`CloudObject`/`ServerId` names may remain only as retained local object model names. New comments and docs should describe local persistence, local object identity, and selector visibility instead of cloud sync or server ownership.
- Warp no longer has an app-level cloud online/offline mode; terminal suggestions should rely on their OpenAI-compatible request path rather than a global Warp cloud status.
- Shared-block cloud sharing UI and settings are removed, including shared-block title generation gates and stale shared-block user-facing copy.
- Local external secret manager references in environment-variable collections are retained only when they call user-installed CLI tools such as 1Password or LastPass from the local shell. Do not confuse this with deleted Warp managed secrets or cloud secret storage.
- ACP client capabilities are only the host capabilities Warp advertises to the ACP agent, such as terminal and file read/write handlers. They are not MCP server capability queries.
- OpenAI-compatible Next Command and Prompt Suggestions are standalone terminal suggestion calls and must not depend on bundled skills, app-managed MCP, or ACP agent skills.
- App-bundled skill resource directories under `resources/bundled/skills/`, `resources/bundled/mcp_skills/`, and `resources/channel-gated-skills/` must remain absent, not merely excluded from bundling scripts.
- Bundled onboarding PNG resources under `app/assets/async/png/onboarding/` are removed with onboarding/marketing flows and should not be restored.
- Local AgentView context syntax such as `<plan:...>` is retained only as ACP prompt context plumbing.
- ACP tool-call diffs may render read-only, but shared-session viewer action sync, cloud action-result replay, resize sync, and input peer-edit sync should not return.
- SSH/subshell Warpify remains retained terminal functionality. Cleanup may update Warpify payload field names to the current schema, but must not remove tmux checks, SSH warpification, subshell bootstrap, or the UI states that keep remote/nested sessions in full Warp mode.
- App/CLI/remote-server Web/WASM compile branches and no-op fallbacks are removed for the macOS-only host target. Remote host platform parsing remains only where SSH/remote-server setup needs it; it is not permission to restore local Linux/Windows packaging or host implementations.
- Local dev SQLite data can be backed up and cleared when it predates the current fork schema. Do not add backward-compatibility migrations solely to preserve stale local fork data.
- Runtime warning cleanup should fix the state boundary that emits the warning. Expected no-context startup states should be gated before execution; stale UI events should carry enough identity to be handled after focus/close ordering changes.

## Wiki Files

- `fork-contract.md`: Detailed product and architecture contract.
- `upstream-merge-guide.md`: Decision process for pulling from original Warp.
- `change-map.md`: Path-level map of added/replaced/removed/retained code.
- `upstream-master-audit-2026-05-10.md`: Historical commit-by-commit audit as of May 10, 2026.
- `upstream-master-audit-2026-05-14.md`: Current audit of `19659d12..master` under ACP-only, macOS-only, no-backward-compatibility rules.
- `upstream-master-audit-2026-05-16.md`: Incremental in-progress audit for commits after `1ca5496d8`; use it together with the 2026-05-14 audit until the new range is fully reviewed.
- `upstream-master-audit-2026-05-17.md`: Incremental audit for `fa732953d..53da56352`, completing the currently fetched upstream master tip.
- `upstream-master-audit-2026-05-18.md`: Incremental audit for `53da56352..24e799977`, completing the currently fetched upstream master tip.
- `upstream-master-audit-2026-05-19.md`: Incremental audit for `24e799977..b37688958`, completing the currently fetched upstream master tip.
- `upstream-master-audit-2026-05-26.md`: Incremental audit for `b37688958..fc110333a`, completing the currently fetched upstream master tip.
- `upstream-master-audit-2026-06-04.md`: Triage audit for `fc110333a..3497d1844` after refreshing local `master` to current upstream `master`.
- `upstream-master-audit-2026-06-07.md`: Incremental audit for `3497d1844..d3757291a`, including retained terminal focus/logging fixes and the fork-specific SSH Warpify automation change.
- `upstream-master-audit-2026-06-12.md`: Incremental audit for `d3757291a..a30cc7a33`, including retained security quoting fixes, terminal/shell fixes, local UI fixes, and rejected cloud/Oz/MCP/skills/native-platform changes.
- `upstream-master-audit-2026-06-13.md`: Incremental audit for `a30cc7a33..c0c6cead9`, including retained code-review routing, local code-editor/settings/UI fixes, path canonicalization cleanup, and rejected cloud/GraphQL/skills/tab-grouping changes.
- `upstream-master-audit-2026-06-16.md`: Incremental audit for `c0c6cead9..d4bb3f5b7`, including retained terminal inline-image, CRLF paste, prompt-suggestion, markdown, Nix grammar, code-review, macOS UI, and dependency-security fixes, plus rejected warpctrl/GraphQL/cloud-credential/skills/tab-grouping changes.
- `upstream-master-audit-2026-06-19.md`: Incremental audit for `d4bb3f5b7..b5d8b48b6`, including retained file-tree, editor, terminal, macOS windowing, markdown-pane, PowerShell bootstrap, and command-signature fixes, plus rejected cloud/Oz/account/Sentry/MCP/skills/tab-grouping/warpctrl changes.
- `upstream-master-audit-2026-06-22.md`: Incremental audit for `b5d8b48b6..8cb48ba94`, including retained terminal mouse-event and integration-test render-loop fixes, plus rejected remote Agent Mode context snapshots, skills/MCP, Cloud Agent continue-locally, Grok/free-AI, cloud tracing, TUI framework, and Windows test changes.
- `upstream-master-audit-2026-06-30.md`: Incremental audit for `8cb48ba94..c0902a246`, including retained terminal/editor/AI-block/shell-integration fixes (file-link period, OSC 1337, Precmd metadata, markdown link clicks, vim paste, requested-command crash, AI-block copy, agent tip, Copy as Markdown, queued-prompt `/fork` bypass), plus rejected cloud-agent/custom-model-router/BYOK/tab-groups/TUI/onboarding/telemetry/MCP/skills/native-Linux-Windows changes.
- `upstream-master-audit-2026-07-04.md`: Incremental audit for `c0902a246..05927696c`, including retained fallback-shell panic guard, wide-char resize crash fix, SSH `RemoteCommand` fallback, markdown highlighting, visual-line navigation, multiline command-prefix fix, conversation-rewind correctness, heap-profile action, settings deeplinks, and PS1 copy, plus rejected cloud-agent/billing/TUI/MCP/skills/managed-secrets/native-platform changes and a deferred terminal-lifecycle stack.
- `upstream-master-audit-2026-07-11.md`: Incremental audit for `05927696c..upstream/master` (98 commits), including retained passwd/getent resolution, cross-window tab drag crash guard, LRC auto-resume, TaskStore IndexMap refactor, macOS CGFont glyph fix, vertical-tab group headers, `/repos` menu flash fix, launch-config tab commands, Copy-current-path action, forked-conversation working directory, markdown-viewer file URL preference, pane-drag width guard, and tools-panel toggles, plus rejected TUI-only, cloud-agent/billing/GraphQL/MCP/skills/managed-secrets/native-platform changes.
- `upstream-master-audit-2026-07-14.md`: Incremental audit for `a01df387a..upstream/master` (27 commits), including retained code-review pinned-file header corner fix, imported PR-comment context-line outdated fix, repo-gated slash-command stale-cache fix, macOS Info.plist EventKit usage descriptions, homebrew non-interactive bootstrap, and the `warp-command-signatures` rev bump, plus rejected TUI-only, orchestration, telemetry, agent_sdk/video-recording, MCP JSON tree, and wasm-specific changes.
- `upstream-master-audit-2026-07-17.md`: Incremental audit for `62da4ee72..upstream/master` (82 commits), including retained WarpUI table click-through selection, GFM autolink trailing-punctuation, agent tool-call banner border gap, TaskStore not-yet-linked-subtask exchange resolution, fullwidth/CJK punctuation link detection, procedural box-drawing glyph renderer, headless local-HTTP-server gating, and warp_cli global-flags-before-subcommand parsing, plus rejected TUI-only, cloud/agent-sdk/orchestration/computer-use/GraphQL/MCP/skills/managed-secrets/native-platform/telemetry changes, and deferred OSC 8 hyperlinks, tab_config menu, command-palette directory exclusion, and Markdown image refresh ports.
- `upstream-master-audit-2026-07-24.md`: Incremental audit for `f1547fefc..upstream/master` (166 commits), including retained dynamic zsh glitch-width prompt stripping, WARP_DATA_PROFILE macOS config-dir scoping, vim d%/c%/y% and gg-count code-editor fixes, resizable-bounds small-window guard, stale classic completions, render-test logging idempotency, and diesel/h2 security bumps, plus rejected TUI-only, computer-use recording, cloud/orchestration/agent-SDK/managed-secrets, onboarding, MCP/skills, telemetry/Sentry (N/A — infrastructure removed), voice input, Linux/Windows/WASM platform, Rust 2024 edition, and feature-flag changes, and a deferred repo-metadata tree-walk cancellation port.
- `upstream-master-audit-2026-07-30.md`: Incremental audit for `940c50594..upstream/master` (112 commits), including retained markdown HTML comment stripping, worktree path quoting, vim indent/dedent operators, code-block file-references-respect-editor, and macOS notarization polling resiliency (adapted to API key auth), plus rejected TUI-only, computer-use, cloud/orchestration/agent-SDK/GEAP, auth/billing/Teams, MCP/skills, telemetry/Sentry (N/A), voice input, Linux/Windows/WASM platform, and eval CI changes, and a deferred expandable-toast port blocked on missing warpui accessibility/button-variant infrastructure.

## Quick Merge Principle

When an upstream commit improves generic terminal behavior, port it.

When an upstream commit improves AI UI or local data structures, adapt it to ACP and local suggestions.

When an upstream commit restores cloud product behavior, reject it or reduce it to a local utility.

Do not resolve conflicts by bringing back deleted modules just because upstream still depends on them.
