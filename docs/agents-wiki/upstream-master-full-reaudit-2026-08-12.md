# Upstream Master Full Re-audit 2026-08-12

## Scope And Result

- Upstream divergence point: `27f4933b81f339f33f8206fd4e9dcb3450ad270a` (`feat(uri): add warposs://pane/{uuid} deep link for pane focus (#9655)`).
- Fixed upstream tip: `69254d73db0c568db55333cad1d3090041cd334a` (`[QUALITY-1333] Harden TUI focus ownership (#14961)`).
- Reviewed range: `27f4933b8..69254d73db`, 1812 first-parent commits.
- Fork product baseline remains `19659d12`; `27f4933b8` is only the requested upstream-history starting point for this audit.

The previous audits used an overly restrictive interpretation of the fork contract. In particular, they treated absence of an existing fork feature flag, prerequisite, or product inventory entry as a reason to reject a new upstream feature. That is incorrect. A feature is retained when its essential behavior can run locally or through a retained provider without Warp-owned services, even if the feature did not exist at the fork baseline.

The full range was indexed by commit, subject, touched paths, prior audit mentions, and current-fork anchors. Of 1812 commits, 1629 were named in an existing audit and 183 were not. Among the 183 unmentioned commits, 164 touched retained or mixed areas and therefore required source inspection rather than automatic rejection. This document records corrected decisions and newly found omissions; unchanged cloud-only, TUI-only, native non-macOS, app-managed MCP/skills, telemetry, and upstream-process decisions remain covered by the incremental audit files.

No feature code was ported during this audit.

## Decision Rules Applied

- The retained-area list is directional, not exhaustive.
- A missing feature flag or prerequisite means follow the upstream ancestry and port the authoritative source stack; it does not make the feature not applicable.
- Size, conflict count, or architectural drift can affect port order, but cannot be the final reason to omit a retained feature.
- Mixed commits keep upstream local UI, state, persistence, and behavior while removing only Warp service, account, cloud sync, telemetry, tracking, or unsupported-platform pieces.
- The exact upstream commits, files, hunks, tests, and assets are the implementation authority. A manual or selective port starts from that source and then makes the smallest fork adaptations. It must not recreate the upstream core implementation from a description, screenshot, observed behavior, or memory.
- Patch identity is insufficient for already-adapted code, so current behavior and call sites were inspected before marking a feature missing or present.

## Corrections To The Reported 22 Features

