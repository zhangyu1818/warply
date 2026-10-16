# Upstream Master Audit — 2026-10-16

Incremental audit for `dc0b890ba..upstream/master` (`4c509880f`), 13 commits. The previous audit is `upstream-master-audit-2026-10-15.md`.

## Summary

Two adapted ports and eleven not-applicable/rejected commits. `53f8a71ef` preserves the explicit order of agent request inputs across tasks — retained AgentView request/history behavior that maps directly onto the fork's ACP prompt construction (`all_inputs()` feeds `acp_prompt_from_request`, so `HashMap` task ordering made multi-task action-result prompt order nondeterministic and overwrote all but the last task exchange on the response stream). `f4868ff31` adds the Antigravity CLI session listener, event aliases, second command prefix, and DelayedEnter rich-input strategy; its prerequisite `84b11bb1d` (#12380, detection/branding/icon/asset) was never ported and is ported now as part of the same feature. The other eleven commits live in fork-absent surfaces (`agent_sdk`, billing/GraphQL, server_api/prewarmed sandbox, plugin_manager, orchestration injection, external-harness ACP stack) verified per file; `2d0c6fd4b` is an empty commit whose tree is identical to its parent.

## Commit-by-Commit

### `522481cd0` Use task-snapshot deferred inventory with real checkout skill (#16461) — not applicable

Server-coupled REMOTE-2960 client half over the task-snapshot contract: client-built checkout recipes, synthetic `/deferred-repositories`, the bundled `factory-deferred-repositories` skill, `FactoryDeferredRepositories`/`WARP_FACTORY_ID` gating, `taskGitCredentials` scope, and `OZ_WORKSPACE_ROOT` driver exports. Every touched path is fork-absent (`app/src/ai/agent_sdk/**`, `app/src/ai/skills/bundled_tests.rs`, `resources/bundled/skills/**` — removed surface). Nothing separable.

### `94abbc17b` Require Codex notification plugin 0.4.3 (#16368) — not applicable

Raises the Codex notification-plugin version floor in `app/src/terminal/cli_agent_sessions/plugin_manager/{codex.rs,codex_tests.rs}`. The fork removed the plugin-manager surface (no `plugin_manager/` directory; listener runs without `CodexPlugin`).

### `53f8a71ef` Preserve explicit order of agent request inputs across tasks (#16328) — accepted (adapted)

`RequestInput.input_messages` becomes `Vec<(TaskId, Vec<AIAgentInput>)>` (ordered batches, a task may recur), `for_task`/`for_actions_results` push batches, `all_inputs()` flattens in order, and `update_for_new_request_input` groups batches per task with an `IndexMap` while accumulating every task exchange on `added_exchanges_by_response` instead of overwriting. Applied via `git diff 53f8a71ef^ 53f8a71ef -- <retained paths> | git apply --3way`.

Adaptations (upstream core preserved):

