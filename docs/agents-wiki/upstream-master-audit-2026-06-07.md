# Upstream Master Audit 2026-06-07

Range under review: `3497d1844..d3757291a`

Previous audited upstream tip: `3497d1844 Stop watching gitignored directories in the repo file watcher (#12122)`

Current upstream tip detected: `d3757291a Add basic tab group rendering for horizontal tabs (#12089)`

Total upstream commits in this incremental range: 37

Status: triage complete. Compatible retained fixes were ported manually. Deleted cloud, app-side MCP/skills, orchestration, local-control, process-doc, and tab-grouping surfaces were rejected or deferred according to the fork contract.

## Ported

- `e70eef10c` Downgrade headless delegate event-loop send failures from `warn` to `debug`. This is retained generic `warpui` test/headless behavior.
- `370b671f2` Report DECSET 1004 focus events in normal-screen terminal apps by checking the active terminal mode instead of alt-screen state. This is retained terminal emulator behavior.
- `2789b0b54` Remove the extra `focus_ai_block_if_self_focused` call after an AI exchange finishes. The retained ACP AgentView uses the same focus path and should not force an async accidental focus steal.

## Fork-Specific Adaptation

- SSH Warpify was updated after comparing the fork against current upstream Warpify handling. Upstream still routes `ReadyToWarpify` through a prompt/footer path when the footer feature is enabled; the ACP-only fork had already removed the older prompt branch and kept only the footer button. That made SSH Warpify require a manual click before the retained `add_ssh_warpifying_block` path wrote the remote shell hook. The fork now automatically starts the existing SSH Warpify block when `evaluate_warpify_ssh_host` returns `ShouldPromptWarpification`, while preserving the existing settings, tmux, denylist, and pending-host checks.

## Reviewed But Not Ported

| Commit | Decision | Reason |
| --- | --- | --- |
| `c429bb8a6` | Reject | App-side skills and repo metadata standing results restore a deleted skills ownership model. |
| `6c4125ce1` | Reject | Cloud/orchestration pane-group refactor for removed agent surfaces. |
| `633a8e39d` | Reject | Inline model selector behavior belongs to the removed Warp model selector flow. |
| `79de42b51` | Not applicable | The remote git-ops guard depends on upstream `LocalOrRemotePath` code-review architecture that this fork did not port. Do not guess remote status from plain `PathBuf`. |
| `09cd44513` | Reject | Agent event stream/server upload logging belongs to deleted cloud agent APIs. |
| `69bb47708` | Reject | Orchestration rollout cleanup touches deleted Agent SDK/orchestration surfaces. |
| `38e45a756` | Defer | "Jump to latest agent message" is a possible retained AgentView UX idea but the patch includes telemetry and upstream agent-origin plumbing; adapt separately if desired. |
| `48252f501` | Reject | Agent SDK harness timeline event for removed third-party harnesses. |
| `b3bde3566` | Reject | Oz/shared-session client retry logic for deleted cloud sharing surfaces. |
| `1f1826298` | Reject | Remote project skills are app-managed skills and remain deleted. |
| `b624e2299` | Reject | App-side MCP runtime migration is outside Warp ownership in this fork. |
| `51c380ce9` | Reject | MCP settings UI is deleted; ACP agents own MCP configuration. |
| `a856c95d2` | Defer | Repo metadata standing-query cleanup is entangled with deleted skills/rules infrastructure. |
| `43828a6d6` | Defer | Repo metadata file-tree performance change is retained-adjacent but broad; port in a focused repo metadata pass. |
| `3fe061620` | Defer | File-search truncation fix spans repo metadata local/remote models; port separately with focused tests. |
| `a8df31722` | Defer | Block navigation fix adds settings/UI surface; retainable, but outside this SSH Warpify release. |
| `099855bfe` | Not applicable | The queued prompt panel file is absent/restructured in this fork. |
| `e8024b5a1` | Defer | Lazy repo metadata force-include behavior is retained-adjacent but tied to the standing-query stack. |
| `426427583` | Reject | Linux watcher behavior is native Linux support, not retained macOS-to-remote SSH behavior. |
| `03ad9ea9a` | Defer | Lazy repo metadata subtree behavior is retained-adjacent; port only with repo metadata test coverage. |
| `eaa936c78` | Defer | Rich Input Ctrl+Enter setting is retainable UI behavior but broad and not required for this release. |
| `8e1b7e9a5` | Reject | Remote project rules through standing results depend on upstream remote rules/skills ownership. |
| `e4695f219` | Defer | Project Explorer hidden-file toggle is a new retained UI feature; not mixed into this Warpify release. |
| `327445ee8` | Defer | User-added workspace indexing spans persisted workspace and repo metadata; requires a separate local-data review. |
| `2700e7dde` | Reject | Upstream stale-PR automation and specs are process docs, not fork product code. |
| `dd274743a` | Reject | Upstream contribution-label documentation is not fork memory. |
| `2bb3a04b4` | Defer | Tink dependency pin is unrelated to the SSH/Warpify fix and should be updated only with dependency-specific verification. |
| `21a8ae477` | Reject | Orchestration message display setting belongs to deleted orchestration UI. |
| `457ffadd3` | Reject | Orchestration feature-flag cleanup touches deleted orchestration surfaces. |
| `e4ab1917b` | Reject | Local-skill filtering still depends on deleted app-managed skills. |
| `c2e974b93` | Defer | Remote-server initialization hook is retained-adjacent but bundled with codebase indexing restoration; port only after isolating remote-server behavior. |
| `1c2d4ccb3` | Reject | Large-orchestrator event streaming belongs to deleted cloud orchestration. |
| `5967abf0b` | Reject | Warp Control CLI v2 adds new local-control product, settings, specs, Linux packaging, and CLI surfaces outside this fork contract. |
| `d3757291a` | Reject | Tab-grouping rendering continues the previously rejected tab-group feature line. |

## Verification

Commands run after porting:

- `cargo fmt`
- `cargo fmt -- --check`
- `cargo nextest run -p warp -E 'test(ready_ssh_login_auto_warpifies_when_enabled) | test(ready_ssh_login_respects_disabled_ssh_warpification) | test(focus_reporting_writes_focus_events_in_normal_screen)'`
- `cargo check -p warp --all-targets --message-format short`
- `cargo check --workspace --all-targets --message-format short`
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`
- Deleted-surface scans from `AGENTS.md`, plus added-line scans for the same patterns. Full scans produced only existing allowed false positives such as weak-handle `upgrade()` names, tokenizer vocabulary, retained SSH `ForwardX11`, and bootstrap comments; added-line scans were empty.
