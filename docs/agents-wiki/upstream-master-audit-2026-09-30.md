# Upstream Master Audit 2026-09-30

Range under review: `7aa4de12b..upstream/master` (4 commits)

Previous audited upstream tip: `7aa4de12b Scope Bedrock token requests to the run team (#16133)`

Current upstream tip: `648d03b51 Adding the option to switch teams on the client settings > teams page (#16120)`

Total upstream commits in this incremental range: 4

Status: triage complete. Zero ports — all four commits are rejected or not applicable against fork-absent anchors. The released tree is unchanged apart from this audit record.

## Per-Commit Triage

### `167f2003c` — Defer configured MCP startup until after environment setup (#16143)

Decision: **reject / not applicable**.

- The implementation lives in the removed `app/src/ai/agent_sdk/` surface (absent from this fork): `driver.rs`, `driver/mcp_startup.rs` (+224/−174 rewrite), `driver/mcp_startup_tests.rs`, and the `setup_observability.rs` `ProfileMcpServerStartup` step removal. App-side configured MCP server startup is itself a Removed System in this fork — MCP configuration and startup belong to the ACP agent process, so there is no ACP path to route this ordering fix through.
- The single shared-file hunk, `app/src/terminal/model/session/command_executor.rs` (+1), adds `ShellType::PowerShell if cfg!(windows) => "powershell.exe"` to the test `Command::new` shim so upstream's cross-platform runnable MCP fixtures work on Windows hosts. That is Windows-host test plumbing for the rejected agent-SDK tests; the fork is macOS-only and its retained test shim is unchanged.

### `569db5ecf` — Include sanitized setup command output in environment failures (#16197)

Decision: **reject / not applicable**. All six touched paths are under `app/src/ai/agent_sdk/` (`driver.rs`, `driver/environment.rs` + `environment_tests.rs`, `driver/error_classification.rs` + `error_classification_tests.rs`, `driver_tests.rs`), the Removed System absent from this fork. The sanitized 4-KiB setup-output attachment, Markdown/plain-text task-status variants, and retained-session recovery guidance all describe cloud-agent environment setup failures with no local terminal or ACP consumer here. Same standing family as the rejected `1b99853db` and the agent-SDK portion of `8dfda6bbd` (2026-09-29 audit).

### `77b373f15` — Open per-turn usage card above its footer icon (#16202)

Decision: **reject**. APP-6179 interaction fix for the per-turn request-metadata "Turn" panel — the APP-5720 `PricingTransparency` usage stack that was rejected and never ported (2026-09-16, 2026-09-17, and 2026-09-19 audits; the fork's persistence model has no usage tracking and its blocklist has no usage variants). Every hunk anchors on fork-absent symbols, verified absent by search:

- `super::usage::request_metadata_turn_view::RequestMetadataTurnView` and the entire `blocklist::usage` module — absent.
- `AIBlock` fields/methods `is_turn_panel_expanded`, `is_usage_footer_expanded`, `turn_panel_view`, `set_turn_panel_view`, `emit_turn_panel_toggled` — absent.
- `TerminalView::turn_panel_view_ids`, `RichContentMetadata::TurnPanel`, `RichContent::is_turn_panel`, `is_usage_footer` — absent (the fork's `RichContentMetadata` has no usage/turn-panel variants).
- Test-side `FeatureFlag::PricingTransparency` and `insert_dummy_ai_block` — absent.

The `rich_content.rs` hunks only remove the `TurnPanel` variant; no separable generic fix exists in the diff.

### `648d03b51` — Adding the option to switch teams on the client settings > teams page (#16120)

Decision: **reject / not applicable**. Teams is a Removed System:

- `app/src/settings_view/teams_page.rs` — the Teams settings page, absent from this fork.
- `app/src/workspace/util.rs` — the new `team_switcher_menu_items` helper requires `crate::workspaces::team::Team`, `WorkspaceAction::OpenNewWindowForTeam`, and `crate::server::ids::ServerId` multi-team machinery, all fork-absent (the multi-team stack has been rejected across many audits).
- `app/src/workspace/view.rs` — the hunk refactors the navbar team-switcher menu (which references `user_workspaces.team_uid_for_window` and `workspace.teams`) to call that helper; those anchors do not exist in the fork's workspace view.
- `app/src/view_components/clickable_text_input.rs` — the component itself is absent from this fork, and its only consumer on `upstream/master` is `teams_page.rs` (verified with `git grep ClickableTextInput` on the upstream tree). The generic widget tweaks (unchanged-content submit guard, `with_centered_text_label` → `with_text_label`, editor prefill + `select_all` on edit) have no fork consumer to port into, so nothing is separable.

## Verification

Zero ports; verification confirms the released tree is healthy. Run on `merge/upstream-2026-09-30` (= `main` + this audit doc, with `CARGO_PROFILE_DEV_DEBUG=0`):

- `cargo check -p warp --all-targets --message-format short` — pass.
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — pass.
- `cargo build -p warp --all-targets --message-format short` — pass.

Deleted-surface scan not applicable (no code changes); the standing scans from the 2026-09-26 audit remain valid for this tree.

`cargo clean` run after merge/tag/push.
