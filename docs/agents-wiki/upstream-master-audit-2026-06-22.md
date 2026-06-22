# Upstream Master Audit 2026-06-22

Range under review: `b5d8b48b6..8cb48ba94`

Previous audited upstream tip: `b5d8b48b6 Fix /test-warp-ui skill (#12731)`

Current upstream tip detected: `8cb48ba94 Reduce flakiness of windows local_tty test. (#12881)`

Total upstream commits in this incremental range: 10

Status: triage complete. Compatible retained terminal event-dispatch and integration-test render-loop fixes were ported or adapted manually. Upstream remote Agent Mode context snapshot publishing/consumption, app-managed skills/MCP/global-rule synchronization, Cloud Agent continue-locally semantics, Grok/free-AI account gates, cloud tracing, managed secrets, broad TUI framework restructuring, and native Windows local TTY test changes were rejected or marked not applicable under the fork contract.

## Ported Or Adapted

- `19a3bc552` Adapted the headless integration-test render-loop fix by adding a `defer_scene_build` feature through `warpui`/`warpui_core` and enabling it for the app `integration_tests` feature. The invalidation callback now skips eager `build_scene` for integration-test builds that opt into deferred scene building, without importing upstream agent-mode eval feature lists or removed product features.
- `1d1ee8299` Ported the retained terminal `BlockListElement` mouse-down dispatch fix so block-list left-click handling does not run after a child element has already handled the event.

## Rejected Or Not Applicable

| Commit | Decision | Reason |
| --- | --- | --- |
| `002eb1c33` | Reject | Publishes remote Agent Mode context snapshots containing bundled/home skills, MCP metadata, and remote global rules. App-managed skills/MCP remain removed, and ACP agents own their own skills and rules. |
| `46265f499` | Reject | Consumes those remote Agent Mode context snapshots into `SkillManager` and project global rules, restoring app-side skill catalogs and remote skill synchronization. |
| `62f95346e` | Reject | Changes `/fork` vs `/continue-locally` behavior for cloud conversations and Cloud Mode composers. This fork has ACP-only local AgentView semantics, so the local fork button should continue inserting `/fork`. |
| `9e2015d7f` | Reject | Counts SuperGrok/xAI OAuth subscriptions as AI availability and adjusts free-AI account modal behavior. Account/billing/free-AI/Grok subscription gates are removed product surfaces. |
| `1ef980b22` | Reject | Adds a broad TUI API surface, new dependencies, and a large `warpui_core` element hierarchy move with no current retained fork caller. Revisit only if a retained macOS or ACP feature needs this framework layer. |
| `109deb73d` | Reject | Adds tracing spans to Cloud Agent initialization, old Agent SDK attachment downloads, server API calls, and managed secrets. Cloud Agent, telemetry/tracing, managed secrets, GraphQL/API paths remain removed. |
| `f84c6cac6` | Not applicable | Fixes a build break in the rejected TUI `StoredView` restructuring from `1ef980b22`. |
| `8cb48ba94` | Not applicable | Updates a Windows-only local TTY test under native Windows host code, which is outside the macOS-only host scope. |

## Verification

Commands to run after porting:

- `cargo check -p warp --all-targets --message-format short`
- `cargo check --workspace --all-targets --message-format short`
- `cargo fmt -- --check`
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`
- `git diff --check`
- Full deleted-surface scans from `AGENTS.md` for cloud/auth/billing/telemetry, MCP/skills, and local Linux/Windows/Web host patterns.

Results:

- `cargo fmt -- --check` passed.
- `git diff --check` passed.
- `cargo check -p warp --all-targets --message-format short` passed.
- `cargo check --workspace --all-targets --message-format short` passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` ran 150 tests: 150 passed, 2373 skipped.
- The cloud/auth/billing/telemetry scan found only existing weak-handle `upgrade()` false positives, tokenizer vocabulary entries, local docs/comments, and retained local paths. Added-line matches were documentation-only audit rationale.
- The MCP/skills scan had no source hits.
- The local Linux/Windows/Web scan found only retained SSH `ForwardX11` settings and existing ConPTY bootstrap comments.
