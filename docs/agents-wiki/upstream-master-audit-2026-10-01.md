# Upstream Master Audit 2026-10-01

Range under review: `648d03b51..upstream/master` (7 commits)

Previous audited upstream tip: `648d03b51 Adding the option to switch teams on the client settings > teams page (#16120)`

Current upstream tip: `0b4b71f84 Add ChatGPT subscription support (#16216)`

Total upstream commits in this incremental range: 7

Status: triage complete. One adapted port (`6d6eac300`); six commits rejected or not applicable.

## Per-Commit Triage

### `d1a325b0e` — Carry Factory experiments through cloud agent driver (REMOTE-3251)

Decision: **reject / not applicable**. All twelve touched paths live in Removed Systems absent from this fork: `app/src/ai/agent_sdk/**` (driver/config_file/integration/schedule/ambient, verified absent) and `crates/cloud_object_models/src/{cloud_agent_config,scheduled_ambient_agent}.rs` (the crate directory does not exist here). The uninterpreted server-owned experiment map is cloud-task snapshot transport between the Warp server and the cloud agent driver; there is no ACP or local path to route it through.

### `7cef787ea` — Two windows opened when starting on Windows (#6004)

Decision: **reject / not applicable**. The fix exempts the Windows crash-recovery watcher child from single-instance startup-args forwarding. Touched paths: `app/src/app_services/windows/mod.rs` and `app/src/app_services/linux/mod.rs` — both absent (this fork's `app_services/` contains only `mac.rs` and `mod.rs`), plus the two `app/src/lib.rs` match arms for `app_services::{linux,windows}::StartupArgsForwardingError::IgnoredForCrashRecoveryProcess`, whose enum has zero references in the fork (no single-instance URL forwarding path exists on the macOS-only host). No macOS behavior in the diff.

### `556046e29` — Serialize ambient workload token refreshes (#16167)

Decision: **reject / not applicable**. Both files (`crates/warp_server_client/src/base_client.rs`, `base_client_tests.rs`) are in the removed Warp server client crate, absent from this fork. The NSC workload-token mutex/once-cell serialization serves ambient-agent startup against the Warp server; no retained consumer.

### `c0b7cf95f` — Bump minimum codex plugin version (#16214)

Decision: **reject / not applicable**. Both hunks are inside `app/src/terminal/cli_agent_sessions/plugin_manager/codex.rs` (+tests): the `MINIMUM_PLUGIN_VERSION` 0.4.0 → 0.4.1 bump and test-fixture updates for the Warp-managed Codex plugin auto-installer gated by `FeatureFlag::CodexPlugin`. The fork is pre-`plugin_manager` lineage (`cli_agent_sessions/` has no `plugin_manager/` directory; no `CodexPlugin` anchors) and has no Warp-managed plugin install — external CLI installs are detected, not managed. Same standing family as the rejected Grok plugin installer (2026-09-09 audit) and the trivially-satisfied `2012bacff` removal (2026-09-11 audit).

### `6d6eac300` — Fallback to file paths after empty combined completions (#16213)

Decision: **adapted port** (follow-up to the #16093 combined-completions stack and the `9c870be87` empty-native fallback port from the 2026-09-22 audit). Retained terminal completions behavior: when Warp + native shell completions are both enabled and both sources return empty, Tab now falls back to file-path-only Warp completions instead of showing no menu; nonempty native results still win over file paths; the phase-two native handoff became abortable and cancels the session's active commands when the completion request is aborted.

- `app/src/terminal/input.rs`: applied verbatim via `git apply --3way` of the upstream path diff (clean). Phase-one `WarpThenNative` spawn now threads `matcher`, `completion_context`, and `session_env_vars` through the result tuple into `dispatch_native_shell_completions`; that function gained the three parameters plus `#[allow(clippy::too_many_arguments)]`, renames the recv binding to `native_suggestions`, falls back to `completer::suggestions` with `CompletionsFallbackStrategy::FilePaths` + `suggest_file_path_completions_only: true` on an empty native reply, switches `ctx.spawn` → `ctx.spawn_abortable`, and cancels active session commands on abort.
- `app/src/terminal/input_tests.rs` (this fork's `app/src/terminal/input_test.rs`, same child-module lineage): all hunks ported. The stale-dispatch test gained the real `SessionInfo`/`simulate_directory_for_completion` fixture and the three new dispatch arguments; the `CancellationTrackingExecutor`, `aborting_native_completions_after_empty_specs_cancels_session_commands` test, `respond_to_native_shell_completions` helper, and the two `combined_completions_*` tests are upstream code verbatim. Recorded intentional adaptations, both forced by the fork's pre-`ExecuteCommandOptions` `CommandExecutor` trait (four `execute_command` parameters, `anyhow::Result<CommandOutput>`): the `ExecuteCommandOptions` import and the `_execute_command_options` stub parameter are omitted, matching the fork's existing `RecordingCommandExecutor` idiom in `current_prompt_test.rs`. Import placement follows this fork's header layout.

Focused verification: `cargo nextest run -p warp -E 'test(combined_completions_) | test(input_tab_does_not_ask_the_shell_when_bundled_specs_are_non_empty) | test(input_tab_asks_the_shell_once_when_bundled_specs_are_empty) | test(native_completions_after_empty_specs_bails_when_stale) | test(aborting_native_completions_after_empty_specs_cancels_session_commands)'` — 6 passed.

### `582cc97cf` — Fix Bedrock OIDC credential reuse in cloud agents (#16185)

Decision: **reject / not applicable**. All five paths are fork-absent: `app/src/ai/agent_sdk/mod.rs` (Removed System), `app/src/ai/aws_credentials.rs`, `app/src/settings_view/warp_agent_page.rs`, and `crates/ai/src/api_keys.rs`(+tests) — the fork's trimmed `crates/ai` has no `api_keys` module (no `ApiKeyManager`, no Bedrock/OIDC credential machinery). The fix clears loaded AWS credentials when the cloud agent driver switches refresh strategy; its runtime owner is the removed cloud-agent/agent-SDK stack, with no local or ACP consumer here.

### `0b4b71f84` — Add ChatGPT subscription support (#16216)

Decision: **reject**. The feature core is Warp-account ChatGPT-subscription linking billed through Warp: GraphQL `start_chatgpt_link`/`disconnect_chatgpt`/`get_chatgpt_connection`, `ChatGPTSubscriptionModel` on the server AI client, `AuthManager`/`AuthStateProvider` integration, a billing-and-usage ChatGPT usage card, a cloud-synced `did_show_chatgpt_plan_modal` setting, a `warp://chatgpt-link` URI host, `FeatureFlag::ChatGPTSubscription`, and `skip_chatgpt_subscription` server-request semantics. Account auth, billing, Teams/workspace, GraphQL, and Warp server APIs are Removed Systems; there is no local or ACP boundary this can run through (model routing belongs to ACP adapter configuration in this fork).

Every shared-file hunk was checked and anchors on fork-absent symbols (verified by search):

- `app/src/ai/llms.rs` is a one-line re-export stub here; the upstream hunks rebuild `LLMPreferences`/BYO machinery that does not exist (`is_using_api_key_for_provider`, `ByoKeySource`, `ApiKeyManager`, `TeamUpdateManager`, `refresh_authed_models_for_team_uid`).
- Blocklist hunks require `RenderableAIError::ChatGPTSubscriptionError`, `RenderableAIError::GeminiEnterpriseCredentialsExpiredOrInvalid`, `AIBlockAction::ContinueWithWarpCredits`, `warp_multi_agent_api::response_event::stream_finished::Reason::ChatgptSubscriptionError`, `FailedOutputPresentation::GeminiEnterpriseCredentialsExpiredOrInvalid`, `AIRequestUsageModel`, and `AgentConversationData.use_warp_credits_instead_of_chatgpt` — all absent. The persistence field is documented upstream as "Sent as `skip_chatgpt_subscription` on every request", i.e. Warp-server request state, not retained local data.
- Terminal hunks anchor on `maybe_add_buy_credits_banner`/`BuyCreditsBannerDisplayState` (`input/common.rs`), `query_model_picker_choices`/`ModelSearchItem` (`input/models/data_source.rs`, absent), `AIBlockEvent::ContinueWithWarpCredits` (`view.rs`), and the new persistence field (`load_ai_conversation.rs`).
- `root_view.rs`/`one_time_modal_model.rs` hunks wire `notify_workspace_shown`/`on_workspace_shown` through auth-onboarding state transitions; the fork has no `OneTimeModalModel`, no auth onboarding, and no cloud-preferences syncer.
- `app/src/features.rs` is a one-line `pub use warp_core::features::*;` here (no `enabled_features` to extend); `app/src/lib.rs` hunks need `ChatGPTSubscriptionModel`, `AuthManagerEvent`, `ApiKeyManagerEvent`, `AIRequestUsageModel`.
- The generic-looking helpers (`error_primary_cta_button`, `manage_button_label`/`manage_url`, `chatgpt_subscription_message_with_links`) exist only to render the rejected feature's error card and have no fork consumer to port into.
- Test-only `..Default::default()` churn in `AIAgentInput` fixtures (integration/blocklist tests) tidies field lists whose `origin`/`author`/`source_message` fields were never added to this fork's fork-adapted types; not applicable.

## Verification

Run on `merge/upstream-2026-10-01` (with `CARGO_PROFILE_DEV_DEBUG=0`; a mid-session disk-full failure corrupted `target/` artifacts and was resolved with `cargo clean` before these runs):

- `cargo check -p warp --all-targets --message-format short` — pass.
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — pass.
- `cargo build -p warp --all-targets --message-format short` — pass.
- Deleted-surface scans (`rg` for auth/billing/GraphQL/Sentry/telemetry, MCP/skills, and platform-gated code) — no new hits; the only diff under `app/src/` is `terminal/input.rs` + `terminal/input_test.rs`.

`cargo clean` run after merge/tag/push.
