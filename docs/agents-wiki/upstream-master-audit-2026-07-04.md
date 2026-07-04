# Upstream Master Audit 2026-07-04

Range under review: `c0902a246..05927696c`

Previous audited upstream tip: `c0902a246 QUALITY-919: register orchestrators for child events on wait_for_events (#13208)`

Current upstream tip detected: `05927696c Avoid panic in fallback shell when current uid has no passwd entry (#13367)`

Total upstream commits in this incremental range: 79

Status: triage complete. Retained terminal, shell, editor, AgentView, ACP-rendering, macOS-integration, and local-persistence fixes were ported or adapted manually. Cloud agent, custom model router, BYOK/billing, tab-grouping/pinning, TUI app, onboarding/telemetry, MCP/skills, managed secrets, native Linux/Windows packaging, and the terminal lifecycle stack were rejected or marked not applicable.

## Ported Or Adapted

- `05927696c` Ported the fallback-shell panic guard. `compute_fallback_shell()` maps a missing passwd entry (`Ok(None)`) or lookup error to `None` and falls through to the well-known shell chain instead of panicking.
- `f18169282` Ported the wide-character spacer crash fix. `GridStorage::shrink_cols` now resets a retained trailing `WIDE_CHAR` cell when its `WIDE_CHAR_SPACER` was discarded during a no-reflow clear resize. The `specs/GH12243/**` documents were rejected. The `grid_handler_tests.rs` regression was skipped (file absent in fork); the `grid/tests.rs` reflow regression was retained.
- `e082f0b9f` Ported the SSH `RemoteCommand` fallback. The bash/zsh/fish bootstrap scripts now probe `ssh -G` for a `remotecommand` setting and fall back to plain `command ssh` when the user's ssh_config sets `RemoteCommand`, so Warpify does not break those sessions.
- `6c85d81de` Adapted the tab-divider contrast fix. Only the `TabComponent` border color change in `tab.rs` was ported (active `fg_overlay_2 -> fg_overlay_4`, inactive `fg_overlay_1 -> fg_overlay_3`). The `workspace/view.rs` tab-group header divider was skipped (fork has no tab groups).
- `0d2e85281` Adapted the internal doc-link fix. Only `INTEGRATION_TESTING.md` was corrected (pointed to `warpui_core/src/integration/step.rs`). The figma `wwds` files were skipped (absent in fork).
- `dfabfa5bb` Ported Markdown syntax highlighting. Registered the markdown grammar (`.md`/`.markdown`, `md` alias) via the `arborium` `lang-markdown` feature, extended `convert_capture_name_to_color` for `text.*`/`punctuation.*` captures, and added `markdown`/`md` to `ProgrammingLanguage::to_extension`.
- `29fb9e466` Ported the header toolbar chip label contrast fix for light themes (`readable_chip_label_color` WCAG AA adjustment) plus regression tests.
- `814753f3c` Ported visual-line navigation. `cmd+←/→` now jumps to the start/end of the soft-wrapped visual line, and macOS `Home/End` performs document navigation. Adapted: the `with_linux_or_windows_key_binding` predicate was dropped (macOS-only host).
- `fa8318157` Ported the multiline command-block duplicated-prefix fix. `header_grid.rs` now reconciles the command grid against the preexec canonical command to remove redraw artifacts. Two regression tests were ported; a third was skipped (depends on a `PromptMarker` API obfuscated in the fork).
- `2e5184485` Ported the standalone macOS CLI `bundled_resources_dir` fix. Added a `standalone` Cargo feature and a `cfg!(feature = "standalone")` branch that resolves resources to `<exe_dir>/resources` for loose binaries.
- `922ba2584` Ported the imported-comments perf guard. `BlocklistAIHistoryModel` gained a `conversations_with_imported_comments` registry so `update_imported_comments_disabled_state` short-circuits without recomputing per block.
- `09d931139` Ported the `TaskStore` O(1) `exchange_with_id` lookup via an `exchange_id_index: HashMap` maintained alongside `rebuild_linearized_refs_index`. The fork-deleted `conversation_tests.rs` was skipped.
- `6be84ac0b` Ported the conversation-rewind correctness fix. `truncate_from_exchange` now removes rewound-away non-root-task messages and prunes unreachable subtasks, and `persistence/agent.rs` handles the replace/delete-missing cases.
- `2f3b0c009` Ported the editable "New File" keybinding, switching `CustomAction::NewFile` from a `FixedBinding` to an `EditableBinding`. The ambient-agent tab binding constant was not introduced.
- `f1701be39` Ported the vertical-tab close behavior: closing the active vertical tab now activates the tab below (clamped) instead of the previous tab.
- `ddafc51ab` Ported the heap-profile command-palette action. `dump_heap_profile_to_disk` writes a local dhat/jemalloc profile and reveals it in Finder. Sentry upload paths remain absent.
- `0e7478737` Ported the permanently-dismissible ControlMaster/completions banner, reusing the fork's `private_user_preferences` dismissal pattern (the upstream OSC-52 banner surface was already removed in the fork).
- `a25e40b10` Ported PS1 inclusion when copying blocks (`command_and_output_to_string`) plus integration tests.
- `dbd412209` Adapted the `format_elapsed_seconds` extraction. Only the app-side move from `blocklist/block/view_impl/common.rs` to `util/time_format.rs` was ported; the TUI thinking-block body was rejected.
- `676c882b7` Adapted the `warp://settings` deeplink entrypoints (open/search/scroll-to-widget). All MCP settings, Teams, billing, cloud-environment, custom-router, and platform-API-key slug branches were stripped; only fork-existing widgets (`global_hotkey`, `appearance`) are mapped.

