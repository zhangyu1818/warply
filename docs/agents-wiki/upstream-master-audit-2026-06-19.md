# Upstream Master Audit 2026-06-19

Range under review: `d4bb3f5b7..b5d8b48b6`

Previous audited upstream tip: `d4bb3f5b7 Improve error message when LlmModelHost cannot be parsed (#12666)`

Current upstream tip detected: `b5d8b48b6 Fix /test-warp-ui skill (#12731)`

Total upstream commits in this incremental range: 63

Status: triage complete. Compatible retained editor, file-tree, local code-editor, terminal, macOS windowing, markdown-pane, settings subscriber, PowerShell bootstrap, and dependency-signature changes were ported or adapted manually. Upstream cloud/Oz/orchestration, account/auth/billing/free-AI, SuperGrok, GraphQL, Sentry, app-managed MCP/skills, warpctrl/local-control, tab-grouping/pinning/cross-window-dragging, cloud shared-session, native platform, and upstream spec/process changes were rejected or deferred under the fork contract.

## Ported Or Adapted

- `e653dc2a0` Ported natural numeric-aware sorting for retained local file-tree entries by adding `alphanumeric-sort` and preserving the existing dotfile-before-non-dotfile grouping.
- `4aeeebfc8` Ported the macOS Objective-C native-window chrome event preservation fix for retained macOS windowing.
- `4c5c3abfa` Ported code-review git dialog button tooltips as local UI polish.
- `378f3f1b4` Adapted the terminal focus fix so block completion only releases input focus when blocks are still selected, without restoring upstream block-latency telemetry.
- `bf14cbec0` Ported non-ASCII find/replace matching by reversing encoded UTF-8 bytes for reverse DFA matching and adding CJK/accent regression coverage.
- `1aa6811fb` Adapted the remote/local file-save refresh fix by emitting saved content versions through `GlobalBufferModelEvent::FileSaved`, refreshing `LocalCodeEditorView` baselines, and notifying code-review views without restoring upstream server-local buffer, bundled-skill, or remote skill paths.
- `ec27d06d7` Ported the retained macOS version-check logic.
- `1d99e3971` Ported the `working_directory` settings subscriber update to the current `emitter_handle` subscription shape.
- `eab3b3fa9` Ported deferred PowerShell function-name loading by limiting bootstrap-time function queries to core PowerShell modules and loading the rest asynchronously through the retained shell command executor.
- `52f80716c` Ported the `warp-command-signatures` bump for eza completions while rejecting unrelated upstream `winit` and `x11rb` dependency additions.
- `6c9604f15` Adapted markdown file-pane reopen behavior to the fork's current `FileNotebookView::local_path()` API so reopening an already-visible markdown preview focuses that pane instead of opening a duplicate. The deleted upstream workspace test file was not restored.
- `a6f4671f9` Ported the tree-sitter parse size guard for retained editor syntax parsing, including the `BufferSnapshot::byte_len()` helper and updated indent-query tests.

## Rejected, Deferred, Or Not Applicable

