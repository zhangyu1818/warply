# Upstream Master Audit 2026-05-16

Range under review: `1ca5496d8..fa732953d`

Previous audited upstream tip: `1ca5496d8 docs: clarify bug readiness wording (#10866)`

Current upstream tip fetched: `fa732953d add better logging for replay skipping (#11069)`

Total upstream commits in this incremental range: 84

Status: complete. All 84 upstream commits in this incremental range have a port/adapt/reject decision for this ACP-only macOS fork.

## Batch 1 Reviewed

Reviewed commits 1 through 24 in chronological order. Compatible changes were manually adapted; no upstream commit was cherry-picked directly.

## Ported Or Adapted

- `5afbb0ce8` Fix New worktree config clipping in tab config menu: ported by removing the vertical-tabs width branch and always using the 268px new-session menu width.
- `5c90c9cc3` Fix link display text escaping for notebooks/plans: ported to the retained markdown parser/editor. Autolinks now unescape markdown backslash escapes before storing URL text, with parser and editor round-trip tests.
- `424e33358` Fall back to lazy index when repo exceeds max file limit: ported to retained local repo metadata. Repos that exceed the full index file limit retry as first-level lazy trees. Upstream telemetry was not ported.

## Rejected Or Not Applicable

- `0f4577a0b` Billing page autoreload credits: rejected as billing/product cloud surface.
- `129783073` Custom inference endpoints for third-party API models: rejected because it targets the old Warp Agent SDK/server model endpoint flow. This does not replace the retained OpenAI-compatible terminal suggestions provider.
- `9eef1d25c` Revert back to NLD V1 on dogfood: not applicable. The fork uses the retained ONNX/heuristic NLD path and removed dogfood rollout/platform bundle branches.
- `b4752084d` Oz changelog skill release workflow: rejected as Oz/skill release automation.
- `fbfca3bb5` script/run common-skills reinstall check: rejected as app-bundled/common skill infrastructure.
- `7225e824b` Cloud Agent tab menu rename: rejected as cloud agent UI.
- `98aaece54` Auth secret dropdown picker for non-Oz orchestration: rejected as managed secret/orchestration surface.
- `b9ec4f39f` Cloud mode tombstone UX: rejected as cloud mode conversation UI.
- `0b0318a32` HTTP POST form upload targets: rejected as old server API/upload target plumbing.
- `e12c35e5c` Send custom endpoints to server: rejected as server model endpoint plumbing.
- `f61826505` Theme picker stale colors during OS sync toggle: rejected because the changed code is in removed onboarding/theme-picker flow.
- `ed455d902` Local to cloud handoff while requests are in progress: rejected as cloud handoff.
- `798c8c471` Local to cloud handoff hints: rejected as cloud handoff.
- `3c4b76c00` Migrate code view to use FileLocation: not applicable to the current fork. The upstream patch depends on `app/src/code/buffer_location.rs` and a global buffer/location model that this fork does not have; current code-pane selection-as-context remains retained and already emits file/line/text context into the terminal/ACP input path.
- `140d0952d` Feature flag cleanup workflow token: rejected as upstream CI process change.
- `e7e97a301` Disable local Claude/Codex child harnesses: rejected as old child harness/orchestration surface.
- `b9930c906` Orchestration pill bar pinning: rejected as orchestration UI.
- `3239e96b0` Add RemoteDiffStateModel: not applicable without upstream's split local/remote diff-state module, diff-state protocol, and remote code-review callsites. The current fork keeps the retained monolithic local `DiffStateModel`; adding the upstream remote model now would be unused plumbing.
- `f6faaf0d3` Cloud mode v2 history menu: rejected as cloud mode UI.
- `4e99f5f25` Server forking endpoint for local conversation forking: rejected as server conversation API.
- `5a9bf6712` Rename FileLocation to LocalOrRemotePath: not applicable because the prerequisite FileLocation/buffer-location migration was rejected for this fork. Do not introduce a compatibility alias or unused path abstraction until a retained remote editor/code-review flow owns it.

## Batch 2 Reviewed

Reviewed commits 25 through 44 in chronological order. Compatible local UI, code review, tab config, command-palette, and macOS AppKit changes were manually adapted. Remote-server installer changes were rejected where they depended on Warp-hosted binary downloads.

## Batch 2 Ported Or Adapted

