# Upstream Master Audit — 2026-11-21

Incremental audit for `cf9434a18..upstream/master` (`280d5eaed`, 5 commits). The previous audit is `upstream-master-audit-2026-11-20.md`.

## Summary

Zero-port audit. All five commits live on fork-absent surfaces: four are the removed `app/src/ai/agent_sdk/` Oz harness stack (standby/prewarmed-sandbox skill loading, sandbox assignment logging, cwd-scoped cloud-skills flag cleanup with its `server_api`/`graphql` companions, and worker skills-folder reference resolution), and one fixes a TUI reauth message in `agent_sdk/mod.rs` plus `crates/warp_tui` (both fork-absent; account-login messaging). The single retained-crate hunk (`crates/warp_features/src/lib.rs` adding `CwdScopedCloudSkills` to `DOGFOOD_FLAGS`) is not applicable: that flag does not exist in the fork's `warp_features`, and cloud skill listings are a removed product area. Every touched path was verified absent in the fork before rejection. No removed product surface was reintroduced; no code changed.

## Commit-by-Commit

### `842e6dea0` A standby reads skill files; the assigned run registers them (#16919) — rejected

Moves skill reading out of the removed driver's run path into the standby preparation (`PreparedSkills` read at prepare time, `RunSkills::FromRepositories`/`FromPrepared` deciding where the assigned run's environment/global skills come from, the standby registering them once the assignment lands). All six paths — `agent_sdk/driver.rs`, `driver/agent_setup.rs`, `driver/agent_setup_tests.rs`, `driver/prewarmed_tests.rs`, `agent_sdk/setup_observability.rs`, `setup_observability_tests.rs` — are fork-absent with the removed `agent_sdk` driver and its skill loading (skills are a removed product area in this fork).

### `9fa48263d` Log run ID when a prewarmed sandbox accepts an assignment (#16936) — rejected

Two-line `log::info!` addition in `agent_sdk/standby.rs` `wait_for_assignment`, logging the run id a prewarmed sandbox is assigned to. Fork-absent path (no `agent_sdk`, no prewarmed sandbox).

### `fdb20a15f` Gate cwd-scoped cloud skills on the feature flag alone and enable it for dogfood (#16938) — rejected

Removes the `experimental.skillsListingDeltas` / `ExecutionConfiguration.experimentalJson` experiment paths so only `CwdScopedCloudSkills` decides cwd-scoped cloud skill listings, and adds that flag to `DOGFOOD_FLAGS`. Touched retained-crate path: `crates/warp_features/src/lib.rs` (+`FeatureFlag::CwdScopedCloudSkills` in `DOGFOOD_FLAGS`) — not applicable, the fork's `warp_features` has no `CwdScopedCloudSkills` flag (verified: no occurrences outside upstream), and cloud skills are removed. Everything else is fork-absent: `agent_sdk/**` (driver/agent_setup/execution_config/standby_test_support/mod_tests), `app/src/server/server_api/**` (prewarmed sandbox + testdata), `crates/graphql/**`.

### `bc0f4ffb2` Propagated the correct error message for tui NeedsReauth error (#16943) — rejected

Reauth prompt wording for TUI login failures (`authenticate_then_execute` prefixing "You are not logged in." and the hint phrasing, plus the `warp_tui` session message). Both paths are fork-absent: `app/src/ai/agent_sdk/mod.rs` with the removed driver/auth surface (account auth is a removed area), and `crates/warp_tui/src/session.rs` (the fork has no `warp_tui` crate).

### `280d5eaed` fix: resolve skill references in worker skills folders (#16937) — rejected

`definition_level` in `agent_sdk/driver/skill_references.rs` now also accepts `workers/` next to `agents/` when resolving a skills dir's agent level, with a new test. Fork-absent path; app-side skill discovery/reference materialization is a removed product area in this fork.

## Restored-Surface Scans

All three skill scans re-run against the unchanged tree: scan 1 returns the same 234 hits, scan 2 the same 7, scan 3 the same 16 as the 2026-11-20 baseline — no new hits, the sets remain the documented inert/local ones.

## Verification

No code changed, so the tree is identical to `v2026.11.20`. All commands run with `SDKROOT` pointed at the full Xcode SDK and `CARGO_PROFILE_DEV_DEBUG=0`:

- `cargo fmt -- --check` — passed repo-wide.
- `cargo check -p warp --all-targets --message-format short` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 157 passed.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo build -p warp --all-targets --message-format short` — passed, followed immediately by `cargo clean`.

## Tag

`v2026.11.21` on `main` (zero-port audit; direct doc commit per the 2026-11-07/2026-11-10/2026-11-12/2026-11-14/2026-11-15 precedent).