| Commit | Decision | Reason |
| --- | --- | --- |
| `466d1856c` | Reject | Child-agent chip cleanup depends on upstream child-agent/orchestration failure surfaces, not the ACP-only retained backend. |
| `b548948f6` | Reject | Refreshes AWS Bedrock OIDC cloud credentials, a removed Warp cloud credential path. |
| `50cb3c9bc` | Reject | Platform-plugin failure handling belongs to upstream Oz/platform plugin flows removed from this fork. |
| `c54de43cb` | Defer | LRC stop/takeover is retained-adjacent, but the upstream patch is tied to old Agent feature flags and needs a focused ACP-specific port. |
| `e09838dfa` | Reject | Adds auto-handoff sleep modal and cloud handoff UX. |
| `ecbd86288` | Reject | Resumes/retries cloud runs on transport failures, a Warp-hosted agent behavior. |
| `53f273e92` | Reject | Continues upstream tab-group pinning invariants, a product line not accepted in this fork. |
| `2a251933c` | Reject | Persists tab pin state for the rejected tab-grouping/pinning feature line. |
| `1cdb4794e` | Reject | Enforces tab-group contiguity for the rejected tab-grouping/pinning feature line. |
| `5a9ea0ffb` | Not applicable | Adds transient-network debug detail in upstream server/network error paths that are not retained as Warp cloud API behavior. |
| `8794f7325` | Reject | Fixes tab-group behavior for special tabs; tab grouping remains rejected. |
| `d21855ab0` | Reject | Adds bundled-skill host behavior for Agent Mode. App-bundled skills remain removed. |
| `55b411ec6` | Reject | Adds SuperGrok OAuth paste-code account flow. |
| `81d02ea43` | Reject | Adds `warp://settings/warp_agent` deep link for upstream Warp Agent settings. |
| `c29cf0fde` | Not applicable | Queued-prompt inline panel file is absent in this fork; applying it would restore deleted queue-panel UI. |
| `36af5410e` | Reject | Adds Continue Locally behavior for cloud Oz runs. |
| `9bd47fd29` | Defer | Remote global search depends on broad upstream remote ripgrep/protocol architecture and prior rejected remote-search work. |
| `23b686302` | Reject | Handles 4xx MAA errors in Oz/cloud-agent paths. |
| `eaec8b95e` | Reject | Adds GitLab env repo handling for Oz/cloud environments. |
| `c4833c440` | Reject | Fixes Sentry-linked standalone CLI RPATH; Sentry/crash reporting remains removed. |
| `29d7d9f3f` | Not applicable | Deletes a free-AI experiment already removed with account/billing surfaces. |
| `9eb570cd5` | Defer | Rewind slash-command prefill is retained-adjacent but upstream patch crosses old shared-session and command model assumptions; needs separate ACP/static-command review. |
| `0d5e8bbaf` | Reject | Adds local orchestration subagent iconography tied to old orchestration surfaces. |
| `2dbf25fe4` | Defer | macOS Dock-icon hiding is retained-adjacent, but upstream commit includes specs and broad platform settings changes; any port should be specs-free and focused. |
| `07fb1b214` | Reject | Adjusts server ping frames in cloud shared-session networking. |
| `81d06dae4` | Not applicable | Show-hidden-files setting path is absent and would restore deleted settings `code_page` structure. |
| `ced1ede48` | Defer | PTY spawn error specificity is useful terminal work, but the upstream commit is entangled with deleted old `agent_sdk`; local TTY-only adaptation should be separate. |
| `1acbff119` | Defer | Backend-neutral `warpui_core` hierarchy changes are broad framework/TUI groundwork with no immediate retained fork call site. |
| `79fdd7ceb` | Reject | Cross-window dragging fixes depend on rejected tab grouping/splitting behavior. |
| `7070e4d4f` | Defer | Cmd-K cancellation touches retained-adjacent long-running command behavior but relies on upstream queue/LRC assumptions. |
| `89ec9a397` | Defer | Lost-connection banner suppression touches old blocklist/agent stream rendering; needs ACP-specific validation before porting. |
| `8de0888ae` | Reject | Adds horizontal tab pinning UI for the rejected pinning line. |
| `a9ecba83f` | Reject | Adds `glab` config for cloud sandboxes. |
| `2d8587373` | Defer | LRC auto-queue setting brings upstream specs, telemetry, and queued-prompt UI assumptions absent from the fork. |
| `106176f4d` | Not applicable | Updates CI install action only; no retained runtime or fork contract change. |
| `3ede77814` | Reject | Onboarding/free-AI notice logic belongs to removed account/onboarding/billing surfaces. |
| `9c9412316` | Reject | Resolves managed MCPs in agent-run CLI; app-managed MCP remains removed. |
| `b101722e9` | Defer | Cross-window tab dragging is broad retained-adjacent UI/framework work and includes Windows/native-platform paths and deleted workspace tests. |
| `474cf6b01` | Reject | Enables cross-window tab dragging rollout for Windows/macOS feature flags. |
| `1d65e362b` | Reject | Adds `warpctrl` wrappers and PATH install behavior. |
| `63b3ca251` | Reject | Adds `launch_mode` to internal spans, restoring telemetry/span plumbing. |
| `293754ae7` | Reject | Reduces task status updates in old Agent SDK/local task sync/server API paths. |
| `19c2ca135` | Not applicable | Adds `@babel/core` to deleted GraphQL schema tooling. |
| `4f133a634` | Reject | Adds free-AI removal notice/modal account/billing UX. |
| `3ffa815d5` | Defer | Broad `emitter_handle` subscription API migration touches deleted cloud, MCP, skills, Agent SDK, warpctrl, and platform paths; only narrow retained follow-ups were ported. |
| `81fcb5d38` | Reject | Suppresses warnings on cloud shared-session ping frames. |
| `3ed3ae1d0` | Defer | Git branch status context chip is retained-adjacent, but upstream patch includes remote proto, broad chip UI rewrites, and remote-server hydration assumptions. |
| `939988876` | Not applicable | Updates a one-time-modal subscriber in account/free-AI modal code that remains deleted/rejected. |
| `ebb792bdb` | Defer | AsyncFind rollout flag change should be reviewed against the fork's feature-flag cleanup separately. |
| `e31d8cd90` | Reject | Shares remote context file hydration through skills/file-watchers; app-managed skills remain removed. |
| `b5d8b48b6` | Reject | Fixes `/test-warp-ui` app-managed skill behavior. |

## Verification

Commands to run after porting:

- `cargo check -p warp --all-targets --message-format short`
- `cargo check --workspace --all-targets --message-format short`
- `cargo fmt -- --check`
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`
- `git diff --check`
- Full deleted-surface scans from `AGENTS.md` for cloud/auth/billing/telemetry, MCP/skills, and local Linux/Windows/Web host patterns.

Results:

- Both cargo check commands passed.
- `cargo fmt -- --check` passed after formatting two migrated call sites.
- `cargo nextest` ran 150 tests: 150 passed, 2373 skipped.
- `git diff --check` passed.
- The cloud/auth/billing/telemetry scan found only existing weak-handle `upgrade()` false positives, tokenizer vocabulary entries, local docs/comments, and retained local paths. Added-line matches were documentation-only audit rationale.
- The MCP/skills scan had no source hits; added-line matches were documentation-only audit rationale.
- The local Linux/Windows/Web scan found only retained SSH `ForwardX11` and existing ConPTY bootstrap comments. Added-line matches were documentation-only audit rationale for rejecting `x11rb`.
