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
| 1 | Tab grouping | **Ported** | Applied the individually reviewed upstream grouping history beginning at `fc110333ac`, including UI, actions, persistence, drag/drop, colors, menus, keybindings, tests, and correctness follow-ups. Removed only rollout flags and deleted product/platform integrations. |
| 2 | Tab pinning | **Ported with grouping** | Applied the upstream pinning history beginning at `ae7f6574ad`, preserving its grouping dependency, persistence, ordering invariants, vertical/horizontal UI, and cross-window drag behavior. |
| 3 | OSC 8 hyperlinks | **Ported** | Applied `4b39aa3163` as the authoritative source for the registry, flat storage, parser, grid/model routing, hover/click/context-menu/tooltip behavior, and tests. Removed only its rollout gate, upstream specs, and unrelated repository metadata. |
| 4 | Jupyter `.ipynb` rendering | **Ported and enabled directly** | Applied prerequisite `2da530282e` and client integration `13e8b61148`. The fork keeps the parser, formatted editor buffer, data-URI images, local notebook routing, Rendered/Raw toggle, menus, and source tests without the upstream rollout flag. |
| 5 | OSC 7 tab CWD/git branch | **Ported** | Applied `08996b5601` as the implementation authority, including parsing, dedicated CWD events, block/session/AgentView propagation, tab subtitle and prompt-chip refresh, repository detection, and source tests. Adapted only the missing `LocalOrRemotePath` boundary to the fork's retained local `PathBuf` repository API. |
| 6 | Code-editor line-number mode | **Ported** | Applied `ce73fe07bf` with its setting, gutter behavior, repaint path, and source tests; omitted only cloud-sync/telemetry/spec surfaces. |
| 7 | AgentView Cmd-Up/Cmd-Down navigation | **Ported with keymap fix** | Applied `da4da09f86` with its transcript ordering, prompt/user-command filtering, selection cursor, boundary behavior, visible target state, and source tests. Extended the existing select-block bindings to the AgentView rich-input context identified in the upstream commit's verification notes. |
| 8 | Async Find | **Ported and enabled by default** | Applied the implementation from `fb5ad384a2`, AI focus fix `cd745fac9b`, close-bar cleanup `f2cc205f33`, filtered-row parity fix `6c5356ba58`, and Stable enablement `9c594f729f`. The superseded Dogfood/Preview rollout-only changes in `2566f54af7` and `ebb792bdb1` were not used to restore cloud-synced experimental settings or channel UI. |
| 9 | Project Explorer hidden-files toggle | **Ported** | Applied `e4695f2199` and backward-compat/settings follow-up `81d06dae40`, preserving the local file-tree filter, setting, action, macOS keybinding, command-palette state, settings control, and source tests. |
| 10 | Rich Input Ctrl+Enter setting | **Ported** | Applied `eaa936c78d` to the restored upstream Rich Input foundation. Preserved the upstream Enter/Ctrl+Enter editor behavior, local setting, ACP-only settings-page control, unit tests, and integration-test registration; omitted cloud sync and removed feature-flag/non-macOS AgentView keymap dependencies. |
| 11 | Hide macOS Dock icon | **Ported** | Applied `2dbf25fe44` to the retained macOS preference, subscription, settings, delegate, and AppKit bridge; omitted upstream specs and cloud-sync metadata. |
| 12 | Generic `JsonTreeView` | **Conditional prerequisite, not a standalone missing product feature** | `f0afcc12e8` applies as generic local UI, but currently has no retained consumer. Bring it with an ACP structured tool-call JSON consumer; do not restore app-managed MCP solely to consume it. |
| 13 | Jump to latest Agent message | **Ported** | Applied `38e45a7565` with its latest-visible-exchange selection, AgentView entry origin, deferred post-layout scroll, same-view direct scroll, command-palette action, and source tests; removed only rollout and telemetry paths. |
| 14 | Text-editor auto-save setting | **Ported** | Applied `29ed596a0c` with debounced, focus-change, window-change, and close-time save behavior plus the retained local setting. |
| 15 | Active-command live timer | **Already present; do not re-port** | The behavior from `568ed62089` is present in current command-block rendering and tests; blame reaches the fork baseline cleanup. |
| 16 | `$CDPATH` completion | **Already present; do not re-port** | The behavior from `0ed3663851` is present in bootstrap/session/DCS/completer code and tests. |
| 17 | OSC 52 settings UI and security banner | **Ported** | Applied `164e60e425` on top of the existing `Osc52ClipboardAccess` gate, preserving its settings dropdown, read/write-specific blocked-operation banner, allow actions, deduplication, and permanent local suppression. |
| 18 | Scrollable, height-capped new-session menu | **Ported** | Applied `51dae19e92`, preserving the scrollable menu variant, window-height-aware cap, minimum/fallback heights, vertical-tab anchor handling, and source regression test. |
| 19 | Command-palette file-search directory exclusion | **Ported** | Applied `3bf0899d57` and `74a0d6758f`, preserving traversal-time folder exclusion, the file-only query path, and consistent empty/non-empty `files:` palette results. |
| 20 | Refresh changed local Markdown images | **Missing; port source pipeline** | `be547674a5`; port the upstream `content_version` pipeline and consumers rather than approximating refresh behavior. |
| 21 | Repo-metadata tree-walk cancellation | **Ported** | Applied `50853a9b92` after its upstream BFS/COW prerequisites `0f97ef18a7` and `43828a6d69`, plus the required waitable-index state and capped-result prerequisites `689cbce0e9` and `4833187008`. Kept removed cloud indexing, app-managed skills, standing queries, and newer remote-server protocol restructuring absent. |
| 22 | Fixed-height expandable toasts | **Missing; port prerequisites and feature** | `c20645b9b0`; the missing `warpui::accessibility`/button infrastructure must be sourced from upstream first. Do not substitute a simplified toast implementation. |

