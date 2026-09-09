# Upstream Master Audit 2026-09-09

Range under review: `3cb7b96ca2..upstream/master` (16 commits)

Previous audited upstream tip: `3cb7b96ca2 [REV-2383] Add team selection to API key commands (#15806)`

Current upstream tip detected: `5879ee2d0 Scope managed secrets and harness credentials by team (#15862)`

Total upstream commits in this incremental range: 16

Status: triage complete. Two commits ported: the APP-5825 editor text layout-cache bypass (`742ca57b5`, retained `crates/editor` + `crates/warpui_core`) and the local CLI-agent parts of first-class Grok Build support (`2718b6658`, adapted). The remaining 14 commits are team-scoping verticals of the removed Teams/server-API surface, billing/usage, managed secrets, cloud environments, orchestration, GraphQL, the API-key modal, the removed agent-sdk harness, or flag promotions for fork-absent features; every shared-file hunk was verified to have no anchor symbol in the fork.

## Per-Commit Triage

### `742ca57b5` — [APP-5825] Bypass layout cache for editor text (#15831)

Decision: **port** (retained editor/warpui_core performance fix). Commit `4061e6360`.

Editor paragraph text now lays out through `TextLayoutSystem::layout_text_uncached` instead of retaining every frame in a `LayoutCache` (the `RenderState` is itself a block cache, so the retained cache only added memory pressure). Placeholder single-line layouts keep using the per-frame `LayoutCache` passed in from `Element::layout` (`layout_placeholder` gained a `layout_cache` parameter). `TextLayout::from_layout_context` became `for_render_state`, `RenderState::layout_context` dropped its throwaway-cache parameter, `strip_leading_unicode_bom` became `pub(crate)` for the new `layout_text_uncached` path (which also requests fallback fonts for uncached lines via `RequestedFallbackFontSource::UncachedText`), and the `text_layout_bench` criterion bench was added.

Omitted paths and adaptations:

- `char_cell_bench` Cargo registration — the bench source is absent from this fork (different upstream commit).
- `crates/editor/src/content/mermaid_diagram_tests.rs` — fork-deleted test file; upstream's change is the mechanical uncached-layout update of the unported #10431 mermaid-asset test architecture.
- The upstream mermaid test-suite hunks in `edit_tests.rs` (`layout_mermaid_block_for_test`, `mermaid_code_block`, `RenderLayoutOptions.render_mermaid_diagrams`) — they belong to the unported #10431 architecture; this commit's mechanical `TextLayout::new` without `LayoutCache` update applied to the fork's own mermaid test instead.
- `warpui_core::` → `warpui::` facade imports, `LayoutContext` import removal, and `RenderLayoutOptions`-by-value (`*layout_options`, `Copy`) adapted to the fork's editor API.

### `2718b6658` — Adds first-class Grok Build support (#14228)

Decision: **adapt-port** (local third-party CLI-agent surface; installer/telemetry/specs omitted). Commit `1e07ca044`.

Ported:

- `CLIAgent::Grok`: command prefix `grok` (auto-feeds the table-driven `detect` and the v1 event `resolve_agent`), display name `Grok Build`, `GROK_COLOR` brand color, bash-mode support, `Icon::GrokLogo`.
- `Icon::GrokLogo` enum variant + `bundled/svg/grok.svg` path mapping copied from upstream `4e09c695fb` (the model-picker `LLMProvider::Xai` hunk of that commit stays omitted — removed `/model` surface; the icon itself is a retained local asset now used by the CLI-agent footer).
- Listener: `CodexSessionHandler` generalized to `Osc9FallbackSessionHandler { agent }` shared by Codex and Grok; `parse_osc9_text` stamps the handler's agent; `try_parse` only accepts structured events whose `agent` matches the handler's agent; `is_agent_supported` includes Grok.
- Proactive listener on command detection for `CLIAgent::Codex | CLIAgent::Grok`: reintroduced `register_cli_agent_listener_without_session_start_event` (deleted at the fork baseline because it depended on `plugin_manager_for`) as the upstream function minus the `plugin_version`/wasm/`CLIAgentEventSource` machinery this fork's event model predates, wired into the command-detection timer before the footer/auto-open calls. This also repairs the pre-existing fork gap where a Codex session detected by command never received a listener.
- Rich Input submit strategy: Grok joins the `DelayedEnter` arm (fork location: `cli_agent_footer.rs`; upstream moved the function to `use_agent_footer/mod.rs` in an unported refactor).
- Tests: `("grok", CLIAgent::Grok)` detection, adapted `test_grok_public_configuration` (no `skill_command_prefix` in fork), Grok listener preference/agent-filtering tests, Codex other-agent structured-event filter test.

