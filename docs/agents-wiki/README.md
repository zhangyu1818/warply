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
- `upstream-master-audit-2026-08-03.md`: Incremental audit for `9dcef6a88..upstream/master` (69 commits), including retained warpui_core cross-window terminal-lag fix (ViewFromStream stale window_id), repo_metadata directory-symlink watcher pruning (adapted to fork's single-arg WatchFilter architecture), app/build.rs path-API refactor, and macOS codesign timestamp-retry, plus rejected TUI-only, computer-use recording, cloud/orchestration/agent-SDK/Grok-OAuth, auth/billing/credits/onboarding, MCP/skills, telemetry (N/A), and Linux/Windows/WASM platform changes.
- `upstream-master-audit-2026-08-05.md`: Incremental audit for `956ae6be4..upstream/master` (42 commits), including retained TerminalModel exit logging, UniformList panic-to-log (adapted to edition 2021 nested if-let), SSH wrapper rc-sourcing fix (`unsetopt ZLE RCS GLOBAL_RCS`), ctrl-/ → US (0x1f) C0 encoding, Kitty keyboard protocol Cmd/Option editing-keys (modifier_param helper, delete escape sequence, IME composition detection), tab-bar background inheritance, synced-inputs indicator in vertical tabs sidebar (adapted without unread-activity refactor), and open-file-from-agent-preview line-range preservation, plus rejected TUI-only, computer-use recording, cloud/orchestration/agent-SDK/billing/credits/onboarding, MCP/skills, auth docs links (no-op in fork), Linux/Windows/WSL platform, and CI/Docker infra changes.
- `upstream-master-audit-2026-08-06.md`: Incremental audit for `c8a166b6c..upstream/master` (22 commits), including retained Tab path completion symlink-follow fix for remote/SSH sessions (`find -L`), command-signatures bump to `4990fa1d` (journalctl + pkill specs, with serde_with 3.19 → 3.21 security bump), and numeric CSS font-weight preservation for pasted rich text (`CustomWeight::from_css_numeric`), plus a deferred TUI shell-completion alignment refactor, and rejected cloud/orchestration/agent-SDK/billing/credits/onboarding, cloud shared-session copy-link, wasm/web, macOS bundled-CLI Dock bouncing (depends on removed oz/oz-dev surface), and Docker CI infra changes.
- `upstream-master-audit-2026-08-08.md`: Incremental audit for `06e4b74a4..upstream/master` (14 commits), including retained macOS fullscreen window-corner fix (`window_corner_radius_for_window`) and bootstrap prebuilt-binary speedup (SHA-256-verified cargo-binstall/cargo-bundle/cargo-about), a `d78ced530` repo_metadata Git-probe change triaged as not applicable (the `tracked_remote_ref`/`git rev-parse @{u}` storm mechanism is absent in this fork), a deferred Agent Mode Cmd-Up/Cmd-Down prompt-navigation port (large, self-reported UI-inert), and rejected TUI/IAP, cloud/billing/Teams/agent-SDK/orchestration, MCP/skills (factory-mcp bundled skill), Grok logo/model-picker (removed `/model` surface), cloud-agent attachment upload, WASM/web, and Docker CI infra changes.
- `upstream-master-audit-2026-08-09.md`: Incremental audit for `d78ced530..upstream/master` (6 commits). No commits ported — all six touch removed surfaces only (TUI focus-stealing, Warp Drive search scoping, Billing & Usage team scoping, cloud environment selector, cloud-agent/ambient-agent/agent-SDK session retention, multi-level orchestration drill-down). Each shared-file edit was verified to have no anchor symbol in the fork (`attach_execution_session*`, `AmbientAgentLiveSessionState`, `orchestration_topology`, `MultiLevelOrchestration` flag, etc.), so they are not applicable rather than cherry-pick candidates.
- `upstream-master-audit-2026-08-11.md`: Incremental audit for `7d93fa468..upstream/master` (22 commits), including retained zero-size resize log-level fix, Find in File Vim-mode click re-activation, code-review renamed-file baseline/diff fix (adapted from `diff_state/local.rs` to `diff_state.rs`), local-command process-group kill on cancellation (adapted to `safe_warn!`), watcher empty-path guard against macOS `CFRelease(NULL)` crash (partial port of the warpctrl file.open fix), plus rejected TUI/orchestration/cloud-run/cloud-shared-session/factory-mcp-skill/winit/build_cache/model-discount/CI-bump changes, and a deferred context-chip active-surface refactor pending the `ShouldRenderCLIAgentToolbar` setting.
- `upstream-master-full-reaudit-2026-08-12.md`: Full re-audit of all 1825 commits in `27f4933b8..5fb3144db9`, including the 13-commit tail after the earlier snapshot, under the local/provider-backed feature-preservation and upstream-source-fidelity rules. It supersedes blanket rejections based on absent feature flags, missing prerequisites, patch size, architectural drift, or workspace-wide edition changes; records the missing feature stacks, the Rust 2024 migration correction, and current remote code-review port boundary.
- `upstream-master-audit-2026-08-13.md`: Settings-surface correction after the local CLI-agent Rich Input ports. It supersedes the earlier page-level rejection of `fe8138bce8` and records the widget-level rule: retain local third-party CLI-agent, editor, code-review, project-explorer, external-editor, and LSP settings while rejecting only their Warp-service/cloud/account/telemetry/MCP/skills/platform dependencies.
- `upstream-master-audit-2026-08-14.md`: Incremental audit for `5fb3144db9..upstream/master` (9 commits), including retained vim-mode editor sweep, FileTreeState Arc-wrap, is_passive AIBlock cache, code-review diff memory bound, EditFiles diff match-failure error enrichment (adapted to `edit_documents.rs`), command-signatures bump to `32a7fd56`, the settings surface port implementation, plus rejected orchestration and not-applicable Code-page split changes.
- `upstream-master-audit-2026-08-15.md`: Incremental audit for `c9e562294..upstream/master` (15 commits), including retained command-signatures bumps, upstream fixes, and the agent-view file explorer chip, plus rejected cloud/agent-SDK changes.
- `upstream-master-audit-2026-08-16.md`: Incremental zero-port audit for `d15645c77..upstream/master` (2 commits): the upstream ProjectContextModel rule-refresh coalescing fix triaged as not applicable (fork's pre-rework model already enforces the invariant via its `pending_updates` queue), and cloud-agent `oz agent run-cloud` spawn flags rejected.
- `upstream-master-audit-2026-08-17.md`: Incremental zero-port audit for `e72fd7aacb..upstream/master` (3 commits), all rejected: bundled Factory files skill (second bundled-skill attempt), billing credits copy, and Teams invite-link GraphQL migration.
- `upstream-master-audit-2026-08-18.md`: Incremental audit for `5071a868ce..upstream/master` (11 commits), including the retained repo_metadata shared gitignore-matcher cache (APP-4828, adapted around the fork's watcher and standing-queries deletions) and the warp-command-signatures bump to `ac69f9b0` (mpv/ruff/deno), plus rejected Factory-skill/MCP-flag/docker/Sentry-action/Teams changes.
- `upstream-master-audit-2026-08-19.md`: Incremental audit for `f466967f03..upstream/master` (12 commits), including the retained warp_completer SignatureCache length cap + bounded FIFO miss cache (APP-5431), the excessive-memory two-poll confirmation (adapted without telemetry/Sentry), and the grid_renderer out-of-bounds report throttle (adapted from `report_error!` to throttled `log::error!`), plus rejected/absent TUI-usage, Ctrl-C shared-viewer cancel (deferred until upstream ships local parity), agent-SDK/MCP, wasm/web, build_cache, and server-API changes.
- `upstream-master-audit-2026-08-20.md`: Incremental audit for `8ba01aa1a8..upstream/master` (18 commits), including the retained double-cursor fix for finished background blocks (CORE-3798), the key-binding responder `{error:#}` log-form sync, the file-outline payload-leak log fix, the pwsh.ps1 `$PWD.Path` cleanup, the AIBlock tooltip no-op-dismissal scroll optimization, and the shared remote `parse_ls_script_output` byte-level parser with non-UTF-8/truncation hardening (adapted without WSL local-host parts), plus rejected/absent Sentry/report_error!-form, MAA recovery-budget, billing-usage, orchestration-flag, repo-head-override, and settings-page changes.
- `upstream-master-audit-2026-08-21.md`: Incremental audit for `4e49d04f5a..upstream/master` (15 commits), including the retained switch-to-tab shortcut hints, lazy UserBlockCompleted expensive fields (warp_util `Lazy`), faster hash maps (`Hashed`, hashbrown/FxHash), AsyncSearcher full-index rebuild coalescing (APP-5389), markdown Rendered/Raw scroll preservation, block-list Paste context menu, empty settings-search category headers, the '#' AI Command Search trigger setting, the zsh compadd `-ld` shim fix, the right-click behavior setting (context menu or paste), and the flex infinite-constraint once-per-run log guard, plus rejected Teams-context, bundled factory-files skill, and billing pricing changes.
- `upstream-master-audit-2026-08-22.md`: Incremental audit for `8936686f2..upstream/master` (14 commits), including the retained IME marked text default-on (`ime_marked_text` in default features), the Shift+right-click explanatory note on the right-click setting, widened deferred-enter test margins, the settings page trailing-element framework (`PageTitle`/`CategoryHeader`), the redundant Alt+1 Project Explorer FixedBinding removal, and the grep tool NUL-delimited parsing fix with BusyBox fallback, plus rejected MCP PATH/agent-SDK/managed-MCP plumbing, OZ env aliases, Warp Agent page rewrite, Oz API-keys rename, team-scoped code indexing, Sentry build path, and usage-footer changes.
- `upstream-master-audit-2026-08-23.md`: Incremental audit for `9e8ba7341..upstream/master` (8 commits), including the retained four latent shell-integration fixes (Span::slice UTF-8 clamp, PowerShell kill-buffer split write, bash/fish `printf '%s'` hex encoder, fish `warp_preexec` generator-job kill) and the warpui_core update/spawn monomorphization reduction (adapted `StoredView` → `Box<dyn AnyView>`), plus rejected multi-team/Teams, cloud-agent env clone, TUI statusline, and not-applicable CodeForge changes.

## Quick Merge Principle

When an upstream commit improves generic terminal behavior, port it.

When an upstream commit improves AI UI or local data structures, adapt it to ACP and local suggestions.

When an upstream settings page mixes retained local controls with removed cloud controls, split the page by widget and runtime ownership. Never reject or restore the whole page solely because of its upstream grouping.

When an upstream commit restores cloud product behavior, reject it or reduce it to a local utility.

Do not resolve conflicts by bringing back deleted modules just because upstream still depends on them.

Do not recreate an accepted upstream feature from descriptions or memory. Apply or copy its exact upstream source first, then make only the fork adaptations required to remove Warp services, tracking, and unsupported host-platform code.
