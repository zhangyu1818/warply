# Upstream Master Audit 2026-05-26

Range under review: `b37688958..fc110333a`

Previous audited upstream tip: `b37688958 Add orchestration create environment modal (#10857)`

Current upstream tip reviewed: `fc110333a Tab group feature flag and entry points (#11486)`

Total upstream commits in this incremental range: 96

Status: complete. All 96 upstream commits in this incremental range have a port/adapt/reject decision for this ACP-only macOS fork.

## Reviewed Commits

Reviewed commits in chronological order. No upstream commit was merged wholesale; retained changes were manually ported or adapted to the fork architecture.

## Ported Or Adapted

- `427e1ab16` Clip terminal view column to prevent split-pane footer overflow: ported to retained terminal rendering by wrapping the terminal column in `Clipped`.
- `2fe9d43ca` Fix stale git diff chip and code review button: ported to retained local git prompt-chip behavior. Diff-stat shell fallback and repository watcher races now clear stale cache state when the active repository detaches.
- `f8a099380` Make worktree menu paths readable for long entries: ported to retained workspace menus using user-friendly paths, start clipping, and tooltips.
- `8e3c8eb89` Reset pane flex when double-clicking on a pane divider: ported to retained pane group/tree behavior and tests. Double-clicking a divider resets only the containing split branch.
- `62f2668ba` Use async presentation on macOS where possible: ported to retained macOS Metal rendering and AppKit host view integration.
- `a2d6833b8` Fix review comment routing to use visible terminal views only: ported to retained local code-review side panel routing so comments target visible terminal views.
- `b0897e597` Fix 1px window restore after macOS green-tile: adapted to the fork's simplified macOS windowing by filtering too-small persisted window bounds on save and restore.
- `21b7b6427` Remove "open repo" cta for remote sessions: adapted to retained remote-session code-review UI. Remote empty states no longer show the local "Open repository" action.
- `29d88e468` Fix deadlock in secret redaction due to mutex ordering mismatch: ported to retained terminal secret-redaction paths by sharing compiled secret regex state without the old lock-order cycle.
- `fa2e570bf` Add podman exec/run support for warpify subshells: partially ported for retained SSH/subshell Warpify behavior. The fork accepts Docker/Podman subshell detection, uses 7-bit ST in bootstrap assets, and chunks large container bootstrap writes. Upstream Windows/MSYS paths remain rejected.
- `49bbe78e1` Open Mermaid diagrams in lightbox: partially adapted to retained notebook editor rendering. Mermaid code-block footers can open the generated diagram source in the retained lightbox; upstream editor event plumbing that depends on newer AgentView code was not ported.
- `61e0fd354` Skip run command request when disconnected: ported to retained SSH remote-server command execution by exposing client disconnected state and skipping run-command requests after disconnect.

## Rejected Or Not Applicable

