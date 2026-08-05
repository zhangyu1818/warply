# Upstream Master Audit 2026-08-05

Range under review: `956ae6be4..upstream/master` (42 commits)

Previous audited upstream tip: `956ae6be4 Keep TUI zero state stable during shell startup (#14632)`

Current upstream tip detected: `c8a166b6c Only accent an onboarding credit pack when the credit option is chosen (#14712)`

Total upstream commits in this incremental range: 42

Status: triage complete. A focused set of retained terminal/keyboard/UI/macOS-shell fixes were ported or adapted. The bulk of the range is TUI-only (`crates/warp_tui/`, absent in this fork), computer-use recording (`crates/computer_use`, removed surface), cloud/orchestration/agent-SDK/billing/credits/onboarding, MCP app-config/skills, telemetry (N/A — infrastructure removed), voice input, Linux/Windows/WASM platform, and Windows installer CI.

## Ported Or Adapted

- `2f4094713` Ported the `TerminalModel::exit()` diagnostic log. Added `log::debug!("Terminal model exiting: reason={reason:?}");` immediately after the `handled_exit` early-return guard. `ExitReason` already derives `Debug` in the fork. The fork's `exit()` body is shorter than upstream's (no `plan_lifecycle_transition`), so the line was placed at the matching semantic position right after the guard.
- `5c15a0075` Ported the `UniformList` panic-to-log fix into `warpui_core`. `visible_items_tx.try_send()` failures in `UniformList::layout` now log at `error!` instead of `.expect()`. Adapted: fork uses edition 2021 (not 2024), so the let-chain `if let .. && let ..` was written as nested `if let` blocks. The file lives at `crates/warpui_core/src/elements/uniform_list.rs` (no `gui/` subdirectory in the fork).
- `f919f8935` Ported the SSH wrapper rc-sourcing fix into all three bootstrap scripts (`bash_body.sh`, `zsh_body.sh`, `fish.sh`). The bug was that `unset RCS; unset GLOBAL_RCS;` unsets shell *variables*, not the zsh *options*, so rc files were still sourced on warpified remote sessions. Folded both options into the existing `unsetopt`: `unsetopt ZLE RCS GLOBAL_RCS;`. The surrounding payload differs from upstream (fork generates `WARP_SESSION_ID` at runtime instead of using a precomputed `remote_session_id`), so the replacement was applied surgically to the literal token sequence in each file.
- `cb8fe83b3` Ported the ctrl-/ → US (`0x1f`) terminal encoding fix. Added `("/", C0::US)` to the `KEYSTROKE_TO_C0_CODE` table in `escape_sequences.rs`, and reserved `/` in `CONTROL_CHARACTER_KEY_REGEX`. Adapted the `bindings.rs` change: fork is macOS-only and has no `mac_only_keystroke` helper, so `ToggleKeybindingsPage` was changed to `parse_keystroke("cmd-/")` (the fork's equivalent of `mac_only_keystroke`). Added regression test `test_ctrl_slash_emits_us_control_code` and extended `test_keystroke_to_c0_control_code`.
- `af2f7c61d` Ported the Kitty keyboard protocol Cmd/Option editing-keys fix. The CSI-u gate in `kitty_keyboard_protocol.rs` now treats any `Cmd`-modified key as ambiguous and detects macOS Option IME composition from the OS-provided `chars` instead of blanket-excluding `alt`. Replaced the single-byte `ToModifierEscapeByte` trait (now removed) with a shared `modifier_param()` helper that encodes Super (value 8) and multi-digit combos. Added `delete_keystroke_to_escape_sequence()` for modified Delete (`CSI 3;<mods>~`). Rewrote `cursor_movement_keystroke_to_escape_sequence()` to use `modifier_param` for multi-digit modifiers. `keystroke_to_csi_u` now calls `super::modifier_param` instead of duplicating the logic. Ported 3 regression tests (Cmd/Option × Backspace/Delete/arrows, Option+Space composition, Cmd+function-key). The test file is `escape_sequences_test.rs` (singular) in the fork, not `escape_sequences_tests.rs`.
- `ad7723845` Adapted the tab-bar background fix. The fork's `render_tab_bar` lacked upstream's multi-team tint block, so the adaptation simply removed the `FeatureFlag::NewTabStyling`-gated `fg_overlay_1` background so the tab bar inherits the terminal background. `tab_bar_container` is no longer `mut`. The `NewTabStyling` flag remains in use in `tab.rs`.
- `963295b69` Adapted the synced-inputs indicator for the vertical tabs sidebar. `TAB_INDICATOR_SYNCED_COLOR` is now `pub(crate)` in `tab.rs`. Added `shows_synced_inputs_indicator`, `row_shows_synced_inputs_indicator`, `render_synced_inputs_indicator`, and a shared `render_row_title_line` helper to `vertical_tabs.rs`. Added `PaneProps::window_id()`. Adapted: the fork's `vertical_tabs.rs` predates upstream's unread-activity refactor (no `has_unread_activity`), so `render_row_title_line` takes the existing badge-indicator flag (`props.typed.badge(app).is_some()`) as its activity argument. Wired into `render_terminal_row_content` and `render_compact_pane_row` (the two active row renderers). The summary tab item was left unchanged because the fork's version has no existing indicator wrapping path. Synced-inputs is a retained local terminal feature (keystroke broadcast within a pane group), not cloud session sync.
- `04b488e55` Ported the open-file-from-agent-preview line-range fix. Changed `range_start: None` → `range_start: <line_col>` at all four workspace open-file entry points that have a `line_col` in scope (`pane_group::Event::OpenFileWithTarget`, `RightPanelEvent::OpenFileWithTarget`, `LaunchConfigModalEvent::OpenFileWithTarget`, `add_tab_for_code_file`). Added a `code_source: Option<CodeSource>` parameter to `open_file_notebook` and threaded it into `FilePane::new`, so the rendered-markdown → raw-code toggle keeps the requested line. Added `FileNotebookView::code_source()` getter. The fork does not use `LocalOrRemotePath` (plain `PathBuf`), so no type adaptation was needed.

## Rejected Or Not Applicable

The 42-commit range is dominated by removed-surface or TUI-only work. Key decisions:

### TUI-only (N/A — `crates/warp_tui/` absent)

`8b0e2e876`, `782b010ec`, `132db5c54`, `c7d534844`, `01662d634`, `fd8532fb1`, `02c042063`, `df464e50e`, `45db9e008`, `fe22b2ec5`.

### Computer-use recording (Reject — removed `crates/computer_use` overlay/recording + agent SDK)

`314fa20a0`, `fa5d4acf3`.

### Cloud / orchestration / agent-SDK / billing / credits / onboarding (Reject — removed)

`803f786f0` (internal cloud agent dev image), `6653023cc` (built-in Factory MCP server for Oz cloud/CLI agents), `3e8a98990` (agent-dev image non-root user), `0d6f77516` (Warp Agent CLI OAuth device auth), `8b3327f0e` (Billing & Usage V2 workspace-settings CTA), `58075b500` (duplicate teamless workspace plan badges), `d51d1e61c` (workspace admin panel on Billing & Usage V2), `c7e3c4a03` (onboarding ad-hoc AI credit pack), `c8a166b6c` (onboarding credit pack accent).

### MCP / skills (Reject — removed)

`c7ab9c028` (`WARP_SKILL_DIRS` env var for agent-driver headless skill loading).

### Auth / docs links (Reject — no-op in fork)

`a93a68cff` (in-app docs links `/agent-platform/` → `/agents/`/`/platform/`). The fork's `ai_page.rs` and `zero_state_block.rs` contain none of the targeted URL constants or widgets; the ACP-only fork already lacks the cloud-agent/MCP/Rules widgets.

### Linux / Windows / WSL platform (Reject — macOS-only)

`136f451dc` (WSL UNC host case-insensitive recognition — local-host WSL path detection, `parse_wsl_unc_path`/`WslUncPath` removed from fork's `path.rs`), `d46473504` (route Warp-internal git through `wsl.exe` for WSL UNC working directories — local-host Windows-only, `crates/warp_util/src/git.rs` absent), `a9cc833a8` (Windows autoupdate for TUI), `dc00f0086` (Intel Xe Graphics Mesa Vulkan adapter allowlist — Linux/Wayland), `01a0b91ee` (redundant warp-channel-config install step for Windows releases).

### CI / Docker infra (Reject — absent in fork)

`62a687bde` (`script/push-dev-image` chmod — file absent).

### AGENTS.md / docs (N/A — upstream contributor-process docs)

`9d2bab073`, `57d8ca949` (upstream `AGENTS.md` comment-noise and formatter guidelines — fork has its own `AGENTS.md`).

### Agent CLI welcome / copy (Reject — removed cloud-agent surfaces)

`8b0e2e876` (TUI welcome copy across first-run surfaces), `df464e50e` (agent CLI welcome zero-state copy).

## Verification

Commands run after porting:

- `cargo check -p warp --all-targets --message-format short` — passed.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo fmt -- --check` — passed.
- `cargo nextest run -p warp_terminal -E 'test(escape_sequences)'` — 22 passed (including 4 new tests: `test_ctrl_slash_emits_us_control_code`, `test_kitty_protocol_cmd_and_option_editing_keys`, `test_kitty_protocol_mac_option_space_composition_is_not_disambiguated`, `test_fn_keystroke_with_cmd_modifier`).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- `cargo nextest run -p warp -E 'test(vertical_tabs) | test(sync_inputs)'` — 70 passed.
- Deleted-surface scan for `access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment` — no restored surfaces (all hits are `Arc::upgrade()`/`WeakViewHandle::upgrade()` calls).
- Deleted-surface scan for `mcp.*capab|capab.*mcp|mcp_server|mcpServers|bundled skills|channel-gated-skills|ReadSkill|InvokeSkill` — no hits.
- Deleted-surface scan for `target_os = "linux"|target_os = "windows"|cfg(windows)|WSL|MSYS2|ConPTY|Wayland|X11|winreg|x11rb` — only retained SSH `ForwardX11=no` config strings and a `ConPTY` explanatory comment in `zsh_body.sh`.
