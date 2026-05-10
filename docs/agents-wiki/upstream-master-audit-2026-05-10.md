# Upstream Master Audit 2026-05-10

Range audited: `master..upstream/master`

Total upstream commits reviewed: 110

This audit records the decision for every commit in the upstream range. A commit marked `ported` has been cherry-picked or manually adapted onto this fork's `main`. A commit marked `rejected` must not be merged directly because it restores removed product areas or conflicts with the local ACP-only fork contract. A commit marked `rewrite-for-acp` is not suitable as-is; implement the behavior only through the current ACP model if it becomes necessary.

## Decisions

1. `d2f26ae9` Rename Active Pane action: ported. Generic command/action rename.
2. `fc1157e0` macOS IME key-equivalent fix: ported. Platform input fix.
3. `a8ec49e4` README badge formatting: rejected. Cosmetic upstream README churn, no fork behavior value.
4. `74672609` Requested commands auto-cancel race: ported. Adapted to current ACP AgentView history model without restoring cloud behavior.
5. `3c43a6af` Orchestration hover card leave pill: rejected. Old orchestration UI.
6. `8b57ae4a` Oz for OSS README section: rejected. Oz/cloud-agent documentation.
7. `0b3311fa` Add vim to STAKEHOLDERS: rejected. Upstream ownership metadata only.
8. `0320c792` CycleMostRecentTab option: ported. Local tab navigation feature; cloud/shared-session hunks were removed.
9. `560efc3c` AgentConversationEntry migration: rejected. Old agent management model migration.
10. `782058ab` Promote DirectoryTabColors stable: ported. Local appearance feature.
11. `5ae13bd8` 429 run-cloud CLI polling: rejected. Cloud run polling.
12. `f1ae6f35` Send context with `/compact`: rewrite-for-acp. Old agent API and redaction path; only reimplement if ACP compacting needs it.
13. `9d9972cb` Diesel security update: ported. Dependency security fix, lockfile adapted without reintroducing removed auth deps.
14. `5146a5bf` Active rule files race: ported. Local project context and AGENTS/WARP rules fix.
15. `1175e82f` Git branch/diff-stats chip race: ported. Local context chip fix.
16. `9e76f633` Hide notification chip in ambient sessions: rejected. Ambient/cloud session UI.
17. `9c162bca` SCP install fallback without curl/wget: rejected. Reintroduces remote auto-install download path from Warp servers; current fork intentionally disables that install script.
18. `b00d068e` Simplify DiffStateModel to one repo: rewrite-for-acp/local. Direct port reintroduces telemetry/remote-oriented code review structure.
19. `e109bf0c` Secret precedence in Oz CLI: rejected. Oz CLI and managed secret path.
20. `94f63ce2` Clean up ConversationOrTask: rejected. Old cloud agent/task management model.
21. `37274bfe` Revised vertical tab summary mode: rewrite-for-acp. Direct port bakes old Agent/CLI-agent status semantics into vertical tabs.
22. `a639d761` Terminal view rendering deadlock: rejected. Fix is centered on cloud/ambient agent pre-first-exchange and shared-session paths.
23. `bc81fdc4` Child agent notifications: rejected. Orchestration child-agent management.
24. `1f72e823` Do not auto-open details after cloud agent: rejected. Cloud agent details behavior.
25. `c28fdddb` RowIterator crash: ported. Terminal grid crash fix.
26. `0b728175` Contributing and PR template updates: rejected. Upstream process docs.
27. `04069a29` README badge brand assets: rejected. Cosmetic upstream README churn.
28. `e3514ff5` Orchestration pill avatar font sizes: rejected. Orchestration UI.
29. `b08e5782` Accept without orchestration split button: rejected. Orchestration UI.
30. `2e5272dd` Remote env codebase indexing handshake: rejected. Codebase indexing and remote cloud environment handshake.
31. `1244ffbe` Debian duplicate apt source: ported. Packaging fix.
32. `689cbce0` Waiting on initial repo sync: rejected. Mixes agent SDK, Drive sync, server sync, and repo metadata.
33. `916ca128` Cross-platform option in issue template: rejected. GitHub template metadata.
34. `aea652ad` CLI footer exit AgentView behavior: ported. Local AgentView input behavior.
35. `28c9c7d0` Mark focused-pane notifications read on refocus: ported. Local window focus behavior.
36. `543d54ec` Linux/Wayland IME: ported. Platform input fix.
37. `4dbf8758` Tree output filenames as links: ported. Terminal link detection improvement.
38. `55ca9786` Platform credits counting client: rejected. Usage credits/billing accounting.
39. `f8b93fa2` Harness icon background in conversation panel: rejected. Old third-party harness/cloud UI.
40. `1edc9cd8` Stop re-firing `gh pr view`: ported. Local code review fix.
41. `0ba2ad39` Driver credential writes/refresh: rejected. Agent SDK credentials.
42. `35e3a6f5` Do not check server versions on wasm/onboarding: rejected. Autoupdate/onboarding surface.
43. `09a35b58` Decouple DiffStateModel and CodeReviewView: rewrite-for-acp/local. Depends on remote/cloud code review split after rejected diff-state work.
44. `b5c64ff4` Rename tests to `_tests.rs`: rejected. Broad mechanical churn across deleted/changed modules.
45. `7ec0ec37` Promote ConfigurableContextWindow stable: ported. Local execution profile setting; CloudMode dogfood hunk removed.
46. `92cb3d15` Restore cloud conversation transcripts: rejected. Cloud transcript loading.
47. `606e1653` Markdown ToC anchors notebooks: rejected. Notebook product path.
48. `131e9e8b` Disable CLI auto-indexing outside agent run: rejected. Codebase indexing path.
49. `afc8b55d` Orchestration pill bar replace_pane: rejected. Orchestration UI.
50. `eb0b51f3` Taskkill exit code in inno logs: rejected. Autoupdate and telemetry event path.
51. `564c70ee` Async command palette file loading: ported. Local command palette performance.
52. `50003a85` Infinite SSE retry stale agent runs: rejected. Cloud agent event streamer.
53. `3019671e` Re-index project rules on startup: ported. Local project rules context.
54. `a7dbccab` Spawned agents status card indentation: rejected. Orchestration child-agent UI.
55. `0510ea89` Write-tech-spec skill update: rejected. Upstream skill/process doc.
56. `38ead212` Remote backed global buffer: rejected. Remote code/global buffer feature, not part of current retained SSH terminal contract.
57. `1fa2fc30` Docs bullet style: rejected. Upstream docs/process churn.
58. `ec1788fa` Exclude child agent initial prompt from completions: rejected. Child/orchestrator agent history.
59. `8fb2e397` Orchestration confirmation UI bugs: rejected. Orchestration UI.
60. `8a005e5b` Gate git credential refresh: rejected. Agent SDK credential path.
61. `844dc2ce` Remote-server bash 3.2 tilde install script: rejected. Current fork disables remote auto-install script.
62. `9eaa55f7` Remove block attachment in locking attachment: ported. Natural-language detection/input locking behavior.
63. `80b5d1a0` Tombstone regression: rejected. Shared-session behavior.
64. `e7736435` Change-keybinding skill: rejected. Bundled skill/process feature not required by fork contract.
65. `be225e93` Scope upward menu positioning confirmation card: rejected. Orchestration confirmation UI.
66. `9c1df06d` Third-party harness model selection: rejected. Cloud/third-party harness selection.
67. `59e802ea` Linked-worktree branch checkout: ported. Local git context chip fix.
68. `c3df6eb6` Cloud mode for third-party harnesses: rejected. Cloud mode.
69. `36db2396` CloudModeV2 slash sidecar alignment: rejected. Cloud mode UI.
70. `48a648b1` DiffStateModel local/remote wrapper: rejected. Remote/cloud code review split.
71. `10e2bae9` Reveal in Finder: ported. Local code pane menu action; markdown rendering hunk omitted because fork removed that path.
72. `898336e3` MCP servers for third-party harnesses: rejected. Agent SDK harness support.
73. `044d6ebb` Oz OIDC stable feature: rejected. Oz identity federation.
74. `434b50db` CLI agent permission-scoped state: rejected. CLI-agent plugin permission flow, not ACP model.
75. `9d2296d1` Agent CLI flag for cloud runs: rejected. Cloud runs.
76. `84f9584c` In-app auth secret flow: rejected. Auth, managed secrets, cloud settings.
77. `be5d0cfc` HTTP/TLS dependency dedupe for AWS SDK: rejected. The relevant AWS BYO/cloud credential deps are not a retained fork target.
78. `c2565f50` Pacman signing key validation: rejected. Autoupdate.
79. `e75bf809` Orchestration message transcript UI: rejected. Orchestration UI.
80. `f85d69aa` Choosing model with Codex: rejected. Old agent SDK harness model selection; ACP config options are the source of truth in this fork.
81. `7f5a6893` Named agent API key support: rejected. Cloud auth/GraphQL/API key path.
82. `68f6062d` CJK punctuation link detection: ported. Terminal link detection fix.
83. `87dae23` Pin Ubuntu LTS internal dev image: rejected. Internal dev image only.
84. `65418859` Slash command enablement on execution: rejected direct port. It couples execution gating to Cloud Mode v2/ambient Oz datasource; rewrite only against current ACP datasource if needed.
85. `756586ff` Local-to-cloud handoff entrypoint: rejected. Cloud handoff.
86. `42af6f6c` Context field to third-party harnesses: rejected. Agent SDK harness support.
87. `148e80ce` Harness-support report-shutdown: rejected. Telemetry/server harness support.
88. `99e8a65c` Typed remote server SSH install errors: ported. Telemetry calls and unported SCP fallback were removed.
89. `a806bfb2` Hermes CLI agent detection/config: rejected. Hermes/CLI agent integration and telemetry.
90. `e0e2d040` ServerApi optional token refresh: rejected. Auth token refresh/server API.
91. `8a4df58a` Hide chips in handoff input mode: rejected. Cloud handoff input.
92. `09f7b965` Pin generate-changelog action: rejected. GitHub workflow metadata.
93. `90ee3a58` Remote code integration tests: rejected. Remote code/integration test infrastructure outside current fork target.
94. `0a0e9de3` Restore ambient agent conversations into cloud mode panes: rejected. Ambient/cloud agent restore.
95. `bd66e79f` Run-agents picker layout: rejected. Orchestration UI.
96. `ac091058` OpenSSL bump: ported. Dependency security fix.
97. `d426c045` Windows typed chars non-IME: ported. Platform input fix.
98. `ef00af00` Cloud background/status styling: rejected. Cloud agent/background UI.
99. `b87da5fe` Repo picker readability: ported. Local tab config UI.
100. `22c9472d` Forward CLI agent env vars into WSL: rejected. CLI-agent env forwarding, not ACP path.
101. `59edd297` Hide HandoffRehydration system query: rejected. Cloud handoff conversion.
102. `02303121` SQLite remote codebase indexing cache: rejected. Codebase indexing.
103. `8b72d322` Code review find bar cleanup: ported. Local code review UI cleanup.
104. `1711d597` Remote file tree incremental updates: ported. Local/remote retained file tree update path.
105. `2cc7a9f1` Hide command for third-party harness cloud agents: rejected. Third-party cloud harness.
106. `4a2678d1` Harness-specific model selection in orchestration config: rejected. Orchestration/harness config.
107. `b84e3e98` Common skills installer: rejected. Upstream skill installer/process infrastructure, not fork runtime.
108. `4d84af55` Unified agent icon status in management panel: rejected. Old agent management panel.
109. `c8d39088` Mermaid render failure callout: rejected. Notebook/editor markdown product path.
110. `35cb40c3` Raw/rendered Mermaid notebook toggle: rejected. Notebook product path.

## Notes For Future Upstream Pulls

- Do not merge upstream commits only because they touch retained files. Several rejected commits touched retained files but were still cloud, telemetry, old agent, notebook, autoupdate, or codebase-indexing changes.
- When a rejected commit contains a useful local idea, reimplement it against this fork's current primitives instead of reviving the upstream dependency chain.
- For ACP-related behavior, the source of truth is the ACP event/config model, not old Warp Agent SDK, Cloud Mode, Oz, or CLI harness code.