The completed tab grouping/pinning source ledger is: `fc110333ac`, `f3bfb750bc`, `4f5d0d6f8d`, `98dbf7831e`, `910d0fc467`, `9e23bd22f2`, `662bd73767`, `d3757291a1`, `981cb1c7d0`, `a44fbf1633`, `b24fce3db8`, `e0535ca2cb`, `665f0f6578`, `984a889626`, `ebaef155b3`, `f658c30b57`, `011d9da709`, `ae7f6574ad`, `af532bdc3c`, `d0d3d064da`, `53f273e921`, `2a251933c6`, `1cdb4794e6`, `8794f73251`, `79fdd7cebb`, `8de0888ae2`, `b101722e9b`, `3cdccdc81e`, `fc6260c013`, `034e25bec6`, `79fd190898`, `4441e381c1`, `cd233ebde5`, `ad730534b0`, `0b1e4ab4e5`, `97bc2646dd`, `86289c931d`, `c5d5175f51`, `b802cdf571`, `d275b2dcdf`, `9c59c69df0`, `21e28cccee`, `f1701be39a`, `9dcb9b890c`, `3015d875b7`, `8089a74d3c`, `94804667d0`, and `a612c95919`. After that infrastructure existed, the previously inapplicable grouped-color branch and tests from `f73d44f11b` were restored from that source commit as well.

## Additional Major Omissions

These were not fully represented in the reported 22-feature list.

### Queued Prompts And Terminal Command Queueing

The fork now carries the retained local queued-prompts model, panel, settings, AgentView integration, attachment queueing, terminal-command queueing, and long-running-command drain behavior from the upstream source chain:

- Core panel/model and local interaction: `98af7b654b`, `eadc05e6e9`, `fb8d00b073`, `c6b842fe7a`, `1aa03f9c83`, `86a602b990`, `0aee45df21`, `c4b0829094`.
- Attachments, terminal commands, and notifications: `19018bf4ab`, `e367c9de8b`, `16ab972974`.
- Correctness follow-ups: `53e6cd1933`, `9e3d6826b0`, `5ce6dd2dcd`.
- Shared-source follow-ups: `6fe675601c`, `099855bfe8`, `2d8587373d`.

