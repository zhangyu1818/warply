# Upstream Master Audit 2026-06-30

Range under review: `8cb48ba94..c0902a246`

Previous audited upstream tip: `8cb48ba94 Reduce flakiness of windows local_tty test. (#12881)`

Current upstream tip detected: `c0902a246 QUALITY-919: register orchestrators for child events on wait_for_events (#13208)`

Total upstream commits in this incremental range: 100

Status: triage complete. Retained terminal, editor, AI-block, code-review, shell-integration, and AgentView fixes were ported or adapted manually. Cloud agent, custom model router, BYOK, tab-grouping, TUI, onboarding, telemetry, MCP/skills, Cloud Agent continue-locally, Grok/free-AI, native Linux/Windows, and shared-session CRDT changes were rejected or marked not applicable under the fork contract.

## Ported Or Adapted

- `f82b16dfc` Ported the terminal file-link trailing-period exclusion so sentence-ending periods are not absorbed into detected file paths.
- `6b22b5682` Ported the conversation-list hover tooltip fix so the tooltip no longer leaks through modals.
- `1f3a628d6` Ported the inline-code link underline fix. Background/border painting is split out and painted before glyphs so link underlines on backgrounded runs (inline-code links) are no longer hidden.
- `54e3c3fa2` Ported the OSC 1337 panic guard. A bare OSC 1337 with no second parameter no longer indexes out of bounds.
- `16c327350` Adapted the Precmd completion-metadata shell-integration enhancement. Each shell now allocates the next block ID once and reuses the exit-code/next-block-id pair in both `CommandFinished` and `Precmd`. The upstream MSYS2 key-value emission branches were skipped because this fork is macOS-only; only the POSIX JSON path is retained, and the DProtoHook decoder accepts and ignores the two new fields.
- `5fba1fcc8` Ported the `/open-file` argument-clearing fix so clearing the argument no longer re-inserts a lone file.
- `38e765a09` Adapted the queued-prompt help-panel fix. Added the `QueuedPromptInlineEditorOpen` context guard to the `shift-?` binding. The upstream test depends on removed `FeatureFlag::AgentView`/`QueueSlashCommand` overrides, so only the binding guard was ported.
- `27512e689` Ported the markdown link click fix in AI block output. A plain click on a markdown link inside a `SelectableArea` is now reported as handled so the selection handler does not clobber it; the matching release is handled too. Also added the `TextFrame::mock_with_positions` test helper.
- `a0f42752c` Adapted the vim visual paste fix. Visual-mode tail anchors are preserved across the ephemeral-buffer snapshot/restore path. The upstream `ephemeral_is_display_only` branch was not ported because this fork does not carry that `buffer_and_display_map` field; the regular ephemeral snapshot path is kept directly.
- `0183214fe` Adapted the requested-command crash fix. Scoped the approval card key bindings to a new `RequestedActionBlocked` context and downgraded the late-acceptance `debug_assert!` to `log::warn!`. The upstream `requested_mcp_tools` accept/reject card handling was not ported because app-managed MCP tool calls are removed.
- `47300377a` Ported the AI-block selection/copy fix. The upstream `view_tests.rs` fixture was not added because it references removed cloud-agent/CLI-agent harness types; the retained `view_test.rs` module remains the in-tree test path.
- `0bfb81f0d` Ported the agent-tip-below-warping-indicator fix. The warping footer now reserves room for the secondary element line so the clip no longer hides it.
- `c96a018ad` Adapted the Copy-as-Markdown AI document action. The upstream `CreateWarpDriveNotebook`, `CopyLink`, `ShowInWarpDrive`, and `ObjectLinkCopied` telemetry paths were not ported; only the local Copy as Markdown action and its integration test were retained.
- `509ecc7e7` Adapted the queued-prompt `/fork` bypass. Action-emitting slash commands now bypass prompt queuing. Upstream introduced a `slash_command_is_submitted_as_prompt` classifier for `/compact`/`/plan`/`/orchestrate`, but this fork does not carry those commands, so the classifier collapses to "queue `/queue`'s argument, run every other slash command now".

