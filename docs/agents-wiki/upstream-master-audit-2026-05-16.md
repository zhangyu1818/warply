# Upstream Master Audit 2026-05-16

Range under review: `1ca5496d8..0d6f1d6c6`

Previous audited upstream tip: `1ca5496d8 docs: clarify bug readiness wording (#10866)`

Current upstream tip fetched: `0d6f1d6c6 fix: allow non-owner cloud conversation continuation (#11051)`

Total upstream commits in this incremental range: 81

Status: in progress. This file records the incremental audit after the 2026-05-14 audit. Do not treat the full range as complete until all 81 commits have a decision.

## Batch 1 Reviewed

Reviewed commits 1 through 24 in chronological order. Compatible changes were manually adapted; no upstream commit was cherry-picked directly.

## Ported Or Adapted

- `5afbb0ce8` Fix New worktree config clipping in tab config menu: ported by removing the vertical-tabs width branch and always using the 268px new-session menu width.
- `5c90c9cc3` Fix link display text escaping for notebooks/plans: ported to the retained markdown parser/editor. Autolinks now unescape markdown backslash escapes before storing URL text, with parser and editor round-trip tests.
- `424e33358` Fall back to lazy index when repo exceeds max file limit: ported to retained local repo metadata. Repos that exceed the full index file limit retry as first-level lazy trees. Upstream telemetry was not ported.

## Deferred For Retained SSH/Remote Work

- `3c4b76c00` Migrate code view to use FileLocation: deferred. The current fork does not have upstream's global buffer/location model, while retained SSH file tree support still needs separate evaluation.
- `3239e96b0` Add RemoteDiffStateModel: deferred. The feature is relevant to retained SSH remote code diff behavior, but upstream assumes a code review diff-state module layout that this fork does not currently have.
- `5a9bf6712` Rename FileLocation to LocalOrRemotePath: deferred with `3c4b76c00`; it depends on the same missing upstream buffer-location model.

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
- `140d0952d` Feature flag cleanup workflow token: rejected as upstream CI process change.
- `e7e97a301` Disable local Claude/Codex child harnesses: rejected as old child harness/orchestration surface.
- `b9930c906` Orchestration pill bar pinning: rejected as orchestration UI.
- `f6faaf0d3` Cloud mode v2 history menu: rejected as cloud mode UI.
- `4e99f5f25` Server forking endpoint for local conversation forking: rejected as server conversation API.

## Batch 2 Reviewed

Reviewed commits 25 through 44 in chronological order. Compatible local UI, code review, tab config, and macOS AppKit changes were manually adapted. Remote-server installer changes were deferred into a dedicated SSH remote-server pass because they are retained but larger than the local UI batch.

## Batch 2 Ported Or Adapted

- `cda1694eb` Don't run code review git operations on startup when panel is closed: ported to the retained monolithic `DiffStateModel`. Repository watching still initializes, but metadata/diff loading and watcher-triggered invalidations wait until metadata refresh is enabled.
- `35d951cdc` Make queued prompt text selectable: ported to retained AgentView/terminal pending-query blocks. Queued prompt text now participates in selection, copy-on-select, right-click copy, and selection clearing.
- `152d62c8c` Don't run repo menu git stats on startup: ported by removing the eager repo menu query during inline repo menu construction.
- `1679cf4d0` Set the minimum window size in AppKit: ported for the retained macOS host path.
- `207f9d5eb` Add `warp://tab_config/<name>` deeplink: ported for retained local tab configs. The handler reuses `Workspace::open_tab_config`; the old WASM stub from upstream was not ported.

## Batch 2 Deferred

- `ae69bd4c7` Cache remote-server tarballs for SCP fallback: deferred to the SSH remote-server pass. Remote server install reliability is retained scope, but this patch is large and should be reviewed with install/download ownership rules.
- `095fe37b1` Skip SSH extension install on unsupported remote platforms: deferred with the remote-server batch. Remote OS/arch gating is retained SSH behavior, but upstream telemetry pieces must be stripped or reduced.
- `9600cde9d` Remove dead command palette fixed filters: deferred to a command-palette cleanup pass. It is local UI, but it touches stale conversation/repo filter modes and telemetry-adjacent code.

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
- `151ef9e56` Warp credit fallback for custom endpoints: rejected as credit/billing plus old server model endpoint behavior.
- `7aa162504` Orchestration pill bar for shared session viewers: rejected as orchestration/shared-session viewer action sync.
- `16df56882` Revert in-app notifications for child agents: rejected because child-agent management is deleted.

## Verification For Ported Batch

- `cargo fmt`
- `cargo nextest run -E 'test(test_autolink) | test(test_parse_url_preserves_non_escaped_backslash) | test(test_url_link_display_text_round_trip_is_stable) | test(test_pointer_opened_tab_configs_menu_does_not_select_top_item)'`
- `cargo nextest run -p repo_metadata`
- `cargo nextest run -p warp -E 'test(test_find_matching_tab_config) | test(test_pointer_opened_tab_configs_menu_does_not_select_top_item)'`
