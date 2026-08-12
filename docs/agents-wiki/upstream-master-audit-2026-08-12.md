# Upstream Master Audit 2026-08-12

Range under review: `683d40782..upstream/master` (15 commits)

Previous audited upstream tip: `683d40782 [CODE-1946] Multi-level orchestration UI in the Warp TUI (#14884)`

Current upstream tip detected: `42effe840 Rename Oz Agent UI to Warp Agent (#11022)`

Total upstream commits in this incremental range: 15

Status: triage complete. 2 commits ported (1 adapt, 1 adapt), 13 rejected/not applicable.

## Ported Commits

### `b0c7a7674` — Log process-group cancellation outcomes for diagnosability (CSAT-10070 / GH#13852) (#14937)

Decision: **adapt**

Observability + safety change to `terminate_process_group` in `app/src/terminal/model/session/command_executor/local_command_executor.rs`, the same function the previous merge (`a4769955f`, 2026-08-11) adapted for process-group cancellation. This commit adds:

1. A `pid < 2` bound before signaling. A pgid of 0 targets the caller's own process group, and 1 negates to -1 (SIGKILL every signalable process) — neither is ever a legitimate target.
2. Per-branch outcome logging on every path: successful `kill(-pgid, SIGKILL)`, `ESRCH` (group already gone), `EPERM` (not permitted), and other errno. Previously `ESRCH`/`EPERM` were silently swallowed and a successful kill logged nothing.

