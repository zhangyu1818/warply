# Upstream Master Audit 2026-07-11

Range under review: `05927696c..upstream/master` (98 commits)

Previous audited upstream tip: `05927696c Avoid panic in fallback shell when current uid has no passwd entry (#13367)`

Current upstream tip detected: `a01df387a Cache TUI transcript block heights to fix long-transcript scroll lag (#13592)`

Total upstream commits in this incremental range: 98

Status: triage complete. Retained terminal, shell, PTY, macOS-platform, workspace/tab, AgentView, editor, and launch-config fixes were ported or adapted manually. TUI-only changes, cloud-agent/billing/GraphQL/managed-secrets/MCP/skills/native-platform changes, and code paths absent from the fork were rejected or marked not applicable.

## Ported Or Adapted

- `b7430f40a9` + `453f4d30c` Ported both passwd-resolution commits as a unit. `get_pw_entry` was replaced with `resolve_current_user()`, which tries `getpwuid_r` via nix, then `getent passwd <uid>`, then `/etc/passwd`, returning `Option<CurrentUser>` (owned strings) instead of panicking. The dead `get_pw_shell()` helper was removed. `compute_fallback_shell()` in `shell.rs` now calls `resolve_current_user()` and logs an error when all three lookups fail. Adapted: the upstream `report_error!` call was replaced with `log::error!` (the fork does not have the `report_error!` macro). The `Passwd` struct and its `CStr`/`MaybeUninit`/`ptr`-based API were replaced by owned-string `CurrentUser` with new parse_passwd_line tests.
- `b038c0f3f` Ported the `DragTabsToWindows` feature-flag promotion from `DOGFOOD_FLAGS` to `RELEASE_FLAGS`, making cross-window tab dragging a stable feature. The fork uses `DragTabsToWindows` (not upstream's `CrossWindowTabDragging` label — that is just the feature's human description).
- `a612c9591` Ported the cross-window tab drag crash fix. `finalize_preview_as_new_window` now returns `DropResult::NoOp` when the preview workspace is already gone instead of tearing out a bystander tab. `handle_drop_result` guards `remove_tab_without_undo` with bounds checks. `dispatch_window_resized` defers the `window_resized` callback when the app RefCell is already borrowed, preventing panics during synchronous AppKit frame-size callbacks triggered by tab tear-off fullscreen transitions.
- `482f63ac7` Ported tab-level commands from launch config URIs. `TabTemplate` gained a `commands: Vec<CommandTemplate>` field and a `layout_with_tab_commands()` method. The workspace view now calls `layout_with_tab_commands()` instead of cloning the layout, so per-tab commands are injected into the startup pane.
- `df8ba45c0` Ported the "Copy current path" command palette action. Added `CopyCurrentPath` workspace action variant, `path_from_focused_pane()` method on `PaneGroup` (resolves the focused terminal session CWD or file pane path), and an integration test. Adapted: the fork's `FileNotebookView` uses `local_path()` instead of `path().display_path()`, and the integration test uses `warpui::integration` (not `warpui_core`).
- `37dc8830d` Ported the macOS CGFont glyph misrender fix. Replaced the last-writer-wins `fonts_by_name: DashMap<Arc<String>, FontId>` with a `cgfont_to_id: DashMap<CGFontKey, FontId>` keyed on CGFont identity (`CFEqual`/`CFHash`). Fixes rich-text glyph misrendering when multiple font instances share the same PostScript name but have different identities. Added regression tests in `fonts_tests.rs`.
- `328a3ae94` Ported the TaskStore IndexMap refactor. Replaced the dual `linearized_refs: Vec<ExchangeRef>` + `exchange_id_index: HashMap<...>` fields with a single `exchanges: IndexMap<AIAgentExchangeId, ExchangeRef>`, keeping O(1) lookup while eliminating the redundant rebuild. The `pane_group/mod.rs` drive-by hunk was skipped (fork does not have the `remote_tty` import split). Ported 5 applicable tests; skipped 4 `prune_unreachable_subtasks` tests (method absent in fork).
- `85e18dda6` Ported the LRC auto-resume fix after Ctrl-C takeover. Converted `UserTakeOverReason::Stop` from a bare unit variant to `Stop { should_auto_resume: bool }`. Added custom Deserialize impl for legacy `"Stop"` serialization. The Ctrl-C path sets `should_auto_resume: true`; user-initiated stop defaults to `false`. The resume-suppression check now uses `state.should_auto_resume()`. Renamed `set_user_control_with_stop_reason` to `set_user_control_for_teardown`. Added two serde round-trip tests. The `stop_local_agent_conversation` function and `send_telemetry_from_ctx` import (absent in fork) were skipped.
- `b193c5efd` Ported the vertical-tab group header fix. Added `should_show_tab_group_header(has_custom_title, is_being_renamed, visible_pane_count)` helper that returns true when a tab has >1 visible pane, so multi-pane vertical tabs always render their group header. Added 5 test cases.
- `a2a73b26f` Ported the `/repos` menu "No results" flash fix. Added a `GitSummaryCache` to `RepoMenuDataSource` and background loading via `ctx.spawn`, so the menu shows cached git summaries immediately while refreshing, instead of flashing "No results" during the initial load.
- `6b01a8e2f` Adapted the Markdown Viewer preference fix for file URLs. Added `prefer_markdown_viewer: bool` parameter to `classify_open_file_action`. In `open_file`, reads `EditorSettings.prefer_markdown_viewer` (gated on `local_fs`) and passes it through. Adapted: the fork's `classify_open_file_action` has no `Notebook` branch (downstream `resolve_file_target_to_open_in_warp` already enforces the preference), so the parameter is plumbed but currently unused for routing. Updated `uri_test.rs` calls to the new 2-arg signature.
- `a7f705c59` Adapted the pane-drag header width guard. Ported only the `app/src/pane_group/pane/view/mod.rs` fix: wrapped the drag-preview `ChildView::new(&self.header)` in a `ConstrainedBox` with `DRAG_PREVIEW_HEADER_MAX_WIDTH = 400.0` to prevent an infinite-width panic during pane drag. The orchestration-pill-bar test hunk was skipped (feature absent in fork).
- `5ff4f8900` Ported the forked-conversation working-directory fix. Added `startup_working_directory()` to `ConversationRestorationInNewPaneType` — for `Forked` returns `conversation.current_working_directory().or_else(|| conversation.initial_working_directory())`. Updated `target_dir` and both call sites (`pane_group/mod.rs`, `workspace/view.rs`). Adapted to the fork's 3-variant enum (no `HistoricalCLIAgent` variant).
- `7cf461f24` Adapted the tools-panel tab toggles in Appearance settings. Ported 3 of the 4 upstream toggles: `ToggleToolsPanelProjectExplorer` (`CodeSettings.show_project_explorer`), `ToggleToolsPanelConversationHistory` (`AISettings.show_conversation_history`), and `ToggleToolsPanelGlobalSearch` (`CodeSettings.show_global_search`). The `ToggleToolsPanelWarpDrive` toggle was skipped because `WarpDriveSettings` does not exist in the fork (cloud Warp Drive removed). Adapted: used `log_setting_result(...)` instead of upstream's `report_if_error!` macro; dropped `LocalOnlyIconState::Hidden` arg not present in fork's `render_body_item`.