- `1bb368b6a` Add sleep auto handoff to cloud: rejected as cloud handoff behavior.
- `c6b8f7fdd` Change owner of vertical tabs: not applicable to retained product behavior.
- `52a328fad` fix(feedback): confirm before filing issues: rejected as hosted feedback/issue filing surface.
- `7f04a3542` Fix orchestration pill hover contrast: rejected as orchestration/cloud-agent UI.
- `c784aad7e` Update stakeholders: not applicable to fork product code.
- `fba90a510` Skip AgentTips with unresolved keybinding placeholders: rejected as upstream AgentTips/onboarding help surface not retained in this fork.
- `910f7568c` Rename empty orchestration environment label: rejected as orchestration/cloud environment UI.
- `2be6f35d6` Add-on credits panel and autoreload UI refactor: rejected as billing/credits/autoreload account UI.
- `1c4b181e4` Fix orchestration picker menu widths: rejected with the orchestration picker surface. The generic width idea has no retained caller.
- `8c72bd5e8` Add View in Oz to orchestration pill menu: rejected as Oz/orchestration UI.
- `600715949` Downgrade missing snapshot script error log: rejected as Agent SDK snapshot-script behavior.
- `d27983252` Custom model TOS hyperlink coloring: rejected as old custom-model/cloud model settings UI.
- `6f86cadb7` Promote git credential refresh to production: rejected as upstream rollout/feature-flag plumbing not present in this fork.
- `e659efbb0` Add feedback skill setting: rejected as app-managed skill/settings behavior.
- `d194b4c7f` Update add-on credits billing v2 UI: rejected as billing/usage UI.
- `1194bcbee` Pipe connected worker hosts into client host selectors: rejected as cloud worker host selector behavior.
- `340a3290b` Refine onboarding verification branch coverage: rejected as onboarding/app-bundled skill process content.
- `3e9bc88fc` Don't block on codebase indexing for self hosted oz runs: rejected as Oz/self-hosted cloud agent indexing behavior.
- `831de4bec` Use legacy changelog action for non-stable channels: rejected as upstream autoupdate/changelog surface. The fork uses Sparkle/GitHub Releases.
- `f7e19b5ed` Render usage section rows: rejected as billing/usage UI.
- `77b7c9e03` Clean up TMUX SSH warpification setting: reviewed but not ported in this batch. The upstream patch conflicts with fork-specific SSH Warpify settings and old telemetry/banner structure; retained SSH setting cleanup should be handled separately.
- `3bd21f82f` Enable cmd-O and @ context on remote SSH session: rejected for this range because it depends on upstream `LocalOrRemotePath`, remote code-review entrypoints, and broad remote editor plumbing not present in the fork.
- `eaf275869` Fix agent dropdown rendering in new API key modal: rejected as old API-key/cloud model settings UI.
- `fdd74928d` Move feature flag initialization logic out of lib.rs: rejected because the fork intentionally removed many upstream rollout flags and does not need the old initialization surface.
- `ac6e8137e` Add more profiling file types to .gitignore: not applicable to retained product behavior.
- `d2e9affcd` Split embedded assets to warp_assets crate: rejected for this batch as broad build-system churn that would reintroduce deleted asset surfaces unless separately audited.
- `f653e1f1d` Delete unused new_from_server_update function: not applicable. The target old server-update path is already removed or diverged in this fork.
- `f41b4e0a9` Hide admin panel link for non-enterprise billing pages: rejected as Teams/billing UI.
- `8ca81cf3a` Surface detailed GitHub auth errors: rejected because the upstream path is tied to old hosted auth/account flows. Retained local git/GitHub error handling should be improved separately if needed.
- `04e0c2297` Fix orchestration child restore persistence: rejected as orchestration/cloud-agent persistence.
- `2ba4b646d` Remove custom models options from cloud runs: rejected as cloud run/model settings behavior.
- `8578951de` Fix multiple connection across tabs causes unable to load content: rejected as upstream cloud/session connection behavior.
- `3d940d78a` Add host label for remote repos and files: rejected because it builds on the rejected remote code-review/file-location architecture.
- `3457feef2` Update teams page seat-limit upsell copy: rejected as Teams/billing UI.
- `0112f79d6` Add reload credit confirmations for team actions: rejected as Teams/billing/credits UI.
- `500f38b5f` Ship updated conpty: rejected as Windows host dependency.
- `94faa698f` Allow AI for all plans provided they have autoreload enabled: rejected as plan/billing/cloud AI gating.
- `173363bd9` Enable SoloUserByok and BillingAndUsagePageV2 in stable: rejected as rollout for removed model/billing surfaces.
- `467daa883` Add logged-out UI bug reproduction skill: rejected as app-bundled skill and logged-out UI test surface.
- `c4cd0c441` Add run_agents profile permission: rejected as cloud run/orchestration permission behavior.
- `424e3d827` Allow common skills install failures to continue scripts: rejected as app-managed skill install behavior.
- `790d314bc` Enable CustomInferenceEndpoints feature flag in stable: rejected as old custom-model rollout plumbing. ACP and terminal suggestions keep their fork-owned config paths.
- `f3dd3768f` Add Intel HD Graphics 2500 to buggy iGPU adapters: not applicable because the upstream WGPU resource path is absent from the fork.
- `bfdc42feb` Do not include cloud agent metadata on passive suggestions requests: rejected as cloud agent metadata path. Retained terminal suggestions do not use Warp cloud metadata.
- `81c938993` Move some types to warp_server_client: rejected as Warp server-client crate restructuring.
- `4ea8a1fb4` Add shared session QR code flow: rejected as cloud shared-session UI.
- `0385ab1a2` Clarify re-review conditions in docs: not applicable to fork product code.
- `cb4fe42a9` Update filesystem watch filters: rejected for this batch because the upstream watcher paths are tied to codebase-indexing behavior removed from the fork.
- `647bbb9b1` Fix arity of request_ambient_agent_task_id_for_hidden_child test call: rejected as ambient/cloud-agent test surface.
- `90d214af3` Fix markdown rendering on remote SSH sessions: reviewed but not ported in this batch. The retained idea needs a fork-specific implementation because the upstream patch depends on broad `LocalOrRemotePath` and remote markdown editor architecture.
- `98d933cea` Remove managed auto-reload tooltip: rejected as managed autoreload/cloud UI.
- `fe985f6a4` Use standard SVG for Atom icon: not applicable to retained behavior.
- `4aea06734` Run cargo fmt to clean up imports: not applicable as a standalone upstream formatting commit.
- `ac8e80c4d` Fix gh hosts config filename: rejected because the upstream callsite is part of cloud/Oz setup behavior, not retained local terminal GitHub handling.
- `1fb738115` Update telemetry for code review over remote sessions: rejected as telemetry.
- `cf5ebea44` Support local-to-cloud handoff snapshot in remote SSH sessions: rejected as cloud handoff behavior.
- `6acbc0482` Normalize changelog PR metadata from repo sync: rejected as upstream changelog/repo-sync release plumbing.
- `ffe5cff65` Fix redirects going haywire: rejected as upstream cloud/web redirect behavior.
- `08996b560` Update tab CWD and git branch from OSC 7 escape sequences: reviewed but not ported in this batch. OSC 7 is retained terminal functionality, but the upstream patch is a large TerminalView/event refactor and should be reimplemented from the fork's current terminal event path.
- `dbca9ac43` Prevent WASM blocklist local image loads: not applicable because the fork has no Web/WASM host target.
- `194a81d3e` Only refresh tasks on RTC invalidation if a view is open: rejected as ambient/cloud task RTC behavior.
- `81d349656` Session sharing for orchestrated agent sessions: rejected as orchestration/shared-session cloud UI.
- `e1070ff7d` Flip remote code review flag: rejected as upstream rollout flag for the rejected remote code-review architecture.
- `34a7395a1` View attached images in agent: reviewed but not ported in this batch. The retained UX idea must be adapted to ACP attachment state because the upstream patch depends on old AgentView context/block action structure.
- `885c54063` Style rich text editor newlines as line breaks: reviewed but not ported in this batch. The upstream patch conflicts with current editor code and includes deleted remote/drive/telemetry paths.
- `1201c0f8a` Detect common skill updates during script/run: rejected as app-managed skill behavior.
- `cd745fac9` Highlight focused AI block matches under async find: not applicable because the upstream async-find model path is absent from the fork.
- `d5c71b4dc` Update banner UI for exiting shell mode in shell prefix: rejected for this batch because it depends on upstream shell-mode banner structure that diverges from the fork.
- `7f4e11136` Add NLD decision telemetry event: rejected as telemetry. Retained NLD should not regain event reporting.
- `5a8eea31f` Add model metadata to context window telemetry: rejected as telemetry and old model metadata surface.
- `af5b45b6d` Use target-OS path separator for rc file paths: rejected because the fork is macOS host only and the upstream change targets restored cross-platform path branching.
- `dc408d2cb` Move GenericServerObject to warp_server_client: rejected as Warp server-client restructuring.
- `679e426d5` Expose terminal focus URL env vars: reviewed but not ported in this batch. The retained local env-var idea conflicts with current session creation and should be implemented separately without deleted Windows allowlists.
- `a76823c88` Move GenericCloudObject to warp_server_client: rejected as Warp server-client restructuring.
- `b385a14f5` CLI commands for CRUD on agents: rejected as cloud agent management CLI.
- `a1644eed6` Bedrock UI Model Picker Updates: rejected as old model-picker/provider UI.
- `ad6fdc07b` Add observability to oz run setup: rejected as Oz/orchestration telemetry/observability.
- `906433518` Show orchestrator pill as in-progress while children run: rejected as orchestration UI.
- `e9c6fd09c` Add search to Oz API keys: rejected as Oz/API-key UI.
- `46a59cb2b` Daemon startup latency telemetry: rejected as telemetry.
- `9c5c4253f` Fix AI blocks not appearing due to conversation existing in more than one pane: reviewed but not ported in this batch. The retained AgentView issue requires fork-specific ACP conversation ownership analysis and should not import upstream cloud conversation assumptions.
- `a530563eb` Show vertical tab notification dots when inbox is hidden: rejected as inbox/notification surface not retained in this fork.
- `0b737e22a` Banner for remote server disconnect: reviewed but not ported in this batch. The retained remote-disconnect UX should be implemented against the fork's current SSH remote-server banner path.
- `fc110333a` Tab group feature flag and entry points: rejected as upstream feature-flag entrypoint plumbing.

## Verification

Verification run after applying the retained changes:

- `cargo fmt -- --check`
- `git diff --check`
- `cargo check -p warp --all-targets --message-format short`
- `cargo check --workspace --all-targets --message-format short`
- `cargo nextest run -p warp -E 'test(test_reset_pane_sizes_resets_containing_branch) | test(test_reset_pane_sizes_only_resets_containing_branch) | test(test_sqlite_drops_too_small_bounds_on_save) | test(test_sqlite_drops_too_small_bounds_on_read)'`
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`
- `cargo test -p remote_server is_disconnected --lib`
- `cargo test -p remote_server disconnected_on_closed_stream --lib`
- Deleted-surface scans from `AGENTS.md`. No app-side MCP/skills hits were found. A focused cloud-product scan excluding weak-reference `upgrade` terminology and tokenizer vocabulary returned no restored auth, billing, Teams, Warp Drive, GraphQL, Sentry, telemetry, Agent SDK, ambient/cloud-agent, managed-secret, or cloud-environment surfaces. Platform hits were retained bootstrap ConPTY comments and SSH `ForwardX11=no` remote-terminal behavior.
