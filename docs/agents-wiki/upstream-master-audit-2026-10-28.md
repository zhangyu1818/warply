# Upstream Master Audit — 2026-10-28

Incremental audit for `1e6c3ae46..upstream/master` (`eafc6d18d`), 3 commits. The previous audit is `upstream-master-audit-2026-10-23.md`.

## Summary

One adapted port: the agent status-bar history subscription accumulation fix (`eafc6d18d`) — output updates are now handled inside the status bar's lifetime subscription using the current exchange IDs, and the per-exchange `on_updated_output` registration (a WarpUI lifetime subscription per exchange, never removed when the boxed exchange model is replaced) is deleted. The upstream regression test is ported with fork-native fixtures. Two commits are not applicable: the Octo STS trust-policy file and the `GITHUB_TOKEN` permissions rollout both touch upstream-only GitHub CI infrastructure that the fork replaced with its own workflows.

## Commit-by-Commit

### `1ed604025` Add Octo STS trust policy channel-versions-read (#16656) — not applicable

Adds `.github/chainguard/channel-versions-read.sts.yaml` so the `warpdotdev/channel-versions` repo can mint a `contents: read` token for the upstream repo via Octo STS. Policy-only, no workflow changes. The fork has no `.github/chainguard/` directory and no channel-versions consumer; its `.github/workflows/` contains only fork-owned `ci.yml` and `create_release.yml` (last touched by fork commits `8a14a62f3`, `9f78d5c31`). Upstream CI-infrastructure decisions do not transfer.

### `65229eb52` Declare GITHUB_TOKEN permissions for workflows that write to the repo (#16652) — not applicable

Adds minimal `permissions:` blocks across ten upstream workflow files and deletes `close_stale_fix_prs.yml`/`warp_cleanup_fix_prs.yml` (oz-agent PR cleanup jobs). Of the ten touched files, only `ci.yml` and `create_release.yml` exist here and both are fork-owned rewrites (Warply arm64 packaging, Sparkle updater), not upstream copies, so the enterprise read-only-default rollout has no consumer here. No fork workflow writes with `GITHUB_TOKEN` in a way that needs new grants.

### `eafc6d18d` Fix status-bar history subscription accumulation (#16658) — accepted, adapted, ported

Fixes one source of growing main-thread work during long agent conversations: `BlocklistAIStatusBar::reset_model_for_exchange` called `AIBlockModelImpl::on_updated_output`, which registers a new WarpUI subscription on the view's context for every exchange the bar latches onto. WarpUI owns subscriptions for the view's lifetime, so replacing the boxed `active_exchange_model` never removes the previous exchange's callback — every past exchange keeps firing (and re-running `on_updated_active_exchange_output`) on every `UpdatedStreamingExchange` event. The fix instead handles `UpdatedStreamingExchange` in the status bar's own lifetime subscription created once in `new()`: if the event's exchange matches the current active model, process output; else if only the conversation matches, `ctx.notify()`. Both checks run before the per-terminal-view filter, preserving the old unfiltered callback semantics so the current exchange still processes output across terminal surfaces. The `on_updated_output` registration in `reset_model_for_exchange` is deleted verbatim.

Retained-scope classification: `app/src/ai/blocklist/block/status_bar.rs` is the AgentView status-bar shell rendering local ACP conversation state; the touched path has no Warp service dependency. The only backend interaction in `on_updated_active_exchange_output` is the already-ACP-routed summarization-dialog auto-clear.

Ported hunks and intentional divergences from the upstream patch:

- Subscription pre-filter hunk: upstream wraps two `match` arms plus `_ => ()` before the surface-id guard. The fork keeps the `UpdatedStreamingExchange` arm verbatim (the fork's field/param naming `terminal_view_id` already matches; the arm binds only `exchange_id`/`conversation_id`). The `ConversationUsageMetadataUpdated { .. } => ctx.notify()` arm is omitted: the fork's `BlocklistAIHistoryEvent` has no usage variant (usage tracking removed at baseline), and the deleted per-exchange subscription never handled usage events either, so no behavior is lost. With one arm remaining, upstream's `match` is reduced to `if let … { … return; }` (a single-arm match with `_ => ()` would trip `clippy::single_match`).
- `reset_model_for_exchange` hunk: applied verbatim, including upstream's one-line `.map(|model| Box::new(model) as Box<dyn AIBlockModel<View = BlocklistAIStatusBar>>);` formatting.
- Regression test `inactive_exchange_updates_do_not_close_summarization_dialog`: the fork deleted `status_bar_tests.rs` at baseline (its other tests covered removed router/Gemini-agent surfaces), so the file is recreated with upstream's `#[cfg(test)] #[path = "status_bar_tests.rs"] mod tests;` mount and only this test, adapted to fork fixtures:
  - Conversation construction uses the fork's established `view_test.rs` idiom (`history.start_new_conversation` + `conversation_mut().append_reassigned_exchange` + manual `AppendedExchange` emissions) instead of upstream's `AIConversation::new_restored` + `create_api_task`/`create_message`, which build `warp_multi_agent_api` types from a crate the fork does not depend on.
  - `FeatureFlag::AgentTips.override_enabled(false)` is dropped — the flag is removed here and the fork's status bar has no tips gating.
  - `terminal_surface_id` naming → the fork's `terminal_view_id`.
  - The dialog-event subscription uses a plain `let` destructure because the fork's `BlocklistAIStatusBarEvent` is single-variant (`Stop` was pruned at baseline), making upstream's `if let` irrefutable and warning-triggering.
  - The exchange fixture is the fork's `AIAgentExchange` streaming literal (from `view_test.rs`'s `exchange_with_inputs`).
- Verified as a true regression guard: against the pre-fix code the test fails exactly as upstream describes (`[true, false]` vs `[true]` — the stale exchange's accumulated callback closes the current summarization dialog); with the port it passes.
- Untouched fork sibling: `block/cli.rs` also calls `on_updated_output` (CLI subagent blocks), matching upstream, which likewise only changed the status bar.

## Verification

- `cargo check -p warp --all-targets --message-format short` — passed (SDKROOT pointed at the Xcode MacOSX SDK per the CLT tapi prerequisite); remaining warnings are pre-existing dead-code lints, none in the touched files.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo fmt -- --check` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 tests passed, 0 failed.
- `cargo nextest run -p warp --lib -E 'test(status_bar)'` — the ported regression test passes; it fails against the temporarily reverted fix (bug reproduced, then fix restored).
- `cargo build -p warp --all-targets --message-format short` — passed; `cargo clean` run after the push.
- Restored-surface scans over the diff against `main` (two files, 125 added lines): all three scans return zero new hits; repo-wide scans return only the pre-existing allowed hits (documentation, tests, retained SSH/remote-terminal paths, shell-integration ConPTY comments).

## Tag

`v2026.10.28` on `main` (merge commit of `merge/upstream-2026-10-28`).
