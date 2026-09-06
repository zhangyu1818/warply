# Upstream Master Audit 2026-09-06

## Scope

- Current fork before this audit: `b4fd89ea6` (`main`, post-`v2026.09.04` addendum).
- Upstream source reviewed: `a7326f8fe..upstream/master` (10 commits, tip `c388229de`). `a7326f8fe` itself was already ported as the post-`v2026.09.04` addendum and is not re-ported.
- Result: three ports (command-signatures bump, Create New Window editable keybinding, Bash PS1 double-expansion fix) and seven rejected/not-applicable commits, all touching fork-absent removed surfaces.

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `2a58ea28a` | Bump command-signatures for Rust Fig generators (APP-5776) | **Accept (ported)** | Exact `6a39d620` → `d69b340e` pin of the retained `warp-command-signatures` dependency (command-signatures#391/#392, Rust Fig generators and template filters); lockfile `warp-completion-metadata` source moved to the same rev. Fork was on the exact upstream baseline rev. |
| `ec6085e01` | Separate workspace root and harness working directories | **N/A** | Entire change lives in `app/src/ai/agent_sdk/driver/**` plus `app/src/pane_group/pane/local_harness_launch.rs`, all removed with the agent SDK; neither path exists in this fork. |
| `e5db969ae` | Factor team selection from object scope | **N/A** | Touches `app/src/ai/agent_sdk/{ambient,common,schedule}.rs`, `app/src/workspaces/user_workspaces/team_workspace_settings.rs`, `crates/warp_cli/src/scope.rs`, and `crates/warp_server_client/**` — all fork-absent (agent SDK, Teams workspace settings, server client). |
| `53b502c8e` | Make Create New Window an editable keybinding | **Accept (adapted)** | See port record below. |
| `17f432027` | Fix double expansion of Bash PS1 in "honor PS1" mode | **Accept (adapted)** | See port record below. |
| `d58f555a3` | fix(mcp): await global file servers before first turn | **N/A** | Restructures `app/src/ai/agent_sdk/driver.rs` MCP startup into `mcp_startup.rs` and rewrites `app/src/ai/mcp/file_based_manager*`/`file_mcp_watcher*`. Both `agent_sdk` and app-managed `ai/mcp` are removed surfaces in this fork. |
| `5a6ded1e8` | Cloud agents need a team | **N/A** | Cloud Mode team-required blocker: `ambient_agent/team_required.rs`, `harness_availability.rs` `CloudAgentStartBlocker`, `UserWorkspaces::cloud_agents_require_team`, `UserWorkspacesEvent::TeamsChanged`, `SettingsSection::Teams` — ambient/cloud-agent/Teams surfaces are absent. The `terminal/input.rs`/`view.rs` hunks only wire those absent symbols (no anchor: no `is_configuring_ambient_agent`, no `HarnessAvailabilityModel`). |
| `b2bcc408d` | Support azure devops forges | **N/A** | Azure DevOps forge support lives in `crates/cloud_object_models/src/cloud_environment.rs` (crate absent), `agent_sdk/driver/environment*`, shared-session sharer network tests, `drive/export_tests.rs`, and `crates/warp_cli/src/agent.rs` `RepositoryForge::AzureDevOps` — all fork-absent. |
| `a48ff8014` | Auto-attach Factory MCP to third-party harness runs | **Reject** | Agent-SDK harness auto-attach of the bundled Factory MCP server (`mcp_startup`, `skill_dirs_publish`), `resources/bundled/skills/factory-mcp/` (bundled skills stay out), and `specs/REMOTE-3140/TECH.md` (upstream specs rejected by default). |
| `c388229de` | Hide native-workspace teamless CTAs in Warp Drive (REV-2380) | **N/A** | Only touches upstream `app/src/drive/index.rs` with `join_team_header_text`/`should_show_join_separator`/`team_cta_sections`; the fork's `app/src/drive/` has no `index.rs` and no team CTA code (Teams CTAs never existed in the fork's local Drive). |

## Port record: `53b502c8e` Create New Window editable keybinding

### Runtime-ownership review

Purely local windowing/keybinding change: the nameless fixed `CustomAction::AddWindow`
shortcut becomes the named editable binding `workspace:new_window` (default Cmd-N,
`Workspace` context, gated on `ContextFlag::CreateNewSession`), the File-menu
"New Window" item derives its displayed keystroke from the live binding, and the
resource-center stub `CommandBinding` claiming the same name is removed. No Warp
service dependency.

### Applied from the exact upstream source

- `app_menus.rs`: the "New Window" menu item's property closure resolving the
  binding by `Trigger::Custom(CustomAction::AddWindow.into())` and setting
  `changes.keystroke` — verbatim.
