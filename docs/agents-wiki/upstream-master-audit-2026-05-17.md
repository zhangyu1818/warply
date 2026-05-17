# Upstream Master Audit 2026-05-17

Range under review: `fa732953d..53da56352`

Previous audited upstream tip: `fa732953d add better logging for replay skipping (#11069)`

Current upstream tip reviewed: `53da56352 Clip inline menu header to prevent split-pane overflow (#10811)`

Total upstream commits in this incremental range: 13

Status: complete. All 13 upstream commits in this incremental range have a port/adapt/reject decision for this ACP-only macOS fork.

## Reviewed Commits

Reviewed commits in chronological order. No upstream commit was cherry-picked directly.

## Ported Or Adapted

- `53da56352` Clip inline menu header to prevent split-pane overflow: ported to retained terminal inline-menu UI by clipping the rendered header container. This keeps narrow split panes from painting trailing header controls past the pane boundary.

## Rejected Or Not Applicable

- `c46846f14` Allow cloud agent view exiting during LRCs: not applicable. The upstream fix targets ambient/cloud agent panes and dummy cloud-mode sessions. The current fork has no ambient/cloud pane stack or dummy cloud-mode session path; local AgentView still intentionally blocks exit while a local long-running command owns the terminal.
- `93ef0aca3` Escape vim insert mode before leaving ampersand handoff: rejected as cloud handoff behavior. The fork retains long-running command control handoff, but the upstream `CloudHandoff` input prefix and `OzHandoff`/`HandoffLocalCloud` flags are deleted.
- `33f3284d2` Custom host picker for orchestration: rejected as orchestration UI plus upstream `specs/**` product/tech docs.
- `1259cbf29` Show cloud agent environment/status in vertical tabs: rejected as ambient/cloud agent vertical-tabs metadata. Retained vertical tabs should use local terminal, ACP AgentView, CLI-agent, SSH, and tab-config state only.
- `131762b08` Roll up orchestration credit usage in the agent-mode footer: rejected as orchestration child-agent and credit-usage UI. It also touches old Agent SDK code and adds upstream specs.
- `8f883075f` Do not auto-open details panel for parent-orchestrated child agents: rejected as orchestration/shared-session cloud viewer behavior plus upstream spec.
- `5ec74a40b` Promote orchestration client flags to stable: rejected as old orchestration feature-flag promotion. The fork must not restore orchestration v2, shared-session viewer pill bar, or `run_agents` tool-call UI gates.
- `ba7735d0d` Attach to existing session when opening a cloud agent conversation: rejected as cloud agent/shared-session continuation behavior. The touched ambient agent, shared session, pending query, and server task-status paths are deleted product surfaces.
- `b29c42426` Enter agent view for third-party harnesses started outside cloud mode: not applicable to the current fork. The upstream patch depends on shared-session viewer and ambient-cloud harness resolution (`AmbientAgentViewModel`, `ViewerHarnessResolved`, `shared_session` handlers), which are deleted. Retained local CLI-agent terminal integrations remain under `CLIAgentSessionsModel` and should not restore cloud shared-session viewer plumbing.
- `fb5ad384a` Async blocklist search: rejected for now despite touching retained terminal find paths. The upstream implementation is explicitly unfinished, hidden behind a disabled `AsyncFind` feature flag, adds upstream specs, and lists known AI-block highlight, match-preservation, flicker, and ordering issues. Revisit only if upstream stabilizes the feature as direct retained terminal behavior without disabled experimental gates or specs.
- `6dfbb28d9` Auth secret "New" sidecar hover dismissal: rejected as cloud-mode-v2 auth secret picker UI. The fork removed cloud-mode auth secret management and Warp-managed secret surfaces.
- `88c04f387` Disable model selector for cloud-to-cloud follow-ups: rejected as cloud handoff/model selector behavior. ACP model routing belongs to ACP adapter configuration, and old Warp cloud follow-up selectors are removed.

## Verification

- `cargo fmt --all -- --check`
- `git diff --check`
- `cargo check -p warp --all-targets --message-format short`
- `rg -n "llms?\\.txt|llm\\.text" AGENTS.md docs .agents` returned only the expected documentation rules that say not to recreate `llms.txt`/`llms-full.txt`.
- `git diff -U0` deleted-surface scan on the changed code and wiki files returned no restored auth/cloud/MCP/skills/platform/spec paths in the code diff.

Only the inline-menu header clipping fix was ported from this range; no upstream specs or cloud/orchestration code were merged.
