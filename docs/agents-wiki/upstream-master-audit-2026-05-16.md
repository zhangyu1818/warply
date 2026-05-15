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

## Verification For Ported Batch

- `cargo fmt`
- `cargo nextest run -E 'test(test_autolink) | test(test_parse_url_preserves_non_escaped_backslash) | test(test_url_link_display_text_round_trip_is_stable) | test(test_pointer_opened_tab_configs_menu_does_not_select_top_item)'`
- `cargo nextest run -p repo_metadata`