- `resource_center/utils.rs`: removal of the `workspace:new_window` stub from
  `get_additional_keybindings` — verbatim (`FUNDAMENTALS_KEYBINDINGS` keeps the
  name, as upstream).
- `util/bindings.rs`: removal of the "hardcoded keybindings" comment above the
  `AddWindow` fallback — the fallback itself stays (upstream keeps it too).
- `workspace/mod.rs`: removal of the `FixedBinding::custom(AddWindow, …)` and the
  new `EditableBinding::new(NEW_WINDOW_BINDING_NAME, …).with_custom_action
  (CustomAction::AddWindow).with_context_predicate(id!("Workspace"))
  .with_enabled(CreateNewSession)` — verbatim.
- `workspace/view.rs`: `NEW_WINDOW_BINDING_NAME` const — verbatim.

### Fork adaptations

- Import/const lists: the fork's `workspace::view` import block and binding-name
  const list have no `LEFT_PANEL_WARP_DRIVE`, `NEW_AGENT_TAB`,
  `NEW_AMBIENT_AGENT_TAB`, `TOGGLE_NOTIFICATION_MAILBOX`, or `TOGGLE_WARP_DRIVE`
  entries; only `NEW_WINDOW_BINDING_NAME` was added in alphabetical position.
- `util/bindings.rs` conflict: kept the fork's macOS-only single-line
  `ReopenClosedSession => Keystroke::parse("cmd-shift-T")` (upstream's hunk
  context included its Linux/Windows branch, absent here); only the comment-line
  removal from the upstream patch was applied.

## Port record: `17f432027` Bash PS1 double-expansion fix

### Runtime-ownership review

Purely local shell-integration fix in `warp_precmd` (bash bootstrap): decide
`honor_ps1` before expanding/escaping `WARP_PS1`. In Shell (PS1) mode the
dynamic prompt is left to Bash (previously `warp_precmd` expanded `WARP_PS1`
once for the JSON payload and Bash expanded `PS1` again, double-incrementing
counters); in Warp prompt mode the prompt-preview metadata is unchanged.
Includes the upstream GUI integration test
`test_bash_honor_ps1_expands_dynamic_prompt_once`.

### Applied from the exact upstream source

- `app/assets/bundled/bootstrap/bash_body.sh`: both upstream hunks — the early
  `local honor_ps1 / deref_ps1="" / escaped_ps1=""` decision wrapping the
  `WARP_PS1_EXPANSION_SUPPORTED` expansion inside the non-honor branch, and the
  removal of the later `honor_ps1` recomputation (with its
  `escaped_ps1=""/deref_ps1=""` clearing) before the Precmd JSON payload.
- `crates/integration/src/bin/integration.rs`,
  `crates/integration/src/test/bootstrapping.rs` (`assert_active_prompt_text`,
  the new test using `HonorPS1::storage_key` user defaults, counter PS1, and
  prompt assertions `[1]`→`[3]`),
  `crates/integration/tests/integration/shell_integration_tests.rs` — verbatim.

### Fork adaptations

- The fork's bootstrap has no `WARP_IN_MSYS2` gate, so the
  `escaped_ps1=$(warp_escape_ps1 …)` call stays unconditional at the end of the
  non-honor branch (the fork's pre-existing de-MSYS2 form) instead of upstream's
  `if [ "$WARP_IN_MSYS2" = false ]` wrapper.
- `bootstrapping.rs` imports: the fork groups `warp::…`/`warpui::…` imports
  (no `warpui_core` crate); `AssertionCallback` was added to the fork's
  `warpui::{… integration::{AssertionCallback, TestStep}}` import instead of
  upstream's `warpui_core::integration` line.

## Verification

- `cargo check -p warp_completer` after the signatures pin: pass.
- `cargo check -p warp --all-targets --message-format short`: pass (8
  pre-existing lib warnings: warpui mac unsafe-block set, same as `main`).
- `cargo check -p integration --all-targets`: pass (new test compiles; GUI
  integration tests remain compile-verify only in headless sessions).
- `cargo check --workspace --all-targets --message-format short`: pass.
- `cargo fmt -- --check`: clean.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: 156 passed.
- `cargo nextest run -p warp -E 'test(util_bindings) + test(bindings)'`: 3 passed.
- `bash -n app/assets/bundled/bootstrap/bash_body.sh`: clean.
- `cargo build -p warp --all-targets --message-format short`: pass (exit 0;
  remaining diagnostics are the pre-existing warpui `unsafe_op_in_unsafe_fn`
  warnings identical to `main`); `cargo clean` after the release push.
- Deletion-surface scans: only pre-existing allowed hits (`WeakHandle::upgrade`
  in `workspace/view.rs`, etc.); no changed file introduced a removed-surface
  reference.
- `CARGO_PROFILE_DEV_DEBUG=0` used for check/test/build as in prior audits.
