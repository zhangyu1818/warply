# Upstream Master Audit 2026-08-16 — Zero-Port Audit

## Scope

- Current fork before this audit: `2c6062147f` (`main`, `v2026.08.15`).
- Upstream source reviewed: `d15645c77..upstream/master` (2 commits, tip `e72fd7aac`).
- Result: no upstream code ported this cycle; both commits triaged below with provenance and rationale.

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `a9c0a1ebd` | Coalesce superseded ProjectContextModel rule refresh tasks (APP-5401) (#15147) | **Not applicable (equivalent invariant already enforced)** | See provenance analysis below. |
| `e72fd7aac` | Add `--title` and `--parent-run-id` to `oz agent run-cloud` (#15187) | **Reject** | Cloud-agent orchestration spawn flags. Only touches `app/src/ai/agent_sdk/ambient.rs` (agent_sdk/ambient removed) and `crates/warp_cli/src/agent.rs` (warp_cli crate absent). `parent_run_id` wires `ORCHESTRATION` lineage/scope inheritance — removed orchestration semantics. |

## `a9c0a1ebd` — why the fix has no target in this fork

The upstream fix coalesces bursts of `RepoMetadataEvent`/`StandingQueryResultsUpdated` events in `ProjectContextModel::refresh_project_rules_for_repo` (`crates/ai/src/project_context/model.rs`), guaranteeing at most one rule-file read in flight per repository with superseding requests coalesced into one follow-up read.

- The patched machinery (`rule_refresh_generations`, `next_rule_refresh_generation`, `refresh_project_rules_for_repo`, `start_rule_refresh`, `standing_project_rule_paths`, `ProjectRuleContentReader`, `remove_project_rules_for_repo`, `GlobalRules`/`remote_global_rules`, and the reader in `app/src/ai/metadata_project_rules.rs`'s standing-results path) comes from the upstream rework chain #9325 → #10921 → #12047 → #11460 → #12767 → #12749/#12669. Verified via `git merge-base --is-ancestor`: none of those rework commits are ancestors of the fork baseline `19659d12` (2026-05-10), and none were ever ported. The fork's `model.rs` sits on the pre-rework #10238/#10377 lineage plus fork-local repo-metadata perf fixes (`5364600f4`, `b45aea1bb`, `04cebca1a`).
- The fork's own architecture already enforces the commit's exact invariant on every event-driven refresh path: `register_watcher_for_path`'s update stream checks `pending_updates` and **queues** superseding repository updates instead of spawning an overlapping task; `apply_update_result` → `drain_pending_updates` then runs exactly **one** coalesced follow-up task that processes all queued updates sequentially against the latest rules. One-shot paths (`new_from_persisted`, `index_and_store_rules`, `try_initialize_and_register_watcher`) are per-root and watcher-guarded (`watched_roots`), so no event-burst spawn path exists here. The upstream bug (many concurrent reads from a burst of standing-query deltas) cannot manifest in this code.
- The fork's `ProjectContextModel` is a live retained path — `app/src/ai/blocklist/context_model.rs` feeds its discovered project rules into ACP request context — which is why the invariant equivalence was verified rather than assumed.
- A `git cherry-pick --no-commit` of the commit conflicts in both `model.rs` and `model_tests.rs` against absent architecture; every hunk targets code that does not exist here. Porting the commit would first require porting the entire upstream rework chain (including #12047's "project skills" standing results and #12669/#12749 remote Agent Mode context snapshots — removed-area-adjacent), which is a separate architectural migration decision, not this bugfix. Nothing was hand-written to "mirror" the fix: the fork's queueing predates this audit (upstream #10238) and already provides the guarantee.
- Upstream's rewritten test `test_superseding_refresh_coalesces_without_overlapping_reads` exercises `refresh_project_rules_for_repo` + `ProjectRuleContentReader` (absent here); the fork's 296-line `model_tests.rs` covers its own model (17 tests: applicable-rules resolution and `RulesDelta` merge semantics).

## Verification

- No code changes this cycle; worktree identical to `v2026.08.15` (verified via `git status` clean after aborting the trial cherry-pick).
- `cargo fmt -- --check`: clean.
- `cargo build -p warp --all-targets --message-format short`: succeeded, followed immediately by `cargo clean`.
- Deletion-surface scans: MCP/skills scan 0 hits; platform scan 6 hits, all pre-existing retained SSH/remote behavior (`ForwardX11=no` in `remote_server/ssh.rs` and `remote_command_executor.rs`, `#[cfg(windows)]` in `local_or_remote_path_tests.rs`, ConPTY notes in `zsh_body.sh`); broad removed-area scan matches the 2026-08-15 baseline (pre-existing docs/comments/tokenizer vocabulary only). This audit introduces no new hits.