| # | Feature | Corrected decision | Source/provenance note |
| --- | --- | --- | --- |
| 1 | Tab grouping | **Missing; port** | Start at `fc110333ac` and port the complete upstream grouping history. The prior “feature flag absent / product line not adopted” rationale is invalid. Strip flags and service/telemetry hooks only after applying upstream source. |
| 2 | Tab pinning | **Missing; port with grouping** | Start at `ae7f6574ad` and preserve its upstream dependency on grouping infrastructure. |
| 3 | OSC 8 hyperlinks | **Missing; port** | `4b39aa3163` is retained terminal-protocol behavior. Port the upstream registry, flat-storage, parser, rendering, interaction, and tests as one source stack; reject only upstream specs. |
| 4 | Jupyter `.ipynb` rendering | **Missing; port prerequisite and integration** | `2da530282e` supplies the upstream formatting/parser prerequisite and `13e8b61148` wires the client. Absence of `ipynb_parser` is not a rejection reason. |
| 5 | OSC 7 tab CWD/git branch | **Missing; adapt** | Port `08996b5601` from upstream, then adapt its path boundary to the fork. Do not rewrite the behavior against the fork from scratch. |
| 6 | Code-editor line-number mode | **Missing; port** | `ce73fe07bf`; this was not individually audited before. |
| 7 | AgentView Cmd-Up/Cmd-Down navigation | **Missing; port after its upstream fixes** | `da4da09f86`; size and an upstream-reported keymap issue justify focused verification, not indefinite deferral. |
| 8 | Async Find | **Missing full feature stack; port** | `9c594f729f` is only promotion. The authoritative chain starts at `fb5ad384a2`, followed by `cd745fac9b`, `2566f54af7`, `f2cc205f33`, `ebb792bdb1`, `6c5356ba58`, and `9c594f729f`. |
| 9 | Project Explorer hidden-files toggle | **Missing; port** | `e4695f2199`; local file-tree UI. Release batching was not a product-boundary reason. |
| 10 | Rich Input Ctrl+Enter setting | **Missing; restore upstream base then port** | `eaa936c78d`. The fork later deleted `app/src/terminal/view/use_agent_footer/` even though CLI-agent rich input remains retained. Restore the needed upstream source ancestry rather than inventing a new footer implementation. |
| 11 | Hide macOS Dock icon | **Missing; adapt** | `2dbf25fe44`; keep macOS behavior and omit upstream specs/unsupported-platform settings pieces. |
| 12 | Generic `JsonTreeView` | **Conditional prerequisite, not a standalone missing product feature** | `f0afcc12e8` applies as generic local UI, but currently has no retained consumer. Bring it with an ACP structured tool-call JSON consumer; do not restore app-managed MCP solely to consume it. |
| 13 | Jump to latest Agent message | **Missing; adapt** | `38e45a7565`; keep AgentView navigation and origin plumbing, remove telemetry. |
| 14 | Text-editor auto-save setting | **Missing; port** | `29ed596a0c`; not individually audited before. |
| 15 | Active-command live timer | **Already present; do not re-port** | The behavior from `568ed62089` is present in current command-block rendering and tests; blame reaches the fork baseline cleanup. |
| 16 | `$CDPATH` completion | **Already present; do not re-port** | The behavior from `0ed3663851` is present in bootstrap/session/DCS/completer code and tests. |
| 17 | OSC 52 settings UI and security banner | **Partially present; port missing UI/banner** | Core `Osc52ClipboardAccess` and gate are present. Port the dropdown and blocked-operation banner from `164e60e425`; do not duplicate the core gate. |
| 18 | Scrollable, height-capped new-session menu | **Missing; port** | `51dae19e92`; three conflicts are an adaptation task, not a rejection reason. |
| 19 | Command-palette file-search directory exclusion | **Missing; port both commits** | `3bf0899d57` and `74a0d6758f`; preserve upstream query/result behavior while adapting current path types. |
| 20 | Refresh changed local Markdown images | **Missing; port source pipeline** | `be547674a5`; port the upstream `content_version` pipeline and consumers rather than approximating refresh behavior. |
| 21 | Repo-metadata tree-walk cancellation | **Missing; port prerequisites and cancellation** | `50853a9b92` applies to retained Project Explorer, file search, SSH metadata, and context flows. Current synchronous architecture is not a reason to mark it not applicable. Start with the upstream repo-metadata evolution, including `43828a6d69`. |
| 22 | Fixed-height expandable toasts | **Missing; port prerequisites and feature** | `c20645b9b0`; the missing `warpui::accessibility`/button infrastructure must be sourced from upstream first. Do not substitute a simplified toast implementation. |

## Additional Major Omissions

These were not fully represented in the reported 22-feature list.

### Queued Prompts And Terminal Command Queueing

The current fork still has the old single queued-callback path and lacks the upstream queued-prompts model and panel. This is retained local AgentView behavior.

Port from the upstream source chain beginning with:

- Core panel/model and local interaction: `98af7b654b`, `eadc05e6e9`, `fb8d00b073`, `c6b842fe7a`, `1aa03f9c83`, `86a602b990`, `0aee45df21`, `c4b0829094`.
- Attachments, terminal commands, and notifications: `19018bf4ab`, `e367c9de8b`, `16ab972974`.
- Correctness follow-ups: `53e6cd1933`, `9e3d6826b0`, `5ce6dd2dcd`.
- Inspect `6fe675601c` for shared source, but omit its cloud-mode-only wiring.

The old audit already called parts of this an adapt candidate, but never completed the source port.

### Natural Language Detection Evolution

Natural language detection is explicitly retained, but the fork still carries the old single `bert_tiny` model and `nld_onnx_model` wiring. The upstream v2/v3 model, heuristic, channel, and correctness evolution was lost by treating rollout commits as no-op flags.

The source chain includes `2c1f2042d1`, `fd2a608ace`, `9eef1d25cf`, `127b626bfd`, `48ac96fa20`, `9de6d4dc64`, and later correctness follow-ups such as `06e4b74a43`. Port the actual model assets, classifier code, tests, and wiring; then adapt rollout/channel flags to the fork.

### DCS Hook Session Integrity

The security stack `32d21d15c9`, `ca745b402c`, and `51bd326780` was deferred only because it crossed bootstrap, terminal-model, remote TTY, tmux, and viewer paths. Those are retained local/SSH areas. Port the complete upstream stack, omitting only unsupported local-host platform branches.

### SSH And Remote-Terminal Functionality

