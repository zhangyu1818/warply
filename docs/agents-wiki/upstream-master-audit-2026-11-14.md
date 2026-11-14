# Upstream Master Audit — 2026-11-14

Incremental audit for `8048d09e6..upstream/master` (`7a74a2df8`), 9 commits. The previous audit is `upstream-master-audit-2026-11-13.md`.

## Summary

Zero-port audit. All nine commits live on fork-absent surfaces: the agent-sdk ACP-harness client and its usage/prewarm/sidecar extensions, the managed third-party-harness identity/live-query formatter, the `preview_proxy` cloud-sandbox crate, billing usage, and upstream `specs/**` planning documents. Every shared-file anchor was verified absent in the fork. No removed product surface was reintroduced; no code changed.

## Commit-by-Commit

### `d69768ec6` Preserve per-message identity for third-party harnesses (#16834) — rejected

Adds a message-scoped identity header (origin, resolved author UID, eligible co-author UID) to ordinary managed third-party prompts, formatting live and queued PTY/external-harness prompts before delivery. Pairs with warp-server#21840; the changelog line is shared-conversation identity. Touched files: `app/src/ai/agent/base_user_query.rs`, `app/src/ai/blocklist/controller/external_harness.rs`, `app/src/ai/blocklist/controller/startup_queue.rs`, `app/src/terminal/local_tty/terminal_view_adaptor.rs` plus tests — every one fork-absent (verified: no `base_user_query` references anywhere, no `external_harness.rs`/`startup_queue.rs` in the fork's blocklist controller, no `terminal_view_adaptor.rs` in `app/src/terminal/local_tty/`). The external-harness/live-query stack was rejected with the agent-sdk ACP-harness series (2026-10-10) and shared-session attribution with `8f4865adc` (2026-11-13). Server-companion feature with no separable local utility.

### `bf29dec44` Fix preview proxy listener binding for Namespace ingress (#16833) — rejected

Binds the native preview-proxy listener to all IPv4 interfaces for Namespace cloud-sandbox ingress. `crates/preview_proxy` does not exist in this fork (rejected with `d029c5922` in the 2026-11-13 audit; verified zero references). Cloud-sandbox networking is a removed surface.

### `5558ba14f` Extract Pi transcript token usage for server cost estimation (#16821) — rejected

Bounded, deduplicated JSONL extractor for Pi token/tool usage published through `harness_metadata` so the server can estimate public-API costs. All files under `app/src/ai/agent_sdk/driver/harness/` and `crates/warp_harness_usage/` — both fork-absent (verified: no `agent_sdk` directory, no harness-usage crate). Server-side cost estimation is usage tracking on the removed billing path.

### `829f39588` Warm a harness sidecar's launch files while the workspace is prepared (#16828) — rejected

`LaunchFileWarmup` reads a third-party harness sidecar's runtime/adapter/CLI files on a blocking thread while the cloud-sandbox workspace is prepared. Entirely under `app/src/ai/agent_sdk/` (driver, agent_setup, harness/acp manifest, setup_observability) — fork-absent (verified). Continues the prewarmed-sandbox series rejected 2026-10-18 through 2026-11-12.

### `25b5cc8d6` Gate credit-equivalent billing usage on the plan charge unit (#16838) — rejected

Gates credit-equivalent usage costs on `CreditEquivalentBillingUsage` plus `UserWorkspaces::charge_unit()` across `app/src/settings_view/billing_and_usage/` and its tests. The `billing_and_usage/` directory, the flag, and `UserWorkspaces` are all fork-absent (verified: directory missing, zero flag hits). Billing/usage remains removed.

### `ea4e216ef` Add agent hooks scoping: Cursor hook coverage and architecture proposal (#16558) — rejected

Scoping documents under `specs/agent-hooks/` (architecture proposal, Cursor `hooks.json` coverage matrix, five research files) for a future agent-hooks system. Upstream `specs/**` are rejected by contract; the fork's `specs/` directory was emptied at baseline (`778d37e22`). No code changes.

### `87b1d9e52` Release credit-equivalent billing usage (#16840) — rejected

Moves `CreditEquivalentBillingUsage` from `DOGFOOD_FLAGS` to `RELEASE_FLAGS`. The flag does not exist in the fork's `warp_features` (verified zero hits) and the billing files behind it are fork-absent. Flag-only rollout of a removed product area.

### `7630adb37` Fall back to session/load when an ACP agent cannot session/resume (#16839) — not applicable

Chooses `session/resume` vs `session/load` vs new session from the agent's advertised `initialize` capabilities, stamping notifications with an `Arrival` sequence so replayed history can be discarded. All nine files live in `app/src/ai/agent_sdk/driver/harness/acp/` — the agent-sdk ACP-harness client rejected with the 2026-10-09/10 stack (verified fork-absent). The fork's own ACP client (`app/src/ai/acp/`) has no resume path at all: it always sends `initialize` → `new_session` (`app/src/ai/acp/model.rs`) and rebuilds conversation context from local history through prompt content, so the resume/load selection decision has no anchor to fix in this architecture.

### `7a74a2df8` Fix human co-author credit on live Slack follow-ups (#16842) — rejected

Lets verified human Slack Automation messages retain co-author eligibility in the third-party live-query formatter, matching the companion server guard. Only touches `app/src/ai/agent/base_user_query.rs` and its tests — fork-absent (verified). Extends the identity stack from `d69768ec6`; zero `co_author` anchors in the fork.

## Verification

No code changed, so the tree is identical to `v2026.11.13`. All commands run with `SDKROOT` pointed at the full Xcode SDK and `CARGO_PROFILE_DEV_DEBUG=0`:

- `cargo fmt -- --check` — passed repo-wide.
- `cargo check -p warp --all-targets --message-format short` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — passed.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo build -p warp --all-targets --message-format short` — passed, followed immediately by `cargo clean`.
- Restored-surface scans re-run with unchanged results against the 2026-11-13 baseline.

## Tag

`v2026.11.14` on `main` (zero-port audit; direct doc commit per the 2026-11-07/2026-11-10/2026-11-12 precedent).
