# Upstream Master Audit 2026-05-18

Range under review: `53da56352..24e799977`

Previous audited upstream tip: `53da56352 Clip inline menu header to prevent split-pane overflow (#10811)`

Current upstream tip reviewed: `24e799977 Add keybinding to toggle file navigation in Code Review (#11077)`

Total upstream commits in this incremental range: 36

Status: complete. All 36 upstream commits in this incremental range have a port/adapt/reject decision for this ACP-only macOS fork.

## Reviewed Commits

Reviewed commits in chronological order. No upstream commit was cherry-picked directly.

## Ported Or Adapted

- `53264f409` Fix Mermaid SVG text rendering on WASM: adapted as a retained generic SVG rasterization fix in `crates/warpui_core`. The fork still uses the native image cache for local Markdown/Mermaid/SVG rendering, so the bundled Roboto fallback was ported even though the upstream bug was observed on Web/WASM.
- `18c98f266` Fix stale dependency definitions causing unnecessary rebuilds: partially ported. The `handlebars` crate no longer depends on `warpui` only to run parser tests, and the tests now run synchronously. The upstream `crates/app-installation-detection` cleanup was not applicable because that crate/path is not present in this fork.
- `24e799977` Add keybinding to toggle file navigation in Code Review: ported to retained local code review UI. `F` toggles the code review file navigation sidebar while focus is in the code review pane but outside descendant text editors, and the file-navigation button tooltip reflects the active keybinding.

## Rejected Or Not Applicable

- `936a2edff` Add model reasoning level for codex: rejected as old Agent SDK, ambient/cloud agent, cloud model selector, server API, and GraphQL behavior. ACP adapter configuration owns model/backend routing in this fork.
- `913124045` Respect CODEX_HOME and CLAUDE_CONFIG_DIR for writing config: not applicable. The touched old Agent SDK harness driver paths are removed; ACP adapters own their own config directories.
- `ad15522cb` Sync GraphQL schema with staging and add cynic types for tiered usage visibility: rejected as GraphQL/workspace cloud schema and billing/usage visibility.
- `d5d48329a` Wire up the usage data to in-memory types: rejected as cloud workspace usage/billing data plumbing.
- `01a0e9d0d` Display conversation searches by agent run title: rejected for this range. The upstream implementation depends on `warp_multi_agent_api`, ambient agent task data, and old cloud agent conversation search metadata that are not retained in the current ACP-only fork.
- `cc0064174` Inherit-share + eager Oz task creation for run_agents local children: rejected as orchestration/cloud child-agent task creation and shared-session inheritance behavior.
- `2fd4a785f` Hide legacy orchestration setting for v2: rejected as orchestration flag cleanup around deleted orchestration surfaces.
- `62fd80aed` Add more logging around starting shared sessions: rejected as deleted cloud shared-session creator/viewer/server startup logging. The fork retains ACP AgentView and SSH/remote terminal behavior, not cloud session sharing startup.
- `5d6421a00` Keep cloud handoff details panel closed: rejected as cloud handoff/shared-session viewer behavior.
- `8db29ac61` Enable cloud continuation from WASM tombstones: rejected as WASM cloud continuation and shared-session tombstone behavior.
- `05da7af3d` Simplify FTUE Modality Callouts: rejected as onboarding/FTUE plus upstream `specs/**` documents.
- `f652d6dfe` Disable handoff for orchestrated agents: rejected as cloud handoff/orchestrated-agent input behavior.
- `e791e2a84` Enable orchestration launch modal in prod: rejected as orchestration launch modal and onboarding asset surface.
- `60b3a2521` Promote cloud mode input v2 to prod: rejected as cloud-mode input feature-flag promotion.
- `2e925744f` Update the auth FTUX with more info: rejected as auth-secret FTUX and managed cloud secret UI.
- `70d6bea87` Use fresh metadata and updated iconography for cloud agent view entry block: rejected as cloud agent entry-block metadata/iconography.
- `e580d3747` Hydrate cloud mode input from viewed run config: rejected as ambient/cloud run config hydration.
- `95d38732e` Fix dormant Claude wake message race: rejected as old Agent SDK driver/event hydration and orchestration event streaming.
- `d14f1fca5` Add Codex auth secret types to CLI: rejected as cloud auth-secret CLI/API surface.
- `b495c6f53` Launch HandoffCloudCloud and CloudModeSetupV2: rejected as cloud handoff/cloud-mode feature-flag launch.
- `df1a5e8b0` Inline create-API-key flow on orchestration cards: rejected as orchestration cards, cloud API key/auth-secret flow, ambient agent UI, and upstream `specs/**`.
- `c41ca6e22` Back off harness auth secret fetch retries: rejected as Agent SDK harness auth-secret retry behavior.
- `fb8d9d95d` Add cloud conversations to the inline conversations menu: rejected as cloud conversation search/menu plumbing plus upstream spec. Retained inline conversations remain local ACP history.
- `67c215cca` Center child pill avatar within status overlay box: rejected as orchestration pill UI.
- `aa0149c7d` Persist killed child agents and rename Kill to Delete after finished state: rejected as orchestration child-agent persistence/UI.
- `6930c0a3e` Remove hover delays from orchestration pill details card: rejected as orchestration pill UI.
- `6d8ae465d` Implement preflight checks for third-party harnesses: rejected as old Agent SDK harness preflight/cloud agent setup behavior plus upstream `specs/**`.
- `13a7ae947` Fix model selector in cloud mode: rejected as cloud model selector behavior.
- `0e30b6002` Suppress cloud follow-up setup input sync: rejected as cloud follow-up/shared-session setup behavior plus upstream spec.
- `728ff1712` Add orchestration telemetry events: rejected as orchestration telemetry.
- `7f013c1c7` Plumb `--bedrock-role-region` into the Bedrock OIDC STS client: rejected as cloud credential/Agent SDK/BYOK CLI behavior.
- `ac17e9f85` Update winit dependency pin: not applicable. The current macOS-only fork no longer carries the upstream workspace `winit` dependency in `Cargo.toml` or `Cargo.lock`.
- `0f4bb5928` Add blog link for orchestration launch modal: rejected as orchestration launch modal copy.

## Verification

Verification run after applying the retained changes:

- `cargo fmt -p handlebars -p warpui_core -- --check`
- `cargo test -p handlebars`
- `cargo test -p warpui_core test_svg_text_rasterizes_with_bundled_sans_serif_fallback`
- `cargo nextest run -p warp -E 'test(acp_available_commands_are_visible_in_zero_state_for_active_acp_conversation) | test(acp_available_commands_update_emits_active_commands_event) | test(selecting_acp_command_with_input_hint_keeps_editor_open) | test(static_command_registry_matches_reviewed_app_owned_commands)'`
- `cargo fmt -- --check`
- `git diff --check && git diff --cached --check`
- `cargo check -p warp --all-targets --message-format short`
- `cargo check --workspace --all-targets --message-format short`
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`
- `cargo nextest run -p warp -E 'test(code_review)'`
- Deleted-surface scans: the required broad scans were run; noisy `upgrade` hits were inspected as weak-handle/test/tokenizer terminology. Follow-up precise scans without the noisy `upgrade` term returned no restored GraphQL, Agent SDK, ambient/cloud, telemetry, MCP/skills, or local Linux/Windows host surfaces. The remaining broad platform hits were retained bootstrap comments and SSH `ForwardX11=no` remote-terminal behavior.

No upstream specs, GraphQL schema, cloud/orchestration UI, old Agent SDK, auth-secret, telemetry, onboarding, or Web/WASM host behavior were intentionally ported from this range.