## Rejected Or Not Applicable

| Commit | Decision | Reason |
| --- | --- | --- |
| `ac0f658a9` | Reject | Cloud agent follow-ups / session-sharing-server; `terminal_manager.rs` removed in fork. |
| `48b42d122` | Reject | Billing & Usage legend; billing removed. |
| `c59a0f37b` | Reject | Oz/ambient-agent shell-exit failure reporting; deep coupling to removed ambient/Oz sync. |
| `b545c81aa` | Reject | Server-fetched memory citations; depends on removed `warp_multi_agent_api`/`FetchedMemory`. |
| `fd3ebd66d` | Reject | `specs/APP-2527/**`; specs rejected. |
| `6bec2fc6e` | Reject | `skills-lock.json` for bundled common skills; skills removed. |
| `5abd4233b` | Reject | Orchestrator auto-handoff-on-sleep; orchestration/cloud handoff removed. |
| `4c80f6319` | N/A | APP-2527 Phase 2 MCP pipeline; data source is `CallMCPToolResult` (app-managed MCP), replaced by ACP `ToolCallUpdate` in fork. |
| `1a9af7a08` | N/A | APP-2527 Phase 3 MCP JSON tree rendering; depends on Phase 2 `CallMCPToolResult`. |
| `f0afcc12e` | Reject | APP-2527 Phase 1 generic `JsonTreeView` component is clean, but has no consumer in the fork. Introducing it would add dead code; revisit only when wiring ACP `ToolCallUpdate` results to the component as a fork-internal change. |
| `965dd083d` | Reject | `/pr-comments` bundled skill; bundled skills removed. |
| `902b6bcef` | Reject | Windows secure storage; macOS-only host. |
| `ab085501f` | Reject | BYO_ENDPOINT billing policy / workspace GQL; billing and cloud GraphQL removed. |
| `974bd7632` | Reject | `CliAgentUserQuery`/LRC snapshot race; CLI/cloud agent and `server/telemetry` removed. |
| `4ececc2d1` | Reject | Linux AppImage / linuxdeploy; Linux packaging removed. |
| `b69c43b29` | Reject | Managed-secrets BYO first-party + WASM seal exports; managed secrets and WASM target removed. |
| `4c5ca9395` | Reject | TUI settings refactor; touches removed cloud_preferences/cloud_agent/drive and TUI settings. |
| `b2980b099` | Reject | Daily dev macOS bundling for the TUI; TUI target absent. |
| `0ccc55e57` | Reject | VA computer-use recording substrate (Linux X11/ffmpeg capture for cloud sessions). |
| `7ae929bcd` | Reject | VA computer-use video recording; depends on removed `agent_sdk` and cloud artifact upload. |
| `a5cc9008d` | Reject | Connect settings with TUI app; TUI target and removed `server_api` paths. |
| `d33823c82` | Reject | Client-side fallback for disabled models; old `/model`/`/profile` selector removed. |
| `15f1053b5` | Reject | TUI autoupdate; TUI target and upstream autoupdate removed. |
| `fbedf90a2` | Reject | Double ctrl-c to exit TUI; TUI framework + specs. |
| `305fc3a3d` | Reject | Background per-window computer use on macOS. The `crates/computer_use/src/mac/*` host primitives are retained-area, but the change is inseparable from the removed cloud-proto conversion (`agent/api/convert_conversation.rs`) and the Linux/Windows/noop branches; the ACP-only backend does not drive computer use, so the mac primitives cannot land without restoring the removed execution surface. |
| `47873fe18` | Reject | Agentic LLM-as-judge eval infrastructure; the integration-testing hooks depend on the removed warp-server companion linkage and cost/token analytics, and this fork does not run agent-mode evals. |
| `098c307c7` | N/A | Circular view-update crash on queued `/compact-and`; the fork's `input.rs` no longer carries the `compact_and`/`is_ready_for_cloud_followup_prompt` paths the fix targets. |
| `fa9d91a8d` | N/A | Redundant Metal CI install; fork action.yml already lacks the step. |
| `358ba8e5d` | N/A | Generalized virtualized viewport for TUI; `elements/tui/` and `warp_tui` absent. |
| `02169225e` | N/A | Slider switch in orchestration config block; file removed. |
| `9c59c69df`, `21e28ccce`, `1a5d135cd`, `9dcb9b890`, `3015d875b`, `8089a74d3` | N/A | Tab-group/pinning feature; no `TabGroups`/`Pinning` flag or grouping module in fork. |
| `b26cd0a48` | N/A | TUI mouse input; TUI input stack absent. |
| `887c4582b` | N/A | "Add Router" button spacing; `CustomModelRoutersWidget` removed. |
| `20430b8a2` | N/A | Scrollable model section in custom inference modal; modal removed. |
| `d866db41e` | N/A | Intel UHD 770 Windows adapter comment; fork uses Metal, no `wgpu` renderer. |
| `ffc0da0ba` | N/A | Compilation fix for `custom_inference_modal_tests.rs`; file removed. |
| `6cebc7a5a` | N/A | TUI input/transcript wiring; TUI app target absent. |
| `6ba811c9e` | N/A | Basic tool calling for the TUI; TUI target absent. |
| `94804667d`, `b9cbc2d28` | N/A | Tab pinning UI/flag; pinning flag absent. |
| `bc8c043a4` | N/A | Codesigning for TUI; `script/run-tui` absent. |
| `5e8f704a1` | N/A | Windows PTY batch-flush test in `shared_session`; both removed. |
| `0125df664` | N/A | `.github/STAKEHOLDERS` cleanup; file absent. |
| `7ebcd82d7` | N/A | Stop cloud-syncing deprecated SSH-wrapper migration triggers; the bug requires `SyncToCloud`/`EnableSshWrapper`, both removed in the fork. |
| `2f4d1d246` | N/A | `is_any_ai_block_focused` O(1) optimization; the method does not exist in the fork. |
| `2aa06b134` | N/A | Watcher gitignore rebuild storm; fork's `repo_metadata` predates the `build_tree_with_standing_queries` regression and already uses `check_ancestors=true`. |
| `45a062d04` | N/A | PowerShell linter cleanup; fork's `pwsh.ps1` is already clean (no trailing comments to fix). |

