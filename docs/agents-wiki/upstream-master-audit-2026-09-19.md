# Upstream Master Audit 2026-09-19

Range under review: `bb0a0593ce..upstream/master` (4 commits)

Previous audited upstream tip: `bb0a0593ce computer_use: centralize the recording temp-file path across recorders (#16045)`

Current upstream tip detected: `03832b3a8 [Chat][Attribution] Attribute local, shared-session, and restored UserQuery inputs (#15947)`

Total upstream commits in this incremental range: 4

Status: triage complete. One adapted port (`2c6029018`, warpui hunk only); three commits rejected or not applicable.

## Per-Commit Triage

### `2c6029018` — Recognize TIFF images in the macOS clipboard (#16055)

Decision: **adapt/port (warpui hunk only)**. Adds `public.tiff` to the macOS pasteboard image detector in `crates/warpui/src/platform/mac/clipboard.rs`, mapping it to `image/tiff`. This is the retained local macOS GUI clipboard path: the fork's CLI-agent paste flow (`app/src/terminal/view.rs`, `is_cli_agent_paste && clipboard_content.has_image_data()`) sends the `0x16` image-paste trigger byte when image data is present, so recognizing TIFF restores image pasting into CLI agents when macOS supplies TIFF clipboard data — exactly the upstream fix. The GUI path never inspects `mime_type`, so the mime-narrowing behavior is not needed there.

The upstream patch was applied with `git diff <commit>^ <commit> -- crates/warpui/src/platform/mac/clipboard.rs | git apply --3way`. The mime-mapping hunk (`"public.tiff" => "image/tiff"`) applied cleanly; the array hunk conflicted only on the fork's pre-existing `make_nsstring(...)` vs upstream `ns_string!(...)` macro naming and was resolved by keeping the fork idiom while adding `make_nsstring("public.tiff")` in upstream's position (after `public.svg-image`, before `com.compuserve.gif`).

Intentionally omitted paths:

- `crates/warp_tui/src/attachment_bar/image_processing.rs` and `image_processing_tests.rs`: the fork has no `warp_tui` crate. The TUI change narrows TUI attachment classification to `CLIPBOARD_IMAGE_MIME_TYPES` so unsupported mimes fall back to text — a TUI-only concern; the fork's GUI consumer (`has_image_data()`) is unaffected and upstream made no GUI-side change.

### `a0f5eb31a` — [APP-5720] Show "<0.1 credits" and "<$0.01" for tiny charges (#16070)

Decision: **N/A (reject)**. APP-5720 usage-display formatting. The fork has no usage display path: `app/src/ai/blocklist/view_util.rs` contains no `format_dollars`/usage formatting, `app/src/ai/blocklist/usage/` is absent, `app/src/tui_export.rs` is absent, and `crates/warp_tui` does not exist. Matches the standing APP-5720/usage rejections in the 2026-09-16 audit.

### `33f81ed6c` — [Chat] Carry the injected shared-session UserQuery into agent input (#16048)

Decision: **reject**. Server-injected `Request.Input.UserQuery` transport for shared sessions: new `app/src/ai/agent/base_user_query.rs`, `AgentPromptRequest.user_query_b64`, and threading through the shared-session adaptor/controller/startup queue into `AIAgentInput::UserQuery { base }`. Every anchor is fork-absent (verified):

- `app/src/ai/agent/api/` (convert_to/convert_from/impl) — removed old Warp Agent API surface.
- `app/src/ai/blocklist/controller/shared_session.rs`, `startup_queue.rs` — absent (cloud session sharing removed).
- `app/src/terminal/local_tty/terminal_view_adaptor.rs`, `app/src/terminal/shared_session/` — absent.
- The fork's `AIAgentInput::UserQuery` (`app/src/ai/agent/mod.rs`) has no `base` field; `BaseUserQuery`/`user_query_b64` have zero repo hits.
- `crates/persistence/src/model.rs` hunks operate on `ChargedUsageTotals`, which the fork's persistence model does not have.
- The `warp_multi_agent_api` / session-sharing-protocol repins describe removed server transports.

### `03832b3a8` — [Chat][Attribution] Attribute local, shared-session, and restored UserQuery inputs (#15947)

Decision: **reject (one bundled hunk N/A)**. Attribution layer on top of #16048: warp-server author resolution (`WarpClient` origin marker), viewer-typed shared-session attribution via `PresenceManager`, and `BaseUserQuery::from_message` restoration. All hunks anchor on #16048's fork-absent surfaces (`base_user_query.rs`, `api/convert_*`, presence manager, shared-session sharer). The `app/src/ai/blocklist/controller/slash_command.rs` hunk adds `base: None` to `InvokeSkillUserQuery` — skills are a removed surface and the type does not exist in the fork. The bundled `app/src/cloud_object/model/model_tests.rs` flake fix targets `test_force_refresh_correctly_resets_timestamp`, which does not exist in the fork's cloud_object model tests (force-refresh scheduling was removed with cloud sync).

## Verification

Run on `merge/upstream-2026-09-19` with `CARGO_PROFILE_DEV_DEBUG=0`:

- `cargo check -p warpui --all-targets --message-format short` — pass (pre-existing warnings only).
- `cargo check -p warp --all-targets --message-format short` — pass (pre-existing warnings only).
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — pass (156/156).
- `cargo build -p warp --all-targets --message-format short` — pass.
- Deleted-surface scans on the diff-touched file: no hits; the only changed file is `crates/warpui/src/platform/mac/clipboard.rs` (+2 lines).

`cargo clean` run after merge/tag/push.

## Notes

- Upstream clipboard image-detection changes continue to port into `crates/warpui/src/platform/mac/clipboard.rs` with the fork's `make_nsstring` idiom in place of upstream's `ns_string!` macro.