Omitted paths:

- `plugin_manager/grok.rs`, its `plugin_manager/mod.rs` registration, and `plugin_manager` tests — the one-click installer materializes Warp-owned hook files (`warp-plugin.json`, `bin/warp-plugin.sh`) into the user's `$GROK_HOME/hooks`; this is the rejected plugin-installer category (writes Warp-owned plugin entries into user tool configs) and the fork deleted `plugin_manager/` at the baseline. Users can still get rich OSC 777 events from community Grok hooks; Warp just listens.
- `app/src/server/telemetry/events.rs` `CLIAgentType::Grok` — telemetry surface removed.
- `specs/GH11727/**` — upstream specs.
- `SkillProvider::Agents` and `aliases()` hunks — fork has neither mechanism.
- `FeatureFlag::CodexPlugin` gate, `plugin_already_active` `try_parse` parameter, and `CLIAgentEventSource` — fork's listener API predates them.
- OSC 9 toast suppression stays Codex-only, matching upstream `2718b6658` (upstream did not extend the suppression to Grok).

### Rejected commits (removed surfaces, no fork anchors)

- `b81a8be9c` Scope runner discovery to selected team (#15809): `agent_sdk/runner*`, `run_agents_card_view.rs`, `orchestration_config_block.rs`, `server_api/factory.rs`, Oz CLI `warp_cli/src/runner.rs` all absent; no `RunnerCatalog`/`list_runners` anchors.
- `55b605ab1` [REV-2383] Scope oz run list by team (#15811): shared-file hunks thread `TeamContextResolver`/`UserWorkspaces` (removed) into `get_entries(&scope)` and window-team-change subscriptions; zero fork hits for `TeamContextResolver|UserWorkspaces`.
- `c120e1b43` Scope GUI environment pickers by window team (#15850): `cloud_environments/`, `orchestration/`, `handoff_compose.rs` absent.
- `171cfea05` Move usage display unit to Billing and Usage (#15852): `UsageDisplayUnit`/`billing_and_usage_dispatch` absent (billing removed).
- `0c127820f` Fix native workspace prompt limit CTA (#15853): `prompt_alert.rs` absent; enterprise-limit CTA depends on `AuthStateProvider`/`UserWorkspaces`/workspace billing metadata.
- `a68dbaa6f` Scope connected execution hosts by team (#15856): `connected_self_hosted_workers.rs` absent; `input.rs`/`input/agent.rs` hunks reference `ConnectedSelfHostedWorkersModel`/`HostSelector` with zero fork hits.
- `86617fb4c` Log GraphQL staging allowlist blocks (#15747): `crates/graphql` removed.
- `1f0cf55af` Validate Claude Code session restoration requirements (#15803): `agent_sdk/driver/harness/claude_code/**` removed; no `claude_transcript`/`wake_driver` anchors.
- `c0c71f0c8` Pass team scope explicitly through managed-secret operations (#15861): managed secrets + `aws/geap/harness_availability` credential files all absent.
- `558aba3bf` Fix default Agent selection in API key modal (#15825): `create_api_key_modal.rs` absent.
- `dda23c5c6` REMOTE-2111: promote PeriodicHandoffCheckpoints to PREVIEW_FLAGS (#15820): flag absent; REMOTE- handoff checkpoints are cloud remote-control semantics.
- `0d57d4bb2` Promote model selector prompt restoration to stable (#15870): `RestorePromptOnInlineModelSelectorSearch` absent; `/model` selector flows removed.
- `8b94c9c1c` Preserve service account team membership (#15866): `agent_sdk`, `server_api/team.rs`, `gql_convert`, `warp_server_client` all absent.
- `5879ee2d0` Scope managed secrets and harness credentials by team (#15862): all 22+ paths in removed `agent_sdk`/orchestration/managed-secrets/ambient-agent surfaces; no `managed_secret|harness_credential` anchors.

## Verification

- `cargo check -p warp_editor -p warpui_core --all-targets` — pass.
- `cargo check -p warp --all-targets` — pass.
- `cargo check --workspace --all-targets` — pass (`CARGO_PROFILE_DEV_DEBUG=0`).
- `cargo fmt -- --check` — pass (workspace).
- `cargo nextest run -p warp_editor` — 435/435 pass.
- `cargo nextest run -p warp -E 'test(cli_agent)'` — 94/94 pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156/156 pass.
- `cargo build -p warp --all-targets --message-format short` — pass (`CARGO_PROFILE_DEV_DEBUG=0`, 1m 44s), followed immediately by `cargo clean` (removed 29.9 GiB).
- Restoration scans over the `main` diff: no new hits in any changed file; full-repo scan hits are pre-existing allowed documentation/test/remote-terminal hits.