| `077749693` | Reject | Distinguish Precmd hooks with completion metadata (#12854). Introduces `HookSessionId`, but the fork only has the older `16c3273501` form of the `#12853` baseline (`PrecmdValue.session_id` is still `Option<u64>`, `CommandFinishedValue` has no `session_id`); upstream rewrote `#12853` as `14b0489fad`, so this commit cannot land without first re-porting that baseline. |
| `171f681a6` | Reject | Centralize terminal lifecycle mutations (#12855). Part of the lifecycle stack; routes ordered events through the removed `terminal/shared_session/viewer/event_loop.rs`. |
| `cb40a808e` | Reject | Gate terminal lifecycle transitions with a coordinator (#12856). Depends on the removed shared-session event loop and a `lifecycle/telemetry.rs` path that crosses the removed telemetry/crash-reporting boundary. |
| `43c21508f` | Reject | Recover missing CommandFinished hooks from Precmd metadata (#12858). Depends on the `#12854`/`#12855` coordinator and shared-session plumbing. |
| `d81d7b067` | Reject | Enable terminal lifecycle recovery in dogfood (#12859). Enables the `TerminalLifecycleRecovery` flag for the rejected lifecycle stack. |

The terminal-lifecycle hardening stack (`#12854`-`#12859`) was reviewed as a unit and rejected as a group. It is a 7-PR series rooted in `#12852` (a spec document, rejected) and `#12853` (Precmd completion metadata). The fork already ported the older `16c3273501` form of `#12853`, but upstream rewrote it as `14b0489fad` and introduced `HookSessionId` inside `#12854`. The coordinator/transition work routes ordered events through the removed `terminal/shared_session/viewer/event_loop.rs` and emits diagnostics via `lifecycle/telemetry.rs`. None of these commits can land without either restoring removed shared-session/telemetry surfaces or first re-porting the full `#12853`/`#12854` baseline as a fork-internal refactor — which is out of scope for an upstream port.

## Verification

Commands run after porting:

- `cargo fmt -- --check` — passed.
- `cargo check --workspace --all-targets --message-format short` — passed (30.31s, no errors).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed, 2395 skipped.
- Deleted-surface scan of the full diff for `access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment` — no restored surfaces.
- Deleted-surface scan of the full diff for `mcp_server|mcpServers|bundled skills|channel-gated|ReadSkill|InvokeSkill|target_os.*linux|target_os.*windows|cfg(windows)|WSL|MSYS2|ConPTY` — no restored surfaces.
- No changed Rust file imports a removed module.