- `cda1694eb` Don't run code review git operations on startup when panel is closed: ported to the retained monolithic `DiffStateModel`. Repository watching still initializes, but metadata/diff loading and watcher-triggered invalidations wait until metadata refresh is enabled.
- `35d951cdc` Make queued prompt text selectable: ported to retained AgentView/terminal pending-query blocks. Queued prompt text now participates in selection, copy-on-select, right-click copy, and selection clearing.
- `152d62c8c` Don't run repo menu git stats on startup: ported by removing the eager repo menu query during inline repo menu construction.
- `095fe37b1` Skip SSH extension install on unsupported remote platforms: ported to retained SSH remote-server setup without telemetry. Unsupported OS/arch detection now maps to `Unsupported`, `armv8l` is no longer treated as `aarch64`, and setup skips install/prompt decisions for positively unsupported hosts.
- `1679cf4d0` Set the minimum window size in AppKit: ported for the retained macOS host path.
- `207f9d5eb` Add `warp://tab_config/<name>` deeplink: ported for retained local tab configs. The handler reuses `Workspace::open_tab_config`; the old WASM stub from upstream was not ported.
- `9600cde9d` Remove dead command palette fixed filters: ported as local search UI cleanup. The stale hidden recent repos/conversations fixed filter path and `HistoricalConversations` query filter were removed, while upstream telemetry and app-side skill/model filters were not restored.

## Batch 2 Rejected Or Not Applicable

- `008969072` Remove lag from add custom model: rejected as old custom inference/model endpoint UI.
- `429fa758c` Orchestrator transcript avatar child-agent pane: rejected as orchestration/child-agent UI.
- `4de3ffdde` Release local-to-cloud handoff flags: rejected as cloud handoff.
- `8b90b4414` Bedrock OIDC SDK error detail: rejected as old agent SDK/AWS credential path.
- `72751748f` Handoff chip toolbar migration: rejected as cloud handoff migration.
- `0d5da4d2d` Discard files panic: not applicable to the current fork because `FileStatusInfo` still uses local `PathBuf`, not upstream's `StandardizedPath` shape.
- `127b626bf` ONNX runtime incompatibility on new NLD classifier: not applicable. The fork's macOS bundle uses the retained `nld_onnx_model` feature and `bert_tiny.onnx`; upstream's `bert_tiny_v2.onnx` file and Linux/Windows bundle toggles are not present.
- `8da83b42a` Markdown Viewer preference for Markdown file links: not applicable to current notebook link code. The fork emits `OpenFile`, and downstream local file opening already resolves the Markdown Viewer preference.
- `cc9ef06a2` Orchestration pill bar horizontal scrolling: rejected as orchestration UI.
- `ae69bd4c7` Cache remote-server tarballs for SCP fallback: rejected for this fork. The patch reintroduces Warp-hosted remote-server tarball downloads and SCP upload fallback, while the current fork's `install_remote_server.sh` intentionally returns `remote server auto-install is unavailable in this build`; there is no remaining standalone `scp_upload` callsite to port.
- `151ef9e56` Warp credit fallback for custom endpoints: rejected as credit/billing plus old server model endpoint behavior.
- `7aa162504` Orchestration pill bar for shared session viewers: rejected as orchestration/shared-session viewer action sync.
- `16df56882` Revert in-app notifications for child agents: rejected because child-agent management is deleted.

## Verification For Ported Batch

- `cargo fmt`
- `cargo nextest run -E 'test(test_autolink) | test(test_parse_url_preserves_non_escaped_backslash) | test(test_url_link_display_text_round_trip_is_stable) | test(test_pointer_opened_tab_configs_menu_does_not_select_top_item)'`
- `cargo nextest run -p repo_metadata`
- `cargo nextest run -p warp -E 'test(test_find_matching_tab_config) | test(test_pointer_opened_tab_configs_menu_does_not_select_top_item)'`

## Batch 3 Reviewed

Reviewed commits 45 through 84 in chronological order. Compatible retained terminal, SSH remote-server, NLD, and tab-config changes were manually adapted. Cloud handoff, shared-session cloud viewer, app-managed MCP, old Agent SDK harness, billing/team, and native Linux/Windows platform changes were rejected.

## Batch 3 Ported Or Adapted

- `48ac96fa2` NLD heuristic v2: ported as the fork's direct behavior instead of upstream's v1/v2 rollout flags. Shell-syntax tokens no longer vote by themselves in `is_likely_shell_command`, and the described-token threshold is pinned to `1.0`. Upstream feature flags and Linux/Windows bundle edits were not ported.
- `f004f4172` Clear action in terminal right-click menu: ported to retained terminal UI as `Clear Blocks`, reusing `TerminalAction::ClearBuffer` and hiding the item for empty block lists or text-selection menus.
- `18baecd45` Remote server socket path limit: ported to retained SSH remote-server/proxy. Daemon identity directories and versioned socket/PID filenames now use short deterministic hashes, and `remote-server-proxy` fails fast when a socket path exceeds the `sun_path` budget.
- `83df17dcb` Restored command history after SSH close: ported by excluding restored `DoneWithNoExecution` blocks from restored command history.
- `a017b9a6a` Tab config sequential commands: ported to retained tab configs. Tab-config command lists now run as a pending-command queue, so each command becomes its own block and Agent mode entry waits until the queue completes.

## Batch 3 Rejected Or Not Applicable