- **`app/src/ai/blocklist/controller.rs`**: upstream-exact `Vec<(TaskId, Vec<AIAgentInput>)>` field with the upstream doc comment, per-batch pushes in `for_task` (fork name for upstream `new`) and `for_actions_results` (fork name for upstream `from_action_results`), and the upstream `all_inputs` flat_map. In-file `acp_prompt_from_request` tests converted `HashMap::from` → `Vec::from` exactly as upstream's test hunks. The steering-input and orchestration-event piggyback push hunks have no fork anchors (`steered_input`, `OrchestrationEventService`, `will_trigger_server_subagent` are fork-absent) and are omitted.
- **`app/src/ai/agent/conversation.rs`**: upstream-exact `IndexMap` grouping loop and accumulate-or-insert `AddedExchange` handling (the same idiom the fork's `append_reassigned_exchange` already uses); `use indexmap::IndexMap;` added. `should_hide` stays the fork's `false` (the fork never ported `is_passive_suggestion_trigger` on this path) instead of upstream's passive-suggestion check.
- **Tests**: `history_model_test.rs` (fork name for upstream `history_model_tests.rs`) — the eight `HashMap::from` → `Vec::from` conversions are upstream-exact, and the multi-batch retention test is ported as `multi_task_request_inputs_preserve_order_and_retain_every_exchange` with the fork's harness (`BlocklistAIHistoryModel::new(vec![], &[])`, 3-arg `start_new_conversation`, `create_optimistic_cli_subagent_task` without the upstream `_for_test` wrapper) and `AIAgentInput::ResumeConversation` batches where upstream's third batch used `AgentWake` (fork-absent, rejected 2026-09-29 as server-coupled). The `initialize_output_for_response_stream` / `warp_multi_agent_api::StreamInit` tail of the upstream test is pre-existing upstream test body over fork-absent MAA machinery and is not ported.
- **Omitted paths and reasons**: `app/src/ai/agent/request_metadata_tests.rs`, `app/src/ai/blocklist/controller_tests.rs`, `app/src/ai/blocklist/controller/startup_queue_tests.rs`, `app/src/ai/blocklist/handoff/pipeline_tests.rs`, `app/src/ai/agent_sdk/driver_tests.rs` — fork-absent files (pure `HashMap::from` → `Vec::from` hunks with no fork counterpart beyond the retained conversions above).

### `b21b0e50e` Include initial cloud file attachments as ACP resource links (REMOTE-3502) (#16486) — not applicable

Cloud-run task-file attachments formatted as `resource_link` blocks on the initial ACP harness prompt: `agent_sdk/driver/attachments*`, `agent_sdk/driver/harness/acp/**`, `agent_sdk/mod.rs` — all fork-absent (the external-harness ACP stack rejected in the 2026-10-10 audit).

### `440c0f07a` Preserve fractional billing usage in the native client (#16474) — not applicable

Fractional billing/usage display over `app/src/settings_view/billing_and_usage/**`, `app/src/workspaces/gql_convert.rs`, `crates/graphql/**`, `crates/warp_tui/**` usage menu, and `tui_export.rs`. Billing, GraphQL, and warp_tui are removed surfaces with no fork files present.

### `7ae920862` ACP: async inbound requests, typed agent capabilities, capability-aware attachments (#16497) — not applicable

`AcpConnection` out-of-band inbound request handling, typed `AgentCapabilities`, and capability-aware attachment shaping — all under `app/src/ai/agent_sdk/driver/harness/acp/{connection,protocol,attachments}*` (fork-absent external-harness ACP client, distinct from the fork's `app/src/ai/acp/`). The one remaining file, `app/src/ai/attachment_utils.rs`, is also fork-absent; the `#[derive(Clone)]` on `DownloadedAttachment` has no fork consumer.

### `6b3a036ad` feat(server-api): add GraphQL config and sandbox callbacks (#16477) — not applicable

Prewarmed-sandbox GraphQL config/status/heartbeat client over `app/src/server/server_api/**`, `crates/graphql/**`, `crates/managed_secrets/**`, `crates/warp_graphql_schema/**`. All removed surfaces.

### `f4868ff31` feat(cli-agent): support Antigravity CLI session listener and rich input strategy (#16373) — accepted (adapted, with prerequisite `84b11bb1d`)

Antigravity (`agy`) joins `DefaultSessionListener` with canonical plus alias event names (`task_started`/`task_complete`/`prompt_required` mapped in the v1 parser), `antigravity` becomes a second command prefix, and the rich-input strategy becomes `DelayedEnter`. The prerequisite `84b11bb1d` (#12380: `CLIAgent::Antigravity` variant, white brand color, `Icon::AntigravityLogo` + `antigravity_cli.svg`, `"agy"` prefix, dark `brand_icon_color` arm) had never been ported and is ported here as the same feature. Both upstream diffs were applied with `git apply --3way` onto the fork.

Adaptations (upstream core preserved):

- **`app/src/terminal/cli_agent.rs`**: upstream-exact variant, `ANTIGRAVITY_COLOR`, display name, icon, brand color, and the `brand_icon_color` dark-icon arm. The fork keeps the singular `command_prefix()` (`"agy"`) and expresses upstream's `&["agy", "antigravity"]` slice through the fork's established `detect()` second-prefix special case (same idiom as Vibe/`vibe-acp` and Kiro/`kiro`). `supports_bash_mode` unchanged — upstream never added Antigravity there.
- **`app/src/terminal/cli_agent_sessions/event/v1.rs`**: upstream-exact alias arms (`session_start | task_started`, `stop | task_complete`, `agent_needs_input | prompt_required`); `resolve_agent` gains the Antigravity second prefix (upstream's slice-`contains` semantics expressed in the fork's singular-prefix matcher). Upstream's `stop_failure` arm in the same hunk belongs to an unported later change and is omitted.
- **`app/src/terminal/cli_agent_sessions/listener/mod.rs`**: upstream-exact `is_agent_supported` and `DefaultSessionListener` arms (WarpTui/OhMyPi variants fork-absent).
- **`app/src/terminal/view/cli_agent_footer.rs`** (fork name for upstream `use_agent_footer/mod.rs`): upstream-exact `DelayedEnter` arm.
- **`crates/warp_core/src/ui/icons.rs` + `app/assets/bundled/svg/antigravity_cli.svg`**: upstream-exact icon registration and asset copy.
- **Tests**: detect row `("agy", CLIAgent::Antigravity)` (from #12380) added to the fork's `test_detect_known_agents`; `antigravity_notifications_are_supported` ported into the fork's inline listener test module with the fork's 2-arg `try_parse` (upstream's third arg is plugin-transport plumbing the fork lacks); `test_rich_input_submit_strategy_for_antigravity` ported into a new `cli_agent_footer.rs` test module.
- **Omitted paths and reasons**: `app/src/server/telemetry/events.rs` `CLIAgentType::Antigravity` and the `From<CLIAgent> for CLIAgentType` arm — telemetry removed. `plugin_manager/mod.rs` arm — plugin manager removed. `supported_skill_providers` fn and its `Antigravity => &[]` arm — skills removed. `specs/GH11368/**` and `specs/GH13147/**` — upstream specs rejected. `crates/input_classifier` hunks — the fork's `ONE_OFF_SHELL_COMMAND_KEYWORDS` already contains `"agy"` and its `agy doctor` test already exists.

### `034ce7891` refactor(app): centralize command-line launch mode construction (#16478) — not applicable

Moves sandbox/computer-use launch-flag derivation into `LaunchMode::command_line` and `Args::global_options`. The fork has no `LaunchMode::CommandLine` variant, no `CliCommand`/`AgentCommand`/`GlobalOptions` in the minimal `warp_cli`, and no `agent run` command; the refactor's entire surface (`app/src/lib.rs` hunk included) targets that removed CLI surface.

### `64f6fe878` Stop injecting orchestration events into external-harness (ACP) conversations (#16507) — not applicable

P0 guard requiring `conversation.driver().owns_turn_lifecycle()` before orchestration-event injection/piggyback, plus `OrchestrationEventService` drop-on-unbind. Anchors (`ConversationDriver`, `OrchestrationEventStreamer`/`orchestration_event_streamer*.rs`, `controller/startup_queue.rs`, `unbind_native_prompt_conversation`, `triggers_server_subagent`) are all fork-absent removed orchestration/native-setup machinery.

### `40214fdef` refactor(agent-sdk): extract shared agent setup from the driver run path (#16479) — not applicable

Pure `agent_sdk` refactor (`driver/agent_setup*`, `AgentDriver::create`) for the prewarmed-sandbox stack. Fork-absent surface.

### `2d0c6fd4b` Fix ACP test helper after #16497 squash (test build on master) (#16506) — not applicable (empty commit)

The commit's tree is byte-identical to its parent `40214fdef` (verified: equal `git cat-file` tree hashes); GitHub preserved the squash commit with no diff. The fix it describes targets `AcpHarnessRunner::prompt_text_for_test` in the fork-absent `agent_sdk` harness anyway.

### `4c509880f` ACP: steer follow-ups mid-turn and route interrupts through session/cancel (#16499) — not applicable

Steer follow-ups mid-turn and `session/cancel` interrupt routing for external-harness (ACP) conversations: `agent_sdk/driver/harness/acp/{mod,turn_driver}*` (new `turn_driver.rs`), `blocklist/controller/external_harness.rs` (`ExternalHarnessCancel`, `cancel_via_external_harness`, `CancellationRoute`), `terminal/local_tty/terminal_view_adaptor.rs` (shared-session steering), and shared-session cancel plumbing in `terminal/view.rs`. Every hunk anchors on the external-harness/`ConversationDriver`/shared-session architecture rejected in the 2026-10-10 audit; the fork's own ACP client owns all conversation turns, so there is no second driver to route cancels to and nothing separable.

## Verification

- `cargo check -p warp --all-targets --message-format short` — pass.
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- Focused suites: `-E 'test(history_model)'` 7 passed, `acp_prompt` 4 passed, `-E 'test(cli_agent)'` 102 passed, `antigravity` 2 passed; `cargo nextest run` in `crates/input_classifier` — 3 passed.
- `cargo build -p warp --all-targets --message-format short` — pass, followed by `cargo clean`.

## Boundary Scans

Post-merge `rg` scans for restored deleted surfaces: the eleven changed source files have zero hits across all three scan patterns (cloud/auth/billing/telemetry wording, MCP/skills, Linux/Windows/WASM platform). Remaining repo-wide hits are pre-existing, previously adjudicated: retained remote-terminal/SSH files (`crates/remote_server/src/ssh.rs`, `remote_command_executor.rs`, `local_or_remote_path_tests.rs`), the zsh bootstrap's remote-OS detection, and legacy local-object/naming wording untouched by this merge.