The port keeps the final upstream local queue model and tests as its authority. It removes only cloud-mode locked rows, shared-session/cloud-agent behavior, telemetry, rollout gates, and upstream specs. ACP submission remains the fork's backend boundary.

### Natural Language Detection Evolution

Natural language detection is explicitly retained. The fork now carries the upstream v1/v2/v3 ONNX model stack, heuristic v1/v2 selection, and v3/v2 macOS bundle configuration instead of the old single `bert_tiny` model and `nld_onnx_model` wiring.

The completed source chain includes `2c1f2042d1`, `fd2a608ace`, `127b626bfd`, `48ac96fa20`, `5067be3c7b`, `9de6d4dc64`, `9093f116f`, `584b5e453b`, `8a30a37ff2`, and `06e4b74a43`. The superseded rollout revert `9eef1d25cf` and telemetry-only `7f4e111361` were not applied.

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
| Git branch click payload decoding | `9eadcf9398` | Applied the upstream fallback decoding fix and regression test unchanged. | Upstream `warp` regression test; `cargo fmt -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed the generated build artifacts. |
| Filterable dropdown empty-state width | `f16b26052b` | Applied the upstream generic dropdown width fix unchanged; retained local consumers set explicit menu widths. | `cargo fmt -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed the generated build artifacts. |
| Single-tab code-editor unsaved indicator | `112e842cd0` | Applied the upstream header invalidation and unsaved-dot rendering fix unchanged. | `cargo fmt -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed the generated build artifacts. |
| Local AI-query persistence cap | `a30cc7a331` | Applied the upstream FIFO cap, empty-input skip, and regression tests. Omitted the removed ambient/cloud-session guard and its deleted `intended_agent` test field. | Four upstream `warp` regression tests; `cargo fmt -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed the generated build artifacts. |
| macOS system-initiated termination | `ebe8be8459` | Applied the upstream Apple Event detection and termination-source plumbing to retained macOS/headless paths. Kept winit deleted and omitted removed telemetry/autoupdate paths. | 308 `warpui_core`/`warpui` tests passed with 8 existing skips; `cargo fmt -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed the generated build artifacts. |
| SSH child-exit status race | `e010a6bcd0`, `f0ca7861fe` | Applied the upstream cancellation classification, failure-banner suppression, exit-wait state, and 200 ms status timeout to retained SSH paths. Omitted the telemetry consumer. | 54 `remote_server` tests; `cargo fmt -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed the generated build artifacts. |
| Soft-wrapped terminal file-path detection | `0047677f97` | Applied the upstream multi-row scan, filesystem-sized budget, fragment cap, and regression tests unchanged; mapped the upstream plural test path to the fork's existing singular test file. | 16 focused `warp` path-detection tests; `cargo fmt -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed the generated build artifacts. |
| Semantic drag-selection boundaries | `0009e7ca1d` | Applied the upstream semantic-expansion algorithm and regression tests; mapped the upstream `elements/gui/` paths to the fork's flat element layout while preserving the fork's newer hyperlink-click regression test. The upstream `TextFrame::mock_with_positions` prerequisite was already present. | Three focused `warpui_core` regression tests; `cargo fmt`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 12.0 GiB of generated build artifacts. |
| Configurable code-editor line numbers | `ce73fe07bf` | Applied the upstream absolute/relative line-number setting, settings dropdown, gutter rendering, repaint behavior, unit tests, and integration-test coverage. Omitted upstream `specs/**`, the removed cloud-sync field/icon, and telemetry wiring; used the fork's existing local settings error handler. | Six focused `warp` unit tests; `cargo fmt`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 11.1 GiB of generated build artifacts. |
| Text-editor auto save | `29ed596a0c` | Applied the upstream debounced typing save, focus/window-change save, silent close-time flush, toast/unsaved-indicator behavior, local setting, and settings toggle. Kept deleted WASM and the obsolete standalone Code settings page removed, mapped the toggle to the fork's retained Text Editing page, omitted cloud-sync/telemetry code, and adapted saveability to the fork's existing file-id metadata model. | `cargo fmt`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 11.1 GiB of generated build artifacts. Upstream supplied no automated tests for this lifecycle-driven feature. |
| Project Explorer hidden-files toggle | `e4695f2199`, `81d06dae40` | Applied the upstream dotfile filter, reactive `CodeSettings` subscription, stale-selection clearing, local action and macOS shortcut, backward-compatible default, settings switch, command-palette flag, and source tests. Omitted cloud-sync metadata and unrelated non-macOS font-shortcut changes, kept deleted account/team test clients and the obsolete standalone Code settings page absent, and mapped the source switch to the fork's existing Text Editing settings page. The source tests were adapted only to await the fork's newer asynchronous repository-index pipeline. | Three focused upstream-derived file-tree tests passed; `cargo check -p warp --all-targets --message-format short`; `cargo fmt --all -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 16.2 GiB of generated build artifacts. |
| macOS Dock icon visibility | `2dbf25fe4` | Applied the upstream launch-time preference, runtime setting subscription, Appearance toggle, platform delegate, and AppKit activation-policy bridge. Omitted upstream `specs/**` and cloud-sync UI metadata, read from the fork's retained TOML preference store, and mapped the newer upstream `objc2` delegate call to the fork's existing `cocoa/objc` bridge without changing the selector or main-queue behavior. | `cargo fmt`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 11.1 GiB of generated build artifacts. |
| DCS hook session integrity | `32d21d15c9`, `ca745b402c`, `51bd326780` | Applied the upstream client-generated non-zero session IDs, bootstrap injection, registered-session validation, SSH/tmux interpolation, and viewer replay bypass. Omitted removed Windows/MSYS2/WSL, `FinishUpdate`, Web PTY, and cloud shared-session paths; mapped the viewer bypass to the retained local conversation transcript viewer. | Shell syntax checks; focused DCS, Warpify, and viewer tests; `cargo fmt`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 11.1 GiB of generated build artifacts. |
| Repo-metadata BFS partial trees | `0f97ef18a7` | Applied the upstream breadth-first file-tree builder, partial lazy remainder, explicit stop/fail-fast budget modes, local Project Explorer behavior, file-outline and project-rules consumers, and non-skill regression tests. Kept deleted full-source cloud AI indexing absent and removed the upstream app-managed skill path-interest branch. | `cargo check -p repo_metadata --features local_fs,test-util --all-targets`; six focused upstream-derived BFS/budget tests; `cargo fmt`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 12.5 GiB of generated build artifacts. |
| Repo-metadata COW file-tree snapshots | `43828a6d69` | Applied the upstream `Arc`-backed `FileTreeEntry` storage and `Arc::make_mut` mutation paths unchanged, preserving isolated model/view snapshots while avoiding deep clones during view updates. | 55 `repo_metadata` tests passed with 3 existing skips; `cargo fmt -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 12.1 GiB of generated build artifacts. |
| Repo-metadata cancellable tree walks | `689cbce0e9`, `4833187008`, `50853a9b92` | Applied the upstream waitable pending-index state, explicit capped-result errors, async filesystem traversal, owning-repository task tracking, stale callback rejection, teardown cancellation, and coalesced directory-load completion. Mapped completion replies to the fork's retained direct SSH message dispatcher and kept deleted cloud full-source indexing, app-managed skills/standing queries, and unrelated newer remote protocol wrappers absent. | 67 `repo_metadata` tests passed with 3 existing skips; 15 focused Warp file-tree tests passed; `cargo check -p warp --all-targets`; `cargo fmt -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 14.9 GiB of generated build artifacts. |
| Asynchronous terminal find | `fb5ad384a2`, `cd745fac9b`, `f2cc205f33`, `6c5356ba58`, `9c594f729f` | Applied the upstream background work queue, chunked grid scans, absolute-coordinate results, dirty-range invalidation, streamed focus/navigation, AI match focus, close-bar clearing, hidden-row filtering, render integration, and Stable default feature bridge. Mapped the upstream AI close-bar regression tests to the fork's singular terminal view test module. Omitted upstream `specs/**` and the superseded Dogfood/Preview cloud-synced toggle from `2566f54af7`/`ebb792bdb1`. | 23 focused upstream-derived tests passed; `cargo check -p warp --all-targets`; `cargo fmt -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 19.6 GiB of generated build artifacts. The upstream block-filter integration runner compiled but could not start its assertions because the UI driver rendered no frames. |
| Natural-language detection v3 | `2c1f2042d1`, `fd2a608ace`, `127b626bfd`, `48ac96fa20`, `5067be3c7b`, `9de6d4dc64`, `9093f116f`, `584b5e453b`, `8a30a37ff2`, `06e4b74a43` | Applied the upstream FastText removal, Git LFS bootstrap/CI safeguards, exact v1/v2/v3 model objects, classifier selection, heuristic v1/v2 implementation and tests, evaluation binary, final v3 model plus heuristic-v2 macOS bundle configuration, and later command-keyword corrections. Preserved the fork's existing CJK classifier path and direct `InputType` interface. Omitted the superseded rollout revert, telemetry-only decision-source plumbing, deleted native Linux/Windows bundle scripts, and cloud channel/settings wiring. | `git lfs fsck`; shell syntax checks; all-target/all-feature `input_classifier` check; 11 `input_classifier` tests; real v3 ONNX inference through `evaluate`; `cargo fmt`; `cargo build -p warp --all-targets --features nld_classifier_v3,nld_heuristic_v2 --message-format short`; `cargo clean` removed 14.2 GiB of generated build artifacts. |
| CLI-agent Rich Input foundation | `0dbd3d567a`, `34f3adc2ac`, `96fc2046b2`, `4aea06734e` | Restored the upstream local Rich Input state machine, CLI footer/chip configuration, editor rendering, draft lifecycle, Ctrl-G entrypoint, per-agent PTY submission strategies, image attachment flow, code-review/file routing, and source tests deleted by fork commit `141123c3bd`. Split the CLI-only footer host from the removed Warp “Use Agent” product surface and kept the existing standalone Warpify footer. | 20 focused upstream Rich Input tests; `cargo check -p warp --all-targets --message-format short`; `cargo fmt`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 14.6 GiB of generated build artifacts. |
| Rich Input Ctrl+Enter submission | `eaa936c78d` | Applied the upstream local setting, editor enter-key configuration, shared submit path, runtime setting refresh, ACP-only settings-page toggle, unit tests, and integration-test registration to the restored Rich Input foundation. Removed only cloud-sync metadata, deleted rollout flags, and the absent non-macOS AgentView Ctrl+Enter binding; retained source behavior and comments. | Nine focused upstream-derived `warp` tests passed; `cargo check -p warp --all-targets`; `cargo check -p integration --all-targets`; integration UI tests compiled but the runner stopped before assertions because no window frames rendered; `cargo fmt --all -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 21.4 GiB of generated build artifacts. |
| Tab grouping and pinning | 48 individually reviewed commits from `fc110333ac` through `a612c95919`, plus the now-applicable grouped-color branch from `f73d44f11b` | Applied the upstream grouping/pinning source stack: SQLite snapshots and migrations, group/pin state and ordering, horizontal and vertical rendering, group rename/color/collapse/menu actions, tab and pane drag/drop including cross-window transfer, macOS window-drag guard, keybindings, assets, and source tests. Removed the `GroupedTabs`/`PinnedTabs` rollout gates so the retained local feature is directly available. Omitted only telemetry, cloud shared-session behavior, removed Agent/Oz assets, native Windows/winit integration, and upstream specs; no core behavior was reconstructed. | 21 focused upstream-derived tests passed, covering canonical and grouped color cycling, persistence, tab-group headers, group lifecycle/pinning, vertical-tab focus, new-tab placement, and cross-window drag invariants; `cargo fmt --all`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 11.2 GiB of generated build artifacts. |
| OSC 8 terminal hyperlinks | `4b39aa3163` | Applied the upstream OSC parameter parser, bounded per-grid hyperlink registry, flat-storage hyperlink attribute map, cell stamping and lifecycle resets, block/alternate-screen/model lookup, hover precedence, Cmd-click/action opening, context-menu copy/open behavior, URI tooltip, and all source tests. The feature is directly enabled in the fork, so only the upstream `OscHyperlinks` build/channel flag and its disabled-state tests were removed. Upstream `specs/GH6393/**` and an unrelated `.gitignore` entry were omitted; test macro/type paths were mapped to the fork's current `warpui` re-export. | 22 focused `warp_terminal` tests and 10 focused `warp` tests passed; `cargo check -p warp --all-targets`; `cargo check -p integration --all-targets`; all five source integration tests compiled and registered, but the local UI driver stopped each after terminal bootstrap because no frame rendered before the first command assertion; `cargo fmt --all -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 22.6 GiB of generated build artifacts. |
| Jupyter `.ipynb` rendering | `2da530282e`, `13e8b61148` | Applied the upstream v4 notebook parser and raw fallback, editor `Ipynb` content format, bounded base64 data-URI image handling, notebook-model reset path, local file routing, Rendered/Raw toggle, code-editor and terminal menus, open-in-Warp banner behavior, tooltip suppression, and source tests. The fork enables this retained local feature directly, so the upstream `JupyterNotebookRendering` rollout flag was removed. Upstream remote-file-tree branches whose `LocalOrRemotePath` opening entry point is absent were not recreated, and removed cloud/telemetry/WASM dependency chains were not restored. | 24 `ipynb_parser` tests, six `asset_cache` tests, the focused editor data-URI fallback test, the `warp_util` file-type test, and three `warp` routing tests passed; `cargo check -p warp --all-targets`; `cargo fmt --all -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 16.9 GiB of generated build artifacts. The broader editor suite had 411 passes and five pre-existing Markdown/URL round-trip assertion failures outside the added parser/data-URI branches. |
| OSC 7 working-directory propagation | `08996b5601` | Applied the upstream strict local-host OSC 7 parser, percent-decoding rules, dedicated `BlockWorkingDirectoryUpdated` event, per-block CWD updates, terminal/session/AgentView propagation, tab subtitle and prompt-chip refresh, and repository-detection trigger. Preserved the upstream SSH/Warpify spoofing guard and the invariant that OSC 7 neither emits `BlockMetadataReceived` nor drains command-finish callbacks. Adapted the upstream `LocalOrRemotePath` call to the fork's retained local `PathBuf` repository detector without changing the source event flow. | 19 OSC 7 parser tests, the block CWD event test, and the command-finish callback isolation test passed; `cargo check -p warp --all-targets`; `cargo check -p integration --all-targets`; `cargo fmt --all`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 15.5 GiB of generated build artifacts. |
| Queued prompts and terminal command queueing | `98af7b654b`, `eadc05e6e9`, `6fe675601c`, `fb8d00b073`, `c6b842fe7a`, `1aa03f9c83`, `86a602b990`, `0aee45df21`, `099855bfe8`, `19018bf4ab`, `e367c9de8b`, `53e6cd1933`, `16ab972974`, `5a35550d38`, `c29cf0fde6`, `2d8587373d`, `9e3d6826b0`, `5ce6dd2dcd` | Applied the upstream conversation-scoped queue model, editable/reorderable panel, prompt-submission and long-running-command settings, status-bar and command-palette controls, attachment preservation, terminal-command rows, in-flight drain rules, and conversation lifecycle cleanup. Kept the final upstream model and test suite as the source baseline; removed only cloud-mode locked rows, shared-session/cloud-agent behavior, telemetry, rollout flags, and upstream specs, while routing sends through the retained ACP controller. | 41 focused queued-query/panel tests and three slash-command integration tests passed; `cargo check -p warp --all-targets`; `cargo fmt --all`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 21.8 GiB of generated build artifacts. |
| AgentView transcript navigation | `da4da09f86` | Applied the upstream chronological Cmd-Up/Cmd-Down cursor across user prompts and user-executed shell commands, skipping Agent-requested/monitored commands and response-only AI blocks. Preserved end-of-transcript/no-cursor behavior, target lifecycle, scrolling, and source tests. Removed rollout and telemetry wiring; mapped transcript visibility to the fork's `AgentViewState`, exchange lookup to retained rich-content metadata, and the upstream avatar ring to the retained avatar-free prompt container. Extended the existing select-block context predicate to `Input + ACTIVE_AGENT_VIEW`, fixing the keymap reachability issue documented by the upstream commit itself. | Seven focused upstream-derived model/view tests passed; `cargo check -p warp --all-targets`; `cargo fmt --all`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 19.3 GiB of generated build artifacts. |
| Jump to latest Agent message | `38e45a7565` | Applied the upstream activity-aware conversation/exchange selection, latest-visible-exchange targeting, one-shot post-layout scroll after AgentView entry, same-conversation direct scroll, entry-origin state, command-palette action, and source tests. Removed the absent AgentView rollout gate, telemetry event/mapping, and the corresponding disabled-flag test; retained the complete local navigation path and mapped the source test fixture to the fork's current conversation constructor and deleted query fields. | Four focused upstream-derived tests passed; `cargo check -p warp --all-targets --message-format short`; `cargo fmt --all -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 13.8 GiB of generated build artifacts. |
| OSC 52 settings UI and security banner | `164e60e425` | Applied the upstream settings dropdown, access labels, alert banner, read/write-specific messaging and allow actions, same-operation deduplication, temporary dismissal, and permanently persisted local suppression on top of the fork's existing OSC 52 gate. Reused the retained banner/dropdown UI and current local-settings error path. Omitted only the source commit's telemetry action mapping, cloud-synced setting metadata/icon, and unrelated non-macOS Emacs banner branch. | Five focused OSC 52 setting/gate tests passed; `cargo check -p warp --all-targets --message-format short`; `cargo fmt --all -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 13.8 GiB of generated build artifacts. |
| Scrollable, height-capped new-session menu | `51dae19e92` | Applied the upstream scrollable `MenuVariant`, extracted shared menu geometry constants, computed the maximum height from the platform window and anchor position with upstream minimum/fallback behavior, preserved vertical-tab anchor correction, and set the height at menu open time. Adapted only the source test module path to the fork's `workspace/view_test.rs` and kept the fork's current imports/product surfaces. | Three focused workspace menu tests passed, including `test_new_session_menu_is_capped_to_window_height`; `cargo check -p warp --all-targets --message-format short`; `cargo fmt --all -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 13.8 GiB of generated build artifacts. |
| Command-palette file-search directory exclusion | `3bf0899d57`, `74a0d6758f` | Applied the upstream file-only repository traversal with `GetContentsArgs::exclude_folders()`, shared include-folders/query filtering, uncached file-only queries, file-only Command Palette routing, and zero-state/fuzzy directory exclusion. Adapted the source's `LocalOrRemotePath` implementation to the fork's retained local repository path API and imported the fork's `QueryFilter` path in the source regression test; did not alter directory-inclusive callers. | 17 focused file-search/model tests passed, including the upstream-derived folder-option and `files:` palette regression tests; `cargo check -p warp --all-targets --message-format short`; `cargo fmt --all -- --check`; `cargo build -p warp --all-targets --message-format short`; `cargo clean` removed 13.8 GiB of generated build artifacts. |

## Port Order

1. Direct, cleanly applicable correctness/security fixes: zsh paste, SVG cache, flat-storage underflow, decoded branch click, unsaved indicator, local history cap, macOS termination, and remote child-exit race.
2. Complete prerequisite stacks: DCS integrity, repo metadata, Async Find, NLD, and Rich Input footer/settings.
3. Completed major local feature cluster: queued prompts.
4. Remaining editor/settings, Markdown, menu, and toast features.

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
