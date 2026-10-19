# Upstream Master Audit — 2026-10-19

Incremental zero-port audit for `3f02b7028..upstream/master` (`ae60375ef`), 2 commits. The previous audit is `upstream-master-audit-2026-10-18.md`.

## Summary

Both commits extend upstream workstreams whose surfaces are fork-absent: the ChatGPT subscription usage-limit recovery stack (rejected 2026-10-01) gains a server-authored `CopyText` recovery action, and the agent-sdk pre-warmed standby sandbox stack (rejected 2026-10-16/17/18) is realigned with the server callback contract. No fork-present file carries a live anchor for either change, so nothing was ported.

## Commit-by-Commit

### `1e6ef379b` Copy ChatGPT Business admin requests from usage-limit recovery (#16522) — rejected (removed surface)

Adds a server-authored `CopyText` recovery action kind to ChatGPT subscription errors so Business users can copy a plain-text admin request; the primary button copies to the clipboard via the confirmation-toast path, text-only surfaces append the request as prose, and the client advertises `supports_chatgpt_subscription_copy_text` on the next server request. The whole mechanism is the ChatGPT subscription stack rejected in the 2026-10-01 audit: the errors arrive as `warp_multi_agent_api` proto stream-finished events over the removed Warp Agent server backend, which this fork's ACP path never produces.

Anchor verification, per hunk:

- `Cargo.toml`/`Cargo.lock`: pins `warp_multi_agent_api` to `c5aa18336` (warp-proto-apis PR 401). The fork has no `warp_multi_agent_api` dependency (`rg "warp-proto-apis"` over both files returns zero matches).
- `app/src/ai/agent/api/impl.rs`: adds `supports_chatgpt_subscription_copy_text: !cfg!(feature = "tui")` to the `generate_multi_agent_output` request. The fork has no `app/src/ai/agent/api/` directory and no `generate_multi_agent_output`/`supports_chatgpt_subscription` symbols.
- `app/src/ai/agent/mod.rs` + `mod_tests.rs`: `ChatGPTSubscriptionErrorActionKind::CopyText` variant, its proto `From` conversion, and the `RenderableAIError::ChatGPTSubscriptionError` display arm. The fork's `RenderableAIError` is the single-variant `Other { error_message }` enum with no subscription variant, action kinds, or `from_chatgpt_subscription_error` (`rg -i "chatgpt"` over `app/src` returns zero matches).
- `app/src/ai/blocklist/block/view_impl/common.rs`, `output.rs`, `view_util.rs`, `view_util_tests.rs`: doc/comment updates and rendering for `render_chatgpt_subscription_error`, `dispatch_chatgpt_subscription_action`, and `chatgpt_subscription_message_with_links` — none of these symbols exist in the fork's blocklist.
- `app/src/ai/blocklist/block.rs`: adds `AIBlockAction::CopyText(String)` and merges it into the `CopyAIBlockCodeSnippet` clipboard-write arm. This is the one fork-present-file hunk, but its sole producer is the fork-absent `dispatch_chatgpt_subscription_action`; porting the variant alone would restore a rejected product surface as permanently dead code. The retained `CopyAIBlockCodeSnippet` behavior is unchanged by the hunk.

Nothing separable: the only local behavior (clipboard write + toast) already exists via `CopyAIBlockCodeSnippet`, and every new symbol exists to render server-authored subscription recovery actions.

### `ae60375ef` Align standby client with callback API contract (#16538) — not applicable

Contract alignment for the pre-warmed standby sandbox stack (REMOTE-3488, parts 1–5 rejected as N.A. on 2026-10-16/17/18): configuration requests move the workload token from a duplicate GraphQL input to the existing `X-Warp-Ambient-Workload-Token` header, the wire response switches to `fingerprint`/`spec` fields, and lifecycle callbacks send `fingerprint`. All seven touched paths are fork-absent: `app/src/ai/agent_sdk/driver/standby_bootstrap_tests.rs` (removed Agent SDK), `app/src/server/server_api/prewarmed_sandbox*.rs` (no `app/src/server`), and `crates/warp_graphql_schema/**` (removed GraphQL schema crate). No fork-present file is touched, and `rg "prewarmed_sandbox|standby_bootstrap|run_cloud_standby"` over `app crates Cargo.toml` returns zero anchors.

## Verification

Zero-port audit: no source changes, so verification re-confirms the unchanged tree.

- `cargo fmt -- --check` — passed.
- `cargo check -p warp --all-targets --message-format short` — passed (SDKROOT pointed at the Xcode 26.5 SDK per the CLT 27 tapi prerequisite).
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — passed.
- `cargo build -p warp --all-targets --message-format short` — passed; `cargo clean` run after the push.
- Restored-surface scans — zero delta against `main` for all three scan patterns.

## Tag

`v2026.10.19` on `main` (merge commit of `merge/upstream-2026-10-19`).
