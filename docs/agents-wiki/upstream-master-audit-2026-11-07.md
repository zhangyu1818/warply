# Upstream Master Audit — 2026-11-07

Incremental zero-port audit for `f7fc2f2a0..upstream/master` (`abb3fe33e`), 1 commit. The previous audit is `upstream-master-audit-2026-11-06.md`.

## Summary

Zero ports. The single upstream commit is a bug fix inside the Factory cloud-run skill-reference machinery (`agent_sdk` driver `skill_references.rs`), the same REMOTE-3119 stack rejected in the 2026-11-05 audit — a doubly removed surface (cloud Factory runs plus app-managed skill resolution). Every touched path is verified absent in this fork: there is no `app/src/ai/agent_sdk/`, and repo-wide search over `app`/`crates` finds zero `FACTORY_CODE_FORGE_ENV`, `parse_code_forge`, or `skill_references` symbols. Nothing is separable into a retained local path: the fixed value (the run-resolved Factory code forge) only exists for cloud task runs, and ACP agents own their own skills.

## Commit-by-Commit

### `abb3fe33e` Fix shared skill discovery in prewarmed Factory runs (#16763) — rejected

Upstream fix inside `AgentDriver::discover_skill_references`: the code forge that shorthand skill sources (e.g. `acme/shared`) resolve against was read only from the process environment (`std::env::var(FACTORY_CODE_FORGE_ENV)`). In prewarmed Factory cloud runs the assignment-time `WARP_FACTORY_CODE_FORGE` reaches the run environment and terminal shell but not the already-running Oz process, so shorthand sources resolved against the wrong forge (assigned `GITLAB` produced a `GitHub` checkout request). The fix reads the run-resolved forge from `me.resolved_env_vars` first, retaining the inherited-process fallback for ordinary runs. Adds the regression test `skill_sources_use_the_assigned_factory_forge` driving `AgentDriverOptions`/`Fixture`/`plan_checkouts`/`RepositoryForge` from the same module stack.

Both touched files (`app/src/ai/agent_sdk/driver/skill_references.rs`, `skill_references_tests.rs`) were introduced by `d6b50252c` [REMOTE-3119] "Resolve skill references in Factory cloud runs", rejected in the 2026-11-05 audit as app-managed skill resolution for cloud agent runs. The fork has no `app/src/ai/agent_sdk/` tree, no `crates/ai/src/skills/`, and no `FACTORY_CODE_FORGE_ENV`/`parse_code_forge` symbols anywhere; the fix has meaning only for Warp-hosted Factory runs, so there is no fork anchor and no reducible retained local utility.

## Verification

No code was ported in this cycle, so the fork tree is identical to `v2026.11.06` (`6e39d5df3`, zero commits between tag and `main`), whose cycle passed `cargo check -p warp --all-targets`, `cargo check --workspace --all-targets`, `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` (156 passed), and `cargo build -p warp --all-targets` followed by `cargo clean`. That build state carries over unchanged.

- `cargo fmt -- --check` — passed (fresh run).
- Restored-surface scans re-run over the identical tree: the MCP/skills pattern scan returns zero hits, and the platform scan returns exactly the six known retained occurrences (`ForwardX11=no` SSH config ×2, the retained ConPTY explanatory comments in `zsh_body.sh`, and `#[cfg(windows)]`-gated tests in `warp_util`) — unchanged from the 2026-11-05/06 audit baselines. The cloud/auth/billing/telemetry pattern scan returns only the same pre-existing allowed hits as the prior audits (tests, doc comments, weak-handle `upgrade` wording, retained local-model code).

## Tag

`v2026.11.07` on `main` (zero-port audit commit; no merge branch needed, matching the `v2026.11.05` zero-port precedent).
