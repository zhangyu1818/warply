# Upstream Master Audit — 2026-10-08

Incremental zero-port audit for `f571865ca..upstream/master` (`8c6a00c82`), 8 commits, completing the currently fetched upstream master tip. The previous audit is `upstream-master-audit-2026-10-07.md`.

## Summary

All 8 commits belong to one upstream workstream — the pricing-transparency / billed-cents series (PRs #16323, #16325, #16326, #16327, #16353) plus two auth (Firebase refresh) fixes and a README edit. Every commit lives on fork-removed surfaces (Warp server client auth, GraphQL, billing/usage/pricing, Teams workspaces, ambient/cloud agents) or fork-owned files. No shared-file hunk carries an anchor symbol present in this fork, so nothing is separable. No removed product surface was restored and no code was ported.

## Commit-by-Commit

### `44986c9fa` fix(auth): classify terminal Firebase verdicts on direct refresh (#16336) — rejected (removed surface)

Only touches `crates/warp_server_client/src/auth/session.rs` and `session_tests.rs`. `crates/warp_server_client/` is fork-absent (account auth and Warp server API client removed at baseline); Firebase token-refresh verdict classification has no fork path.

### `d807d9876` Fix Firebase refresh retry storms (#16099) — rejected (removed surface)

Same two files as above (`warp_server_client/src/auth/session*.rs`), fixing the auth-session refresh retry loop. Entirely inside the removed account-auth/server-client surface.

### `d888f2946` Plumb billed-cents GraphQL fields through the client (#16323) — rejected (removed surface)

PR 2 of the pricing-transparency series: cynic fragments in `crates/graphql`, `Tier.chargeUnit`/`ChargeUnit`, `RequestLimitInfo` cents fields, `ConversationUsageMetadata.total_billed_cost_in_cents`, and `UserWorkspaces::charge_unit()` resolution. All primary surfaces (`crates/graphql`, `app/src/workspaces/`, `app/src/ai/request_usage_model*`, `app/src/ai_assistant/`, billing pages, Teams pages) are fork-absent. Shared-file hunks verified anchorless in the fork:

- `app/src/ai/agent/conversation.rs`: the fork's copy has no `ConversationUsageMetadata` usage fields at all (`credits_spent`, `total_provider_cost_in_cents`, `usage_metadata_indicates_usage` all absent — the usage-tracking/APP-5720 stack was never ported).
- `app/src/pane_group/mod_tests.rs`, `app/src/settings/ai_tests.rs`: hunks initialize `credits_spent`/`RequestLimitInfo` fields that do not exist in the fork's structs (no `create_server_metadata`, no `RequestLimitInfo`).
- `app/src/ai/blocklist/history_model_tests.rs`, `conversation_details_panel_tests.rs`, `prompt/prompt_alert_tests.rs`, `agent_sdk/*`, `shared_session/*`: fork-absent files.

### `e38562d49` Show billed dollars for per-request and per-conversation usage (#16327) — rejected (removed surface)

PR 4 of the series: per-request/per-conversation dollar display in `blocklist/usage/*` views, usage popover, turn panel, and `view_util` helpers. `app/src/ai/blocklist/usage/` and `request_usage_model.rs` are fork-absent. Shared-file hunks verified anchorless:

- `app/src/ai/blocklist/view_util.rs`: no `format_usage`/`format_dollars`/`format_credits`/`UsageDisplayUnit`/`effective_usage_unit` — the fork's copy never received the usage-formatting stack.
- `app/src/ai/blocklist/block/view_impl/output.rs`: no `render_usage_button`/`usage_pill_text`/turn-panel symbols.
- `app/src/ai/blocklist/agent_view/agent_input_footer/mod.rs`: no `UsagePopover*`/`UserWorkspaces` subscription/`usage_display_unit`.
- `app/src/settings/ai.rs`, `app/src/ai/agent_conversations_model/entry.rs`, `app/src/ai/conversation_details_panel.rs`: no `usage_display_unit`/`usage_totals`/`conversation_cost_in_cents` symbols.

### `8736897db` Show included allowance and grant balances in dollars when billed in dollars (#16325) — rejected (removed surface)

PR 3 of the series: billing pages v1/v2, agent-profiles usage widget, bonus-grant notification, ambient trial pills. All primary surfaces fork-absent (`billing_and_usage_page*`, `agent_profiles_page`, `bonus_grant_notification_model`, `ambient_agent/first_time_setup`). The two fork-present agent_view files are anchorless: `agent_message_bar.rs` and `zero_state_block.rs` have no `render_ambient_credits_banner`/`ambient_credits`/`format_dollars`/`OzUpdates` symbols (ambient/cloud agents removed).

### `1090f336a` Label add-on packs in dollars when the catalog sells them in dollars (#16326) — rejected (removed surface)

PR 5 of the series: `app/src/pricing/` (new upstream directory), billing pages, `buy_credits_banner`, `enable_auto_reload_modal`, `build_plan_migration_modal`, `workspaces/workspace.rs`, `crates/graphql`. Every touched path is fork-absent.

### `e2d7f85db` Remove the client PricingTransparency flag; let Tier.chargeUnit decide (#16353) — rejected (removed surface)

Series cleanup removing the `PricingTransparency` rollout flag from the same usage/billing views and `view_util`. The flag itself never existed in this fork, and every touched symbol (`effective_usage_unit`, `usage_display_unit`, `rollup`, `pricing/addon_pack`, billing dispatch) is fork-absent per the checks above.

### `8c6a00c82` Revise early access request details in README (#16324) — not applicable

One-line upstream `README.md` edit about Warp Factories early-access marketing copy. The fork's `README.md` is fork-owned with no Factories/early-access content; the edited line has no counterpart.

## Verification

Zero-port audit: no source changes, so verification re-confirms the unchanged tree.

- `cargo fmt -- --check`: clean.
- `cargo check -p warp --all-targets --message-format short`: clean.
- `cargo check --workspace --all-targets --message-format short`: clean.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: passed.
- `cargo build -p warp --all-targets --message-format short`: succeeded, followed immediately by `cargo clean`.
- Restored-deleted-surface scans (`rg` over `app crates script Cargo.toml`): no new hits beyond the previously documented allowances.