- `0d24d2cffa`: reuse a user's existing SSH ControlMaster without taking ownership or killing it.
- `f0ca7861fe`: wait briefly for the remote-server child exit status before presenting an error, avoiding the retained manager race.
- `08487819fe` and `4b5c94d434`: remote git operations and large/binary-file filtering for retained SSH code review and remote file handling.
- `a18da95904`: avoid registering macOS secure storage inside the remote daemon.

These commits are local/SSH functionality. Remote Linux/macOS host code needed by the SSH server is retained even though native Linux/Windows clients are not.

### AgentView And Local AI-Surface Behavior

- `170b791b8f`: conversation-row fork/open/context actions. Keep local actions and persistence; remove cloud share, server calls, debug-only UI, and telemetry.
- `bcfd978737`: restore plain-click link/file tooltips in AgentView.
- `a8df317229`: preserve input focus during local block navigation.
- `d58850b2cc`: fix ask-question custom-answer focus and exit semantics; omit TUI-only hunks.
- `b9d1c0ebdb`: support Droid in the retained local CLI-agent rich-status listener.
- `65381be1f0`: fix Cmd-Enter starting a new AgentView conversation.
- `83c11f155b` and `a90be740b2`: ask-question row alignment and content-sized cards.
- `d9c4c1a70b`: carry a local image into a forked conversation.
- `912e4540f8`: fit-width Mermaid sizing in retained Markdown rendering.

### Terminal, Editor, Window, And Generic UI Fixes

The following source changes are absent from main and remain within retained local behavior:

| Commit | Required port |
| --- | --- |
| `43e3f58cf5` | zsh default bracketed-paste workaround. |
| `c4946001fe` | let the focused tab-config picker own Space. |
| `5c57b38503` | cache SVG rasterization by intrinsic size and fit type. |
| `388f5dc129` | prevent flat-storage `RowIterator` underflow after clear. |
| `f16b26052b` | constrain generic filterable-dropdown empty-state width. |
| `9eadcf9398` | use the decoded git branch name in click handling. |
| `112e842cd0` | show and update code-editor unsaved-change state in the pane header. |
| `a30cc7a331` | bound retained local `ai_queries` history and skip empty entries. |
| `ebe8be8459` | allow system-initiated macOS logout, restart, shutdown, and update termination. |
| `0047677f97` | detect terminal file paths across multiple soft-wrapped rows. |
| `0009e7ca1d` | keep semantic drag selection within word boundaries. |
| `4c91056617` | display Maximize/Minimize pane shortcuts in the pane menu. |
| `97829d5630` | carry tab color into Ctrl+Tab and render the title first. |
| `af29c593b9` | include code-editor panes in Copy Current Path. |
| `5bc232d813` | open a new tab by double-clicking empty vertical-tab space. |
| `d2391bad1d` | local setting to hide title-bar search in vertical-tab layout. |
| `2d799049a2` | resizable long-running-command box. |
| `ed91775398` | reserve correct title-bar space for right-side window controls where the macOS layout uses them. |

`43e3f58cf5`, `c4946001fe`, `5c57b38503`, `388f5dc129`, `f16b26052b`, `9eadcf9398`, `112e842cd0`, `a30cc7a331`, and prerequisite `43828a6d69` have patches that apply cleanly to the current main tree. They should still be ported from their upstream commits and verified individually rather than rewritten.

## Previously Batched Decisions That Must Be Split

### 2026-06-12 22-commit defer batch

The single defer row covering `d2391bad1` through `a30cc7a33` is superseded. It contains retained vertical-tabs UI, tab selection/pinning, AgentView question UI, queued prompts, Markdown, SSH/remote-server, local persistence, and code-review work. Each source commit must receive an individual accept/adapt decision.

High-confidence ports from this row include `d2391bad1`, `a44fbf163`, `2d799049a`, `83c11f155`, `08487819f`, `4b5c94d43`, `19018bf4a`, `3ae6f0821`, `e367c9de8`, `ae7f6574a`, `a90be740b`, `d9c4c1a70`, `16ab97297`, `912e4540f`, `a18da9590`, `0d24d2cff`, `65381be1f`, `5bc232d81`, and `a30cc7a33`. `9c4c656d2`, `26e81f9da`, and `4815c8250` require source-call-site review and adaptation around retained local persistence/git models; server-backed rename or unrelated cloud callers must not be restored.

### 2026-06-04 nine-commit flag batch