## Already Present In Fork

- `097901980` The agent toolbelt chip theme-change rebuild was already present in the fork (`has_items`, `AppearanceEvent::ThemeChanged` subscribe in `editor.rs`/`editor_modal.rs`).
- `59047093b` Reverts `67f0c832a` (code-review rounded-corner clip). The fork never imported `67f0c832a`, so the revert is a no-op.
- `67f0c832a` Code-review file-header rounded-corner clip fix. Net effect cancelled by `59047093b` in the same range; the fork keeps the pre-existing code-review header layout.
- `7f0a121cf` SSH Warpify settings unification. The fork already uses `enable_ssh_warpification` as the single source of truth and removed `SshSettings::enable_ssh_wrapper`.
- `866c0bc93` Prompt-cache expiry notification dot. The fork does not carry the prompt-cache-expiry chip surface.
- `160b6c503` Remove Full Terminal Agent model callout. The fork already removed the model-callout/model-selector surface.

## Rejected Or Not Applicable

| Commit | Decision | Reason |
| --- | --- | --- |
| `c0902a246` | Reject | Registers orchestrators for child events on `wait_for_events`; depends on removed orchestration/cloud-agent infrastructure. |
| `b7ac39637` | Reject | Wraps orchestrate-card agent chips; depends on removed orchestration controls. |
| `bede4ffa4` | Not applicable | Downranks Intel UHD 770 on Windows; macOS-only host. |
| `517d7a915` | Reject | Hooks up input view with TUI app; this fork has no TUI app target. |
| `358bbd227` | Reject | Statically compiles CLI/warpctl on Linux; macOS-only host, no Linux packaging. |
| `ec77c9a72` | Reject | Fixes WASM crash in conversation search tool calls; WASM target removed. |
| `1fb30ea1c` | Reject | Claude preflight task-status fix in `agent_sdk/driver.rs`; cloud agent/Agent SDK removed. |
| `14f84c17e` | Reject | Downgrades cross-window tab drag to preview; tab-group feature flag absent. |
| `b484e4fdd` | Reject | Adds memory-store CLI commands in `agent_sdk`; Agent SDK removed. |
| `73834e56f` | Not applicable | Faster hasher for EntityId maps across TUI/warpui_core test files; TUI crate absent and the warpui_core changes are test-only import updates that do not affect macOS behavior. |
| `fd949405c` | Not applicable | Matches `/fork-and-compact` pane options to `/fork`; fork has no `/fork-and-compact`. |
| `148c81179` | Reject | Conversation streaming for TUI; TUI target and cloud-agent/Agent SDK paths removed. |
| `185862d80` | Reject | Clarifies Oz/warpctrl CLI install labels; Warp Control CLI removed. |
| `b802cdf57`, `c5d5175f5`, `97bc2646d`, `ad730534b`, `d45528ad2`, `cd233ebde`, `4441e381c`, `034e25bec`, `35086b22f`, `fc6260c01`, `3cdccdc81`, `79fd19089`, `0b1e4ab4e`, `86289c931`, `d275b2dcd` | Reject/Not applicable | Tab-group/pinning feature work; the fork has no `TabGroups`/`Pinning` feature flag or tab-grouping module. |
| `bc9cd02be` | Accept (already done) | `LSHandlerRank=Alternate` plist fix for macOS file-type claims. Already handled by the fork's `script/update_plist`. |
| `118c6a4ef` | Adapted | See Ported section above. |
| `11742b32c`, `7a60bb9bf`, `fc7739026`, `608bc532e`, `ce1a91c52`, `517d7a915` | Reject | TUI crate and ratatui-backed presenter/element library; this fork has no TUI target. |
| `b15bdd3a0` | Reject | Makes TerminalManager view-agnostic for TUI; TUI target absent and the change restructures terminal-manager ownership around removed TUI/shared-session paths. |
| `ee13ae5e9` | Reject | Buffers shared-session input updates; shared-session viewer removed. |
| `2da530282` | Reject | Adds ipynb formatting buffer dependency (`ipynb_parser` crate) for cloud notebook editing; not a retained fork feature. |
| `eaacdf502` | Reject | Extracts launch profile config for app and TUI; TUI target absent. |
| `a59c4b8a9` | Reject | MCP server config secret-redaction toggle; app-managed MCP settings removed. |
| `73aea9eea` | Reject | Bumps `warp_graphql_schema` js-yaml; GraphQL schema removed. |
| `98541694f` | Not applicable | Re-adds Metal Toolchain download in CI; the fork CI treats this step as optional already. |
| `b349964ea`, `d3d0b95fd`, `dbde9bcda`, `563682edb`, `8af2dbadd` | Reject | Onboarding/auth/login rework; onboarding, account auth, and login UI removed. |
| `9fd0e8874` | Reject | UI for configuring custom model routers; custom model router and old model selector removed. |
| `f554cf762`, `d4ad60368`, `dd6eb69fb`, `0959104f3`, `c2b51ab6c`, `535df012a` | Reject | Custom model router feature, BYOK default-model suggestion, and inline model picker; old Warp model/BYOK surfaces removed. |
| `f4bcdadb4` | Reject | `--runner` flag and `runner_id` plumbing for ambient agents; Agent SDK/ambient agents removed. |
| `abf98bffd` | Reject | Clears permission state when blocking tool completes; depends on removed shared-session/MCP tool-call card path. |
| `5dd23f5fb` | Reject | Voice input stream-error Sentry reporting; voice input and Sentry removed. |
| `45e82234d` | Reject | CRDT input-buffer display-only ephemeral state for shared-session viewers; shared-session viewer and the `ephemeral_is_display_only` field are not retained. |
| `d4997e6c3` | Reject | Extracts `warp_multi_agent_client` crate; cloud multi-agent client removed. |
| `eaa076c32` | Accept (dependency) | Bumps `mermaid_to_svg` upstream rev. Reviewed as a generic dependency bump with no cloud/skills/MCP reintroduction. |
| `68ed7382d` | Not applicable | Vertical-tabs Summary-mode PR chip click; the fork has no PR-chip surface. |
| `109ca0c95` | Reject | Removes reset-grid assertion on conversation restore for cloud conversation restore paths. |
| `3ca45befb` | Reject | Configures git credentials with GitHub inside `agent_sdk`; Agent SDK removed. |
| `3cdccdc81` | Reject | Cross-window drag + pinned tabs; pinning/tab-groups absent. |
| `b82e90c98` | Reject | Codex plugin installation; Codex plugin/CLI-agent plugin manager removed. |
| `df89509e3` | Reject | Prevents 0-byte corrupted installer in auto-update; upstream `autoupdate` removed (fork uses Sparkle 2). |
| `0d83295b8` | Reject | Cache-TTL dev-only usage display; cloud usage/context-window dev surface removed. |
| `849d6cfed` | Reject | Removes strong model handle captures in `agent_sdk`/ambient-agent subscriptions; Agent SDK removed. |
| `a96de307f` | Reject | Upstream spec doc for terminal block lifecycle hardening; specs rejected. |
| `a6e122e4c` | Reject | Fixes ctrl-c not cancelling stream in `blocklist/controller`; depends on removed cloud-agent/streaming controller structure. |
| `9eb49f5fa` | Reject | Fixes input disappearing on sharing completion; shared-session sharing removed. |
| `18e6968af` | Reject | BYOK/BYOE enterprise product spec; specs rejected. |
| `6691e1e0e` | Reject | Searchable agent picker in New API key modal; platform API-key modal removed. |
| `71d95da2f` | Reject | Public Claude platform plugin in CLI-agent plugin manager; CLI-agent plugin manager removed. |
| `620e04a60`, `c13824b77` | Reject | Release workflow inefficiencies/revert touching Linux/Windows bundle scripts; macOS-only packaging. |
| `4f931961d` | Reject | Makes `BaseClient` a concrete struct for cloud server API; cloud server API removed. |
| `50c542564` | Reject | Decouples `IapManager` from the warp crate; IAP/billing removed. |
| `a237521d7` | Reject | Dev-only context-window segment breakdown in usage card; cloud usage card removed. |
| `8006b6ee3` | Reject | Dismissible cloud credits banner; billing/credits removed. |
| `38ba615b7` | Reject | Connected labels in orchestration host picker; orchestration removed. |
| `0974a0f0c` | Reject | Local continuation for third-party harnesses in `agent_sdk`; Agent SDK removed. |
| `46c484aae` | Reject | Tolerates unknown GSO formats in Drive sync; GraphQL/Drive sync removed. |
| `ef4b56219` | Adapted (already done) | Defer cmdlet loading in PowerShell bootstrap. The fork retains the POSIX DCS JSON bootstrap path; the PowerShell bootstrap asset remains only for retained remote SSH hosts. |
| `d2f0f88a6` | Reject | Preserves input after profile selection against old model/profile selector flow; old profile selector removed. |
| `584b5e453` | Reject | Google Antigravity CLI agent support; adds telemetry events, CLI-agent plugin-manager wiring, and specs. |
| `8d3baf91b` | Reject | Removes idle-on-failure in `agent_sdk/driver.rs`; Agent SDK removed. |
| `4595e6713` | Reject | Instruments 3p harness plugin install/updates in `agent_sdk`; Agent SDK and telemetry removed. |
| `178fe89bc` | Not applicable | Flips context-window circle to "context remaining" with a new SVG icon set; the fork has no `ConversationContext`/`ContextRemaining` icon surface or usage module. |
| `2e4da9325` | Not applicable | README typo for DockTilePlugin; the fork README is fork-specific. |
| `e5c2b86f9` | Reject | Adds `ai_access` field to onboarding telemetry; onboarding/telemetry removed. |
| `6f0cedb74` | Not applicable | Renames `WARP.md` to `AGENTS.md`; the fork already uses `AGENTS.md` with fork-specific content. |
| `cccaed63c` | Reject | Adds Ctrl/Cmd+Enter shortcut for code-review "Send to Agent" that routes to cloud Agent SDK; the fork's code review does not route to a cloud agent. |
| `d793e7f93` | Not applicable | Fixes doubled file separators in grep/file-glob headers on Windows; macOS-only host uses `/` separators. |
| `73ab280ee` | Not applicable | Escapes executable path in "dump debug info"; the fork has no `dump_debug_info` command palette action. |
| `278494bad` | Adapted | See Ported section above. |
| `b2ebe0d55` | Adapted | See Ported section above. |
| `2a69b055c` | Adapted | See Ported section above. |

