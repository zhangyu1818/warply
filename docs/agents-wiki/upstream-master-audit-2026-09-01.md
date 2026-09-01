# Upstream Master Audit 2026-09-01

## Scope

- Current fork before this audit: `a64feaadb` (`main`, `v2026.08.31`).
- Upstream source reviewed: `86cfeb9006..upstream/master` (3 commits, tip `6fac731c4`).
- Result: one adapted port (`3a6f05512`, lazy layout for AI plan document editors) and one backfilled deferred port (`885c540634`, newline-as-line-break styling) that fixes a pre-existing test failure surfaced by the port's test group; the other two commits rejected or not applicable.

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `e100bf002` | Fall back to raw transcripts for third-party oz run conversations (#15642) | **Reject / not applicable** | Touches only `app/src/ai/agent_sdk/{ambient.rs,ambient_tests.rs}` and `app/src/server/server_api/ai.rs` — the old Warp Agent SDK and Warp server API surfaces, both absent since fork creation. The `oz run get --conversation` CLI feature is server-API-backed. |
| `3a6f05512` | [APP-5344] Defer layout for AI plan document editors (#15579) | **Adapt (ported)** | See port record below. |
| `6fac731c4` | Show live cloud connection indicator on third-party agent rich input (#15625) | **Reject / not applicable** | The live/new-cloud-VM chips depend on `shared_session/cloud_conversation_continuation` (`AIQueryRouting`/`CloudRoutingIndicator`), `ambient_agent_view_model`, and cloud-agent session state — all removed surfaces with no anchor symbols in the fork (verified: zero hits for `live_session_indicator`, `new_cloud_vm_indicator`, `CloudRoutingIndicator`, `resolve_ai_query_routing`). The `warpui_core` `debug_child_view_ids()` test-util additions exist solely to support the rejected footer tests and were not taken (no consumer). |

## Port record: `3a6f05512` lazy layout for AI plan document editors

### Runtime-ownership review

The change is a pure local performance fix on a retained path: conversation restore
replays `CreateDocuments`/`EditDocuments` into `AIDocumentModel`, and every revision
font-shaped its whole markdown document synchronously through eager
`RenderState` layout. All execution is on-device (editor models, render state,
local conversation history); no Warp service is involved.

### Applied from the exact upstream source

- `app/src/notebooks/editor/model.rs`: `new_unbound_lazy`, `new_internal(lazy_layout)` threading into `RenderState::new`, `RenderEvent::LayoutUpdated | RenderEvent::PendingEditsFlushed` arm, explicit `RenderEvent::ViewportUpdated(_)` arm, `nested_shell_command_count` integration helper (clean three-way apply plus conflict resolution below).
- `app/src/notebooks/editor/notebook_command.rs`: `is_shell_command` helper and the stale `code_block_type` comment removal.
- `app/src/notebooks/editor/model_tests.rs`: `setup_editor_window` extraction from `model_from_markdown` (kept for future-merge parity).
- `app/src/ai/document/ai_document_model.rs`: `LayoutTiming` enum, `layout_timing` parameter on `create_document_internal`/`create_editor_model`, `will_auto_open` on `get_or_create_streaming_document_for_create_documents`, Lazy at `apply_persisted_content`/`restore_document`/`create_new_document_version`, Eager at `create_document`.
- `app/src/ai/blocklist/block.rs`: `will_auto_open` hoist around the streaming CreateDocuments loop.
- `app/src/integration_testing/ai_document.rs` + `crates/integration/src/test/ai_document.rs` + both test registries: `restore_and_open_ai_document`, version-selection steps, and `test_restored_ai_document_populates_code_block_after_first_layout` (registered; compile-verified only — GUI integration tests remain headless-unrunnable in this environment).

### Fork integration glue (handwritten, provider/adaptation boundary)

- Integration helper renamed `assert_viewed_ai_document_has_code_and_mermaid_controls` → `assert_viewed_ai_document_has_code_block_controls`, asserting the code-block control only (renamed because the Mermaid assertions were omitted; see below).
- `restore_and_open_ai_document` calls the fork's 3-arg `start_new_conversation` (upstream's `is_viewing_shared_session`/`is_cli_agent_transcript` params belong to removed surfaces; both were `false`).
- Trimmed the `PendingEditsFlushed` match-arm comment's "and Mermaid render offsets" clause (the sync it references is omitted).

### Intentionally omitted paths (with reasons)

- All hunks depending on the unported upstream `35cb40c31c` (#10431 Raw/Rendered toggle for Mermaid notebook blocks, 2026-05-09, post-baseline): `sync_mermaid_render_offsets` + `RenderState::set_mermaid_render_offsets`/`mermaid_render_offsets`, `NotebookCommand::is_rendered_mermaid` (`mermaid_display_mode` field absent in the fork), `nested_rendered_mermaid_command_count`, the `set_default_mermaid_display_mode`/Rendered default in `create_editor_model`, and `test_rendered_mermaid_offsets_ignore_shell_commands`. The fork's mermaid architecture is still the pre-#10431 `render_mermaid_diagrams_in_state`/`EditableMarkdownMermaid` form. Porting #10431 (28 files) is a separate feature decision, not a prerequisite of this perf fix.
- `markdown_table_count` wrapper on `NotebooksEditorModel` and `create_document_from_notebook`/`hydrate_saved_plan_from_warp_drive`: incoming-side context from other unported upstream commits; the latter pair is Warp-Drive-notebook hydration removed with the cloud sync surface.
- Mermaid assertions inside the ported integration test (see glue above).

## Port record: `885c540634` newlines as line breaks (#10293) — backfill

### Why

`test_plan_markdown_content_preserves_copyable_structure` (added with Copy-as-Markdown, `c96a018adc`) has been failing on the fork because the fork still stripped blank lines via `post_process_notebook` in every AI-document/notebook ingestion path, while upstream removed the function in `885c540634`. The 2026-05-26 audit had deferred that commit ("patch conflicts with current editor code and includes deleted remote/drive/telemetry paths") and it was never revisited. Surfaced again by this audit's test group; per the standing instruction to fix pre-existing failures, the retained parts were ported source-first.

### Applied from the exact upstream source

- `app/src/notebooks/mod.rs`: `post_process_notebook` deletion.
- `app/src/ai/document/ai_document_model.rs`: all four `post_process_notebook` call sites (`update_to_new_markdown`, `apply_persisted_content`, `create_editor_model`, `restore_document_edit`) now pass content through unchanged.
- `app/src/ai/blocklist/action_model/execute/edit_documents.rs`: diff `search`/`replace` no longer post-processed.
- `app/src/notebooks/file/mod.rs`: `FileLoaded`/`FileUpdated` set content directly.
- `app/src/code/editor/comment_editor.rs`: padding 8→4.
- `app/src/notebooks/editor/mod.rs`: `NOTEBOOK_LINE_HEIGHT_RATIO` 1.6→1.5, `minimum_paragraph_height: Some(base_text.line_height())` replacing `PARAGRAPH_MIN_HEIGHT`, `cursor_width` 1.→3.
- `crates/editor/src/render/model/mod.rs`: `TEXT_SPACING` margin 4→0, `HEADER_SPACING` top/bottom 12→4 (resolved onto the fork's pre-existing `uniform(0.)` left/right divergence).
- `app/src/ai/document/ai_document_model_tests.rs` and the four `warp_editor` render test files: upstream expectations ported verbatim (blank lines now round-trip).

### Intentionally omitted paths (with reasons)

- `app/src/code_review/telemetry_event.rs` and `app/src/drive/import/nodes.rs`: removed surfaces (telemetry, Drive import); the cherry-pick re-created them and they were dropped.
- `send_telemetry_from_ctx!` call in the `FileLoaded` handler (telemetry removed).
- `remote.rs` hunks are import-order-only; kept on the fork's import layout (no `CodeReviewTelemetryEvent`/`BranchEntry` imports).
- Unused `pub use block_insertion_menu::BlockInsertionSource;` re-export not taken (upstream context; no fork consumer, produced an unused-import warning).

## Verification

- `cargo fmt -- --check`: clean.
- `cargo check -p warp --all-targets --message-format short`: pass (8 pre-existing dead-code warnings in untouched files; no new warnings).
- `cargo check --workspace --all-targets --message-format short`: pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: 156 passed.
- `cargo nextest run -p warp -E 'test(ai_document) | test(notebooks::editor::model)'`: 76 passed, including the previously failing `test_plan_markdown_content_preserves_copyable_structure` (verified failing on clean `a64feaadb` before the fix).
- `cargo nextest run -p warp_editor`: 434 passed (render-model expectations updated by `885c540634`).
- `cargo build -p warp --all-targets --message-format short`: pass. `cargo clean` after the release push.
- Deletion-surface scans: zero hits in touched files; workspace-wide hits remain the pre-existing allowed set (warpui_core doc/log wording, retained SSH/remote-terminal platform detection, remote-path tests).
- `CARGO_PROFILE_DEV_DEBUG=0` used for check/test/build as in prior audits.
