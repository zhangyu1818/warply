# Upstream Master Audit 2026-05-19

Range under review: `24e799977..b37688958`

Previous audited upstream tip: `24e799977 Add keybinding to toggle file navigation in Code Review (#11077)`

Current upstream tip reviewed: `b37688958 Add orchestration create environment modal (#10857)`

Total upstream commits in this incremental range: 26

Status: complete. All 26 upstream commits in this incremental range have a port/adapt/reject decision for this ACP-only macOS fork.

## Reviewed Commits

Reviewed commits in chronological order. No upstream commit was cherry-picked directly.

## Ported Or Adapted

- `b9a175372` Add "Create new branch" option to branch switcher: ported to retained context-chip branch switching. The branch picker can now synthesize a create-branch row for plausible unmatched queries and execute `git checkout -b <branch> --`.
- `5067be3c7` add approve to AGENT_FOLLOW_UP_INPUTS: ported to retained natural-language input classification so `approve` is treated like other agent follow-up replies.
- `b814dc307` Activate right horizontal tab after close: ported to retained workspace tab behavior. Horizontal tabs now activate the right neighbor after closing the active tab, while vertical tabs keep the previous-tab fallback.
- `2304ae94f` Fix asset-loading errors on WASM: partially ported. The retained generic URL asset loader now calls `error_for_status()` before reading response bytes. The upstream Web/WASM persistence gating was rejected because this fork has no Web/WASM host target.
- `13277cc38` Remove bootstrap block conditionals from input hot path: ported to retained terminal block lifecycle. Bootstrap placeholder blocks are started eagerly, empty pre-bootstrap blocks are excluded from long-running-command state, and `VisibleBootstrapBlock` is emitted when script execution output becomes visible.

## Rejected Or Not Applicable

- `119398514` CLI commands for API key management: rejected as old Agent SDK, Warp CLI cloud API-key, GraphQL permission, and cloud service behavior.
- `696c7c640` Add error pattern telemetry for third party harnesses: rejected as Agent SDK harness telemetry.
- `cc4383ce0` Update auth FTUX skip secret text: rejected as auth-secret/cloud-agent FTUX.
- `4d844f14e` Migrate to use `LocalOrRemotePath`: rejected for this range. It depends on upstream remote code-review/file-location architecture, Agent SDK paths, app-side MCP settings, Web/WASM editor paths, and broad `LocalOrRemotePath` plumbing that is not present in this fork.
- `23c908949` [3/n] Billing & usage dispatcher refactor: rejected as billing/usage/Teams/workspace cloud settings.
- `1c3b8824b` Bump claude plugin version: not applicable. The upstream Claude plugin manager paths are absent from this fork.
- `b3187d028` Migrate code review entrypoints for remote paths: rejected because it builds on the rejected `LocalOrRemotePath`/remote code-review architecture and reintroduces `RemoteCodeReview` rollout plumbing.
- `57f2d4c5e` Add onboarding verification skill: rejected as upstream app-bundled skill/onboarding process content. Skills belong to the ACP agent process, not the Warp app bundle or fork memory.
- `23f00966c` Connect with setup branches RPC: rejected as remote code-review branch RPC plumbing built on rejected remote diff-state architecture.
- `b9ee28cc4` [4/n] Add billing cycle usage section scaffold to v2 page: rejected as billing/usage UI.
- `44ed32abc` QUALITY-731: round-trip orchestrator agent short name through task records: rejected as ambient/orchestration/cloud-agent task records, shared-session viewer behavior, and upstream spec content.
- `dc2eb9bc4` [1/5] [Remote codebase indexing] pass embedding configs through: rejected as remote codebase indexing and remote AI search/settings plumbing not retained in this fork.
- `ad32ee269` Support diff stats chip for remote sessions: rejected because the useful UI behavior depends on the rejected remote code-review entrypoint and `RemoteCodeReview` gate. Local diff stats chips remain retained.
- `c72c4583f` Fix skill link in agent details panel: rejected as cloud/ambient agent details and app-side skill-link behavior.
- `f5fbd5a7a` [3/5] Reduce duplicate remote codebase index status pushes: rejected as remote codebase indexing, Agent SDK environment, and remote environment settings plumbing.
- `84f817c36` Throttle ambient agent task fetch based on RTC invalidation message: rejected as ambient/cloud agent task refresh plus upstream spec content.
- `bfb4e9803` [4/5] Add remote codebase incremental sync: rejected as remote codebase indexing, telemetry, protocol, and manager plumbing.
- `f49457b2d` Forward proxy errors for initialization error telemetry: rejected as telemetry payload/dependency changes. Existing local remote-server stderr logging remains retained.
- `1f314ffcc` Require proof of manual testing and human interactions: not applicable to fork product code; upstream contributor-process documentation was not ported.
- `be5b39ae7` [5/5] Handle remote codebase auto-index follow-ups: rejected as remote codebase auto-indexing, telemetry, settings, and protocol behavior.
- `b37688958` Add orchestration create environment modal: rejected as orchestration/cloud environment modal behavior.

## Verification

Verification run after applying the retained changes:

- `cargo nextest run -p warp -E 'test(test_format_create_git_branch_command_quotes_branch_and_appends_double_dash) | test(test_format_create_git_branch_command_escapes_single_quotes) | test(test_create_git_branch_menu_name_quotes_query) | test(test_create_git_branch_action_data_returns_branch_name) | test(test_create_git_branch_trims_whitespace_in_constructor) | test(query_matches_existing_name) | test(test_is_plausible_new_branch_name) | test(test_close_active_horizontal_tab_activates_tab_to_right) | test(test_close_last_horizontal_tab_activates_tab_to_left) | test(empty_pre_bootstrap_block_is_not_long_running) | test(non_empty_pre_bootstrap_block_can_be_long_running) | test(visible_bootstrap_block_event_fires_when_script_execution_becomes_visible) | test(test_script_execution_block) | test(test_basic_bootstrapping) | test(test_session_restoration_separator)'`
- `cargo fmt -- --check`
- `git diff --check`
- `cargo check -p warp --all-targets --message-format short`
- `cargo check --workspace --all-targets --message-format short`
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`
- Deleted-surface scans: required broad scans were run. No app-side MCP/skills hits were found. The broad cloud-product scan only hit weak-reference `upgrade` terminology and tokenizer vocabulary entries for `billing`/`credits`; a precise follow-up excluding tokenizer data returned no restored GraphQL, Sentry, telemetry, Agent SDK, ambient/cloud, billing/usage, managed-secret, or Warp Drive surfaces. Platform hits were retained bootstrap comments about ConPTY and SSH `ForwardX11=no` remote-terminal behavior.

No upstream specs, GraphQL schema, billing/usage UI, auth FTUX, Agent SDK harness behavior, ambient/orchestration UI, app-side skills/MCP, remote codebase indexing, telemetry, or Web/WASM host behavior were intentionally ported from this range.