## Rejected Or Not Applicable

| Commit | Decision | Reason |
| --- | --- | --- |
| `13e8b6114` | Reject | Jupyter `.ipynb` rendering; depends on prerequisite PR #12462 (`ipynb_parser` crate + `Buffer::from_ipynb`) which is also absent. Would be a large multi-commit feature port. |
| `62c2d17f8` | N/A | CI workflow guard for forks; `.github/workflows/` changes are fork-owned. |
| `cdc3113da` | Reject | Cloud-run conversation model attribution; depends on removed `crates/graphql/` and `server_api/ai.rs`. |
| `150b4c158` | Reject | GUI/TUI agnostic file execution; introduces `diff_storage.rs`/`diff_types.rs` for TUI file-edit rendering and touches removed `warp_files`/`local_code_editor_wasm` paths. The fork's ACP tool-call rendering is separate. |
| `43cf43c06` | N/A | TUI flex `CrossAxisAlignment`; `crates/warp_tui/` absent. |
| `ea644c707` | N/A | Default LLM id panic fix; `app/src/ai/llms.rs` is a 1-line stub in the fork, the `AvailableLLMs`/`default_llm_info()` code does not exist. |
| `44893c508` | N/A | TUI file-edit rendering; `crates/warp_tui/` absent. |
| `be5dfddca` | N/A | `CARGO_FULL_PROFILE` build fallback; the fork's `app/build.rs` is macOS-only and has no `get_build_profile_name()` function. |
| `ce602f6fc` | Reject | Composio MCP gallery icon; app-managed MCP rejected. |
| `593b03f5f` | Reject | NLD history match for agent prompt history; depends on removed multi-agent server API (`warp_multi_agent_api`), `ai_queries` cloud routing, orchestration pill bar, and orchestration event streamer. |
| `79f06eca6`, `32fb25327`, `fa9bef873` | Reject | `skills-lock.json` updates for bundled common skills; skills removed. |
| `56d2022b2` | Reject | Local→cloud handoff with invalid model_id; depends on removed `ambient_agent`, `auto_handoff`, and old model-id validation. |
| `cecb67838` | N/A | TUI softwrapped input expansion; `crates/warp_tui/` absent. |
| `ccb7711b5` | N/A | `AGENTS.md` local-server rule update; fork has its own `AGENTS.md`. |
| `fc7e15fa0` | N/A | TUI tool-call labels; `crates/warp_tui/` absent. |
| `1a67b384f` | N/A | Workflow restriction fixes; fork has its own CI workflows. |
| `dcf985db6` | Reject | AI query cloud routing + sandbox status; depends on removed server API, ambient agents, MCP skills data source, `remote_tty` terminal manager, and profile model selector. |
| `34c909f7d` | N/A | TUI shell command input mode; `crates/warp_tui/` absent. |
| `ed34ab5e5` | Reject | Resend/Sentry MCP gallery icons; app-managed MCP rejected. |
| `0ac6f5948` | Reject | TUI startup speedup; touches removed orchestration/persistence/agent paths and `crates/warp_tui/`. |
| `fa0d6fc85` | N/A | TUI warping indicator; `crates/warp_tui/` absent. |
| `4c4ab7506` | N/A | Remove `ActiveIndexedReposChanged` telemetry event; event already removed in fork. |
| `c88f6b4bc` | N/A | Remove free-tier limit-hit modal; billing already removed in fork. |
| `c6ff8a284` | N/A | Custom Routers settings crash; `custom_router_view.rs` removed in fork. |
| `6f286a557` | N/A | TUI skills/project-context discovery; `crates/warp_tui/` absent and skills rejected. |
| `1c8571500` | Reject | Team BYO policies in model UI; depends on removed GraphQL billing/workspace queries, `model_menu_items`, `profile_model_selector`, and old model-selector flows. |
| `6807ff6d8` | Reject | CLI run-cloud harness/auth-secret flags; depends on removed `agent_sdk` and CLI agent command surface. |
| `883f22b00` | Reject | `log::error!` → `report_error!` migration; depends on removed `warp_errors` crate, agent SDK, agent_management, and cloud-setup paths. |
| `cf3ad092f` | Reject | `warp_errors` direct import; depends on the removed `warp_errors` crate. |
| `1c376cb0f` | Reject | Grok OAuth token refresh; depends on removed `grok_subscription`, `server_api`, and cloud API-key paths. |
| `6e916958e` | N/A | TUI running-block rendering; `crates/warp_tui/` absent. |
| `cac1b5b0c` | N/A | Remove Agent Mode lightened background overlay; `agent_view_bg_fill`, `FeatureFlag::AgentView` fullscreen overlay, and the overlay code paths were already removed in the fork. |
| `34393bc2c` | N/A | Separate persistence scope for TUI/GUI; TUI absent. |
| `a6a7cbf23f` | N/A | Double-click hidden-section expand; fork's `hidden_section.rs` is the primitive no-label version (the labelled/hoverable bar layer was never ported). Also adds `specs/GH11622/**` (rejected). |
| `541e804e3` | Reject | VA computer-use video recording blocklist UI; depends on removed cloud-proto conversion, `agent_sdk`, and cloud artifact upload. |
| `244f33408` | N/A | macOS protoc install resilience; fork's `action.yml` already handles protoc directly. |
| `3e02f83c2` | N/A | Slack label rename; fork does not have Slack labels in app menus. |
| `404cca2fd` | Reject | Slash command mixer extraction; TUI-enablement refactor providing zero benefit without `warp_tui`. Introduces `core.rs`/`gui.rs`/`tui.rs`/`mixer.rs` split and `tui_export.rs` changes (file absent). |
| `cc99722e7` | Reject | Feature-intro popover; depends on removed `one_time_modal_model`, `feature_intro_modal`, custom model router, and `WarpDriveSettings`. |
| `d58b2fa8c` | Reject | Remove feature-flag gating from feature-intro popup; same removed dependencies as `cc99722e7`. |
| `a694b00e7` | Reject | Run-details Status chip opens run in Oz web; Oz/cloud-run removed. |
| `37df8314d` | Reject | VA recording card title from agent summary; depends on removed `agent_sdk` action conversion and cloud recording paths. |
| `dad38605a` | Reject | MCP Servers search field sizing; `app/src/settings_view/mcp_servers/` removed. |
| `4cf1a4fa8` | N/A | Terminal theming probe for TUI; `crates/warp_tui/` absent. |
| `97dfcdeeb` | N/A | macOS computer-use keycode cache fix; the retained `crates/computer_use/src/mac/` primitives exist, but the actor is never constructed from the ACP flow (`create_actor` calls in `request_computer_use.rs`/`use_computer.rs` are dead code serving the removed agent SDK). |
| `fa8d92157` | Reject | TUI footer credits/cost usage; depends on removed billing/credits and `crates/warp_tui/`. |
| `c47792ce1` | Reject | Subscribe button for out-of-credits AI error; billing removed. |
| `bef75a781` | N/A | TUI zero state; `crates/warp_tui/` absent. |
| `b667d96ca` | N/A | TUI editor element extraction; `crates/warp_tui/` absent. |
| `eedb5ac5d` | N/A | TUI inline diff rendering; `crates/warp_tui/` absent. |
| `5e9dc1c24` | Reject | Background computer use in X11; Linux X11/ffmpeg capture for cloud sessions, depends on removed cloud-proto conversion and Linux platform code. |
| `1eb369889` | N/A | Unmodified-lines hover scope; fork's `hidden_section.rs` is the primitive version without label/hover machinery. |
| `453f4d30c` | (see Ported above) | Merged with `b7430f40a9`. |
| `d45970299` | N/A | TUI diff test flake fix; `crates/warp_tui/` absent. |
| `2a682235e` | Reject | Billing & Usage dropdown highlight; billing removed. |
| `8f43af789` | Reject | PowerShell history `report_error!` migration; depends on removed `report_error!` macro and the fork has no PowerShell history read path. |
| `dc2df21d9` | N/A | Re-show queued prompts panel; `queued_prompts_panel.rs` removed in fork. Also touches `skills-lock.json`. |
| `f971dd32c` | N/A | TUI autoupdate status; `crates/warp_tui/` absent. |
| `a2c176009` | N/A | TUI slash command query state; `crates/warp_tui/` absent. |
| `40f59b517` | N/A | Remove selected telemetry events; telemetry already removed in fork. |
| `6250691e9` | Reject | Suppress passive suggestions for shared-session cloud-agent viewers; depends on removed shared-session/cloud-agent viewer. |
| `f69124c65` | N/A | Stop `app:reopen_closed_session` leaking into TUI; TUI absent, no TUI keymap context for the binding to leak into. |
| `a5c11c70b` | N/A | TUI response duration/credits indicator; `crates/warp_tui/` absent and billing removed. |
| `16837e6f7` | Reject | Integration-test-video skill scope; skills rejected. |
| `f6f87aec4` | Reject | Bundled skills display by id; skills rejected. |
| `86dfca99c` | N/A | TUI transcript selection; `crates/warp_tui/` absent. |
| `f98dbbc54` | Reject | Deserialize cloud environments without docker image; depends on removed `cloud_object_models`/`agent_sdk/environment.rs`. |
| `c242e1cae` | Reject | TUI non-interactive login via `--api-key`/`WARP_API_KEY`; TUI absent and this is the removed account-login path. |
| `c8dd4e1cd` | Reject | Skills distinguishing GUI vs TUI front-ends; skills rejected. |
| `78c9f5cf4` | Reject | verify-tui-change skill; skills rejected. |
| `3e3711ce9` | N/A | TUI agent shell command dedup; `crates/warp_tui/` absent. |
| `9e1055004` | Reject | Copy action for locked cloud-mode queued prompt; depends on removed cloud-mode queued prompt. |
| `a01df387a` | N/A | TUI transcript block-height cache; `crates/warp_tui/` absent. |
| `305acd1a7` | N/A | Pair dangling tool_use when forking mid-tool-call; fork's `fork_conversation_at_exchange` uses ACP transcript JSON, not the multi-agent server API the upstream fix targets. |
| `ea644c707` | (see above) | N/A |
| `87f0753cc` | N/A | Make BlocklistAiAinputModel surface agnostic; the refactor extracts GUI/TUI input-mode-policy split. The fork has no TUI and no `input_mode_policy.rs`/`GuiInputModePolicy` module the TUI split depends on. Also adds `specs/input-mode-policy/TECH.md` (rejected). |

## Verification

Commands run after porting:

- `cargo fmt -- --check` — passed.
- `cargo check --workspace --all-targets --message-format short` — passed (20.72s, no errors).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed, 2412 skipped.
- `cargo nextest run -p warp -E 'test(task_store) | test(parse_passwd) | test(resolve_current) | test(cli_controller) | test(launch_config) | test(vertical_tabs) | test(should_show_tab_group)'` — 113 passed, 2455 skipped.
- Deleted-surface scan of the full diff for `access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment` — no restored surfaces (only a fork note comment explaining why the Warp Drive toggle was omitted).
- Deleted-surface scan of the full diff for `mcp_server|mcpServers|bundled skills|channel-gated|ReadSkill|InvokeSkill` — no restored surfaces.
- Deleted-surface scan of the full diff for `target_os.*linux|target_os.*windows|cfg(windows)|WSL|MSYS2|ConPTY` — no restored surfaces.
- No new Cargo dependencies were added (`Cargo.toml`/`Cargo.lock` unchanged).
- No changed Rust file imports a removed module.