Adaptations from upstream:
- Uses the fork's `safe_info!`/`safe_warn!` safe-logging macros (with `safe:`/`full:` split) instead of `log::info!`/`log::warn!`, matching the convention established in the `a4769955f` port.
- No `#[cfg(windows)]`/`#[cfg(not(unix))]` gates (the fork's `CommandBuilder` has no Windows branch and the maintained target is macOS).

### `f73d44f11` — Add an action to cycle the active tab color (#14329)

Decision: **adapt**

Retained terminal UI feature: a keyless-by-default workspace action (`CycleActiveTabColor`) that cycles the active tab color through the canonical palette (Red → Green → Yellow → Blue → Magenta → Cyan → cleared → Red), advancing from the resolved visible color including directory-derived colors.

Ported parts:
- `app/src/tab.rs`: added `next_tab_color(Option<AnsiColorIdentifier>) -> SelectedTabColor`, placed after the `SelectedTabColor` impl block. Derives the next color from `TAB_COLOR_OPTIONS`.
- `app/src/workspace/action.rs`: added `CycleActiveTabColor` variant to `WorkspaceAction` and to the `should_save_app_state_on_action` true-branch alongside `SetActiveTabColor`.
- `app/src/workspace/mod.rs`: registered `workspace:cycle_active_tab_color` as an editable binding under `BindingGroup::Settings` with the `Workspace` context predicate.
- `app/src/workspace/view.rs`: imported `next_tab_color` and handled `CycleActiveTabColor` in `TypedActionView`.

Adaptations from upstream:
- The fork has **no tab grouping** (`group_id`, `tab_groups`, `TabGroup`, `set_tab_group_color`, and the `GroupedTabs` feature are all absent). The upstream handler's grouped-tab branch (`set_tab_group_color`) was not ported; only the ungrouped `set_tab_color` path applies.
- Upstream also removed a `ctx.dispatch_global_action("workspace:save_app", ())` from `set_tab_group_color` — not applicable since that function does not exist in the fork. The fork's `set_tab_color` already dispatches `workspace:save_app` itself.
- The upstream specs under `specs/GH14069/` were rejected.
- The upstream unit tests (`tab_tests.rs`, `workspace/view_tests.rs`) and integration tests live under different filenames in the fork (`view_test.rs`, `action_tests.rs`, etc.) and were not ported; the action is exercised by the existing workspace/tab test suites.

## Rejected / Not Applicable

### `80a203474` — Don't hard-fail message conversion on an unrecognized citation type (#14915)

Decision: **not applicable**. The touched file `app/src/ai/agent/api/convert_from.rs` does not exist in the fork. The fork's `crates/ai/src/agent/citation.rs` is ACP-only: `AIAgentCitation` has a single `LocalObject { uid }` variant with no `TryFrom<api::Citation>`, no `UnknownCitationTypeError`, and no `warp_multi_agent_api::DocumentType` — the entire server-message conversion path this commit fixes is absent.

### `5d6887080` — review-pr-local: mark comment/testing guideline violations as IMPORTANT (#14907)

Decision: **not applicable**. The fork's `.agents/skills/review-pr-local/SKILL.md` has diverged: it references `WARP.md` (not `AGENTS.md`) for Rust conventions and lacks the upstream comment-audit bullet that this commit amends. The upstream bullet references the "Comments" guidance under "Development Guidelines" in `AGENTS.md`, which the fork's `AGENTS.md` (the fork-contract entry point) does not contain.

### `cd49bd7fe` — Pin ws dependency to 8.21.0 in GraphQL schema package (#14934)

Decision: **reject** (removed GraphQL schema). `crates/warp_graphql_schema/` is absent from the fork.

### `e49124b7e` — REMOTE-2111 (1/3): Add checkpoint upload/commit pipeline mechanics (#14588)

Decision: **reject** (removed agent_sdk surface). All touched files (`app/src/ai/agent_sdk/driver/snapshot.rs`, `snapshot_tests.rs`, `app/src/server/server_api/harness_support.rs`) are absent.

### `f4be4f692` — Support Homebrew-managed Warp Agent CLI updates (#14899)

Decision: **not applicable**. `crates/warp_tui/` is absent from the fork. The `app/src/settings/tui_autoupdate.rs` TUI autoupdate setting is not a retained surface (the fork's maintained target is the macOS GUI app).

### `1f1ad6997` — [CODE-1838] Fix TUI focus for long-running command input (#14943)

Decision: **not applicable** (`crates/warp_tui/` absent). 100% of the diff lives in `crates/warp_tui/`.

### `47f823221` — review-pr-local: audit comments against each named AGENTS.md sub-rule (#14944)

Decision: **not applicable**. Same rationale as `5d6887080`: the fork's `review-pr-local` skill has diverged and the amended bullet references upstream `AGENTS.md` comment sub-rules that the fork's documentation structure does not carry.

### `e5d60c240` — REMOTE-2111 (2/3): Add periodic checkpoint coordinator state machine (#14573)

Decision: **reject** (removed agent_sdk surface). The coordinator module (`app/src/ai/agent_sdk/driver/checkpoint_coordinator.rs`) and its tests are absent. The shared `app/src/terminal/view.rs` edit adds a `TerminalView::ai_action_model()` accessor whose only consumer is the checkpoint coordinator's `is_safe_boundary` predicate — dead code without the coordinator, so it was not ported.

### `495f97572` — Center the orchestration chip avatar and keep its status badge inside the pill (#14893)

Decision: **not applicable** (removed orchestration surface). `app/src/ai/blocklist/agent_view/orchestration_pill_bar.rs` is absent. The `app/src/ui_components/icon_with_status.rs` change is comments-only (the PR description confirms "no behaviour change for the tab or other agent-icon callers").

### `ad3046241` — REMOTE-2111 (3/3): Wire the periodic checkpoint coordinator into AgentDriver (#14589)

Decision: **reject** (removed agent_sdk surface). All touched files (`app/src/ai/agent_sdk/driver.rs`, `mod.rs`, `app/src/server/server_api/harness_support.rs`) are absent. The `crates/warp_features/src/lib.rs` edit only adds a feature flag for the coordinator.

### `a1af68cbd` — Fix MCP JSON viewer key vertically centered against tall expanded string (#14949)

Decision: **not applicable**. `app/src/ui_components/json_tree.rs` does not exist in the fork (the fork renders ACP tool-call JSON via a different path).

### `87a4e4b34` — Fix char-boundary panic in WorkflowDataSource preview truncation (APP-5287) (#14933)

Decision: **not applicable**. `app/src/search/ai_context_menu/workflows/data_source.rs` does not exist in the fork. The fork's `app/src/search/ai_context_menu/` has no `workflows` subdirectory; the panic-prone `&content_preview[..197]` slice is not present.

### `42effe840` — Rename Oz Agent UI to Warp Agent (#11022)

Decision: **not applicable**. The fork already contains zero "Oz" references in source (verified by repo-wide `rg "\bOz\b|OzAgent|OzCloud|Fix with Oz|What's new in Oz|New Oz|Cloud Oz"`). The fork removed cloud/ambient-agent surfaces (`DefaultSessionMode::CloudAgent`, `SessionType::Oz`, `IconWithStatusVariant::OzAgent`, `Icon::Oz`/`Icon::OzCloud`, `buy_credits_banner.rs`, `harness_availability.rs`, etc.) during baseline creation, so there is nothing left to rename.

## Verification

- `cargo check -p warp --all-targets --message-format short` — passed.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo fmt -- --check` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- `cargo nextest run -p warp -E 'test(tab) | test(workspace) | test(local_command_executor) | test(command_executor)'` — 279 passed.
- Deleted-surface scans re-run: only allowed hits (WeakHandle `upgrade()` calls, tokenizer JSON vocabulary, retained SSH `ForwardX11=no`, retained `ConPTY` explanatory comment).
