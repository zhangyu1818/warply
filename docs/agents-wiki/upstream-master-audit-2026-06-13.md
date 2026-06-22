# Upstream Master Audit 2026-06-13

Range under review: `a30cc7a33..c0c6cead9`

Previous audited upstream tip: `a30cc7a33 Enhance writing logic for ai_queries sqlite db table (#12484)`

Current upstream tip detected: `c0c6cead9 Authenticate cloud-agent OTLP trace export (#12364)`

Total upstream commits in this incremental range: 17

Status: triage complete. Compatible retained code-review, local UI, code editor, settings, markdown rendering, and local path-performance fixes were ported or adapted manually. Deleted cloud agent, GraphQL/workspace federation, app-side skills, bundled remote skills, upstream tab grouping, SSH Warpify removal, native WSL framing, stale upstream GitHub automation, and upstream issue-template maintenance were rejected or marked not applicable according to the fork contract.

## Ported Or Adapted

- `c3bbb4f6c` Adapted the code-review attach-as-context routing fix to this fork's local `PathBuf` model. `CodeReviewView` no longer captures a construction-time terminal weak handle; the right panel resolves the attach target at action time from the active pane group and selected repo while preserving ACP-only AgentView behavior.
- `d7ecfac54` Ported the markdown header spacing fix in the retained editor renderer by removing the inner uniform header margin while preserving the fork's explicit heading edges.
- `3f83932cd` Adapted format-on-save to the fork's retained local code editor and current settings UI. The setting is local `CodeSettings` state exposed from the Features page and gates language-server formatting before save.
- `526d39cbb` Ported the theme picker traffic-light padding fix and added a focused workspace test so opening the theme chooser no longer suppresses macOS traffic-light left padding.
- `475fdb33e` Adapted appearance-change text color refresh to retained OpenAI-compatible terminal suggestion editors in AI settings. Upstream BYOK/custom-inference editor paths are absent from the fork.
- `aa873b543` Adapted the retained local portions of the canonical-path performance fix. Terminal views cache local canonical PWDs, directory-color lookups use canonical keys at write time, and code/vertical-tab color paths avoid repeated per-render canonicalization. WSL-specific title behavior was not ported.
- `24b585eb6` Ported the DisplayChip menu render-performance fix by sharing filtered menu items through `Rc<Vec<_>>` so render closures do not clone the full menu item list on every frame.

## Rejected Or Not Applicable

| Commit | Decision | Reason |
| --- | --- | --- |
| `498576859` | Reject | Conversation-list rename depends on upstream `ServerApiProvider::rename_conversation`, server conversation tokens, and cloud sync messages. Local ACP conversation entries should not regain Warp server rename semantics. |
| `ec3006c83` | Reject | Gemini Enterprise federation config touches GraphQL schema and workspace federation/product areas removed from the fork. |
| `5a35550d3` | Not applicable / reject | The queued-prompts panel, telemetry, and upstream spec files targeted by Enter-to-send-now are absent or removed in this fork. Do not recreate upstream queued-prompt product scaffolding from this commit. |
| `57062bd92` | Reject | The commit removes tmux-based SSH Warpification. This fork explicitly retains SSH, Warpify, and tmux checks as terminal functionality. |
| `daa158267` | Reject | Fixes `/open-skill`/Open skill button behavior for app-managed skills, which are removed and owned externally by ACP agents. |
| `e721efe64` | Not applicable | The stale requested-changes PR script is not present in this fork; do not recreate upstream GitHub automation scripts solely for this fix. |
| `e9c592274` | Not applicable | Upstream issue-template link maintenance targets upstream process files that are not fork runtime behavior. |
| `af532bdc3` | Reject | Continues the upstream tab-grouping/pinning line that this fork has not accepted. Current tab/vertical-tab behavior remains local and non-grouped. |
| `d775c9226` | Reject | Installs bundled skills globally on remote hosts. App-bundled skills and app-side skill installation remain removed; skills belong to ACP agents. |
| `c0c6cead9` | Reject | Authenticates cloud-agent OTLP trace export and tracing dependency paths for deleted cloud-agent/telemetry surfaces. |

## Verification

Commands run after porting:

- `cargo fmt`
- `cargo fmt -- --check`
- `cargo check -p warp --all-targets --message-format short`
- `cargo check --workspace --all-targets --message-format short`
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`
- `git diff --check`
- Full deleted-surface scans from `AGENTS.md` for cloud/auth/billing/telemetry, MCP/skills, and local Linux/Windows/Web host patterns. Hits were existing weak-handle `upgrade()` false positives, tokenizer vocabulary, retained SSH `ForwardX11`, and retained bootstrap `ConPTY` comments.
- Added-line deleted-surface scan across the current diff. The only matches were ordinary `upgrade()` weak-handle calls in the retained code-review provider path.