- `036b5d61a` Cloud mode task status reporting: rejected as cloud/ambient conversation status.
- `4542e8d3c` Hide handoff hint in cloud conversation: rejected as cloud handoff.
- `898f4b8b7` Handoff pane conversation content: rejected as cloud handoff/shared-session surface.
- `a4e639c5a` Release tag branch script ordering: rejected as upstream release process.
- `e6837332e` MCP OAuth authenticated client refresh: rejected as app-managed MCP/OAuth.
- `f78edb7f7` Null upload target fields: rejected as old server API/upload target plumbing.
- `b2fc07075` Handoff for child agents: rejected as cloud handoff/child-agent behavior.
- `203f34a4a` Local-to-cloud handoff telemetry: rejected as telemetry/cloud handoff.
- `3efab9aba` Conversation details run API error state: rejected as old server run API/cloud conversation details.
- `0634a5e8c` Branch chip during handoff: rejected as cloud handoff.
- `2e8dcc7e8` Create environment modal: rejected as cloud/remote environment product UI.
- `81cc895d1` Local repo session detection: not applicable to the current fork. The current repo detection path does not have upstream's `RepoDetectionSessionType` branch and already derives detection input from the incoming block metadata/session.
- `723bdf148` Cloud agent tombstone/follow-up visibility: rejected as cloud agent conversation UI.
- `b816f9d21` Orchestration message previews collapsed by default: rejected as old orchestration/child-agent UI.
- `efc1b4cbf` Common skill script resolution: rejected as bundled/common skills.
- `fb9718c6d` WASM bindgens for secret types: rejected as managed secrets/WASM.
- `14205aa35` DetectedRepositories remote backing repos: not applicable to the current fork's retained remote file-tree implementation. This fork already tracks remote repository metadata through `RemoteRepositoryIdentifier`, `RemoteRepoMetadataModel`, and `RemoteRepoNavigated`; the upstream patch depends on `LocalOrRemotePath`/buffer-location refactors plus deleted Agent SDK, MCP, and skills callsites.
- `3f0337de1` RPC for fetching branches: rejected as unused protocol plumbing in the current fork. Upstream explicitly did not wire the callsite, and this fork lacks the prerequisite remote diff/path model. Remote branch picking should be implemented only when a retained remote code-review/tab-config workflow owns the callsite.
- `252afbd62` Server quota display message: rejected as server quota/billing/cloud API behavior.
- `ed92a44c1` Revert Linux/Wayland IME: rejected as native Linux host platform code.
- `a99252686` Local-to-cloud handoff errors: rejected as cloud handoff.
- `8ce7d14a9` File-based MCP wait behavior: rejected as app-managed MCP/file-based server orchestration. ACP owns MCP/tool configuration for this fork.
- `fda540595` Gate DetectedRepositories usage for WASM: rejected as WASM platform gating.
- `4c3c95a7c` Retry Cloud Mode after GitHub auth: rejected as cloud mode/GitHub auth flow.
- `b930996b8` Named agent feature flags: rejected as old agent feature rollout.
- `2abb851e0` Multi-harness flag promotion: rejected as old harness/agent SDK rollout.
- `d6788cbe5` Team full alert: rejected as Teams/billing product UI.
- `032750bd3` Hide handoff details panel: rejected as cloud handoff/shared-session viewer UI.
- `6d5128b8d` Remote envs codebase indexing persistence: rejected as old remote-environment/Agent SDK indexing infrastructure. The current fork retains local `RepoOutlines`-based search-codebase support and remote file-context RPCs, but does not have upstream `CodebaseIndexManager`, remote environment auth/index persistence, or app-owned agent action execution.
- `aa9f9084d` Remote envs codebase indexing UI: rejected with `6d5128b8d`. It is a settings/controller UI for the same removed remote-environment codebase-indexing stack, not ACP-owned context management.
- `127161b2d` Broadcom V3D Vulkan driver downrank: rejected as native Linux rendering/platform workaround.
- `ed01fe452` Inno Setup minidump shutdown: rejected as Windows installer/crash-reporting path.
- `0d6f1d6c6` Non-owner cloud conversation continuation: rejected as cloud conversation permissions.
- `6b1e57e27` Do not fork in cloud for fork-from: rejected as cloud forking behavior.
- `fa732953d` Replay skipping logging in shared session: rejected as cloud/shared-session replay diagnostics.

## Verification For Batch 3 And Final Resolution

- `cargo fmt`
- `cargo fmt -- --check`
- `git diff --check`
- `cargo nextest run -p input_classifier`
- `cargo nextest run -p remote_server`
- `cargo nextest run -p warp -E 'test(test_context_menu_includes_clear_when_block_list_non_empty) | test(test_context_menu_omits_clear_when_block_list_empty) | test(test_context_menu_omits_clear_for_text_right_click) | test(test_clear_buffer)'`
- `cargo check -p warp --all-targets --message-format short`
- `cargo nextest run -p warp -j 1 -E 'test(/search::command_palette::conversations::search_test/) | test(/search::command_search::searcher/) | test(/search::command_search::view/) | test(/search::mixer::mixer_test/)'`