The blanket no-op decision is superseded:

- `2566f54af7` belongs to the missing Async Find source stack.
- `c4b0829094` belongs to the missing queued-prompts source stack.
- `9de6d4dc64` and `a3d10ce673` belong to retained NLD evolution.
- `175faadcea` belongs to retained git operations; inspect and port the underlying implementation before applying its promotion semantics.
- `3dc094132a` affects retained remote SSH-server rollout cleanup and requires an individual source review.
- `81d3174246`, `405c83cbae`, and `6616a42eb5` remain applicable only where their eval/test-layout/helper targets survive after the feature source is ported.

Feature promotion commits cannot substitute for their missing base implementation, but they also cannot be used to declare the base feature out of scope.

## Verified Present Or Not Applicable Examples

These checks prevent the broad re-audit from creating duplicate or unrelated work:

- `568ed62089` active-command live duration and `0ed3663851` `$CDPATH` completion are already present.
- Selected-text copy in AI blocks, OSC 1337 malformed-parameter guarding, punctuation-aware terminal links, `/open-file` clearing, inline-code background painting, and equivalent Copy File Path/unsaved-diff semantics are already present through the baseline or adapted merges even where patch identity differs.
- `5b38233b80` and `69254d73db` are TUI-only and remain not applicable because `crates/warp_tui/` is absent.
- `fae2538e1f` only changes the removed MAA passive-suggestion path and remains not applicable.
- Generic `JsonTreeView` is retained-capable infrastructure, but its upstream app-managed MCP consumers remain removed.

## Non-compliant Side-branch Ports

The local branches `merge/upstream-local-2026-08-12-batch2` through `batch13`, `merge/upstream-local-2026-08-12-clean-features`, and `fix/tab-grouping-rendering-from-upstream` are not ancestors of current main. Their commits must not be treated as proof that the omissions are resolved.

At least the tab-grouping foundation commit `183d0aa1b4` describes itself as a “net-effect manual port,” and the branch history contains simplified/reconstructed implementations and conclusions that Async Find and repo-metadata cancellation are not applicable. Those conclusions conflict with this re-audit and the source-fidelity rule. Do not merge these branches as-is. Rebuild each accepted feature from the exact upstream source history, retaining only independently verifiable integration glue where it can be traced back to upstream behavior.

## Source-faithful Implementation Progress

| Feature | Upstream source | Result | Verification |
| --- | --- | --- | --- |
| zsh default bracketed-paste restoration | `43e3f58cf5` | Applied the upstream bootstrap hunk unchanged. | `zsh -n app/assets/bundled/bootstrap/zsh_body.sh`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed the generated build artifacts. |
| SVG rendered-image cache identity and reuse | `5c57b38503` | Applied the upstream cache implementation and regression tests unchanged. | Three upstream `warpui_core` regression tests; `cargo fmt -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed the generated build artifacts. |
| Flat-storage clear/resize row-index underflow | `388f5dc129` | Applied the upstream index invariant fix and regression test unchanged. | Upstream `warp_terminal` regression test; `cargo fmt -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed the generated build artifacts. |

## Port Order

1. Direct, cleanly applicable correctness/security fixes: zsh paste, SVG cache, flat-storage underflow, decoded branch click, unsaved indicator, local history cap, macOS termination, and remote child-exit race.
2. Complete prerequisite stacks: DCS integrity, repo metadata, Async Find, NLD, and Rich Input footer/settings.
3. Major local feature clusters: tab grouping/pinning, queued prompts, OSC 8, OSC 7, and notebook rendering.
4. Remaining AgentView, Project Explorer, editor/settings, Markdown, menu, and toast features.

Each port should use a dedicated upstream-source commit series with provenance recorded per commit. A feature is complete only after upstream tests or faithful fork adaptations cover its retained behavior and deleted-surface scans confirm that Warp services and tracking were not restored.

## Reproducible Checks

```bash
git fetch upstream master
git rev-parse 27f4933b8 upstream/master
git rev-list --count 27f4933b8..upstream/master
git log --reverse --date=short --format='%H%x09%ad%x09%s' 27f4933b8..upstream/master
git show --stat --summary <commit>
git diff <commit>^ <commit> -- <path>
git log --oneline --reverse <feature-parent>..<feature-tip>
git blame <current-path>
```

For an accepted source commit, use `git apply --check` or an isolated cherry-pick to measure applicability. A clean check is evidence about mechanics only; it does not replace product-boundary or call-site review.