## Verification

Commands to run after porting:

- `cargo fmt -- --check`
- `cargo check -p warp --all-targets --message-format short`
- `cargo check --workspace --all-targets --message-format short`
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`
- `git diff --check`
- Full deleted-surface scans from `AGENTS.md`.

Results:

- `cargo fmt -- --check` passed.
- `git diff --check` passed.
- `cargo check -p warpui_core --tests` passed.
- `cargo check -p warp_editor --lib` passed.
- `cargo nextest run -p warpui_core` ran 270 tests: 270 passed, 7 skipped. Includes the retained markdown link-click tests.
- `cargo nextest run -p warp_editor` ran 415 tests: 410 passed, 5 failed. All 5 failures (`test_inline_markdown_roundtrips`, `test_highlight_url_before_link`, `test_highlight_urls`, `test_highlight_urls_unicode`, `test_links_not_auto_highlighted`) are pre-existing on the fork baseline `5459995f6` and are unrelated to this merge.
- The two `expand_selection` punctuation-boundary tests ported with `831327c6e` were dropped because they assume upstream selection semantics that this fork's `formatted_text_element` does not match.
- `cargo check -p warp --lib` could not complete in this environment because the `aws-lc-sys` C dependency rebuild stalls; this is an environment/toolchain issue, not a code issue. The changed Rust crates compile independently.
- The cloud/auth/billing/telemetry scan found no restored surfaces in the diff.
- The MCP/skills scan found no restored surfaces in the diff.
- The local Linux/Windows/Web scan found no restored surfaces in the diff.
- No changed Rust file imports a removed module.
