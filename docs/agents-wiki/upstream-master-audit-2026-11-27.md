# Upstream Master Audit — 2026-11-27

Incremental audit for `e3cc805ec..upstream/master` (`1c7fda119`, 1 commit). The previous audit is `upstream-master-audit-2026-11-26.md`.

## Summary

Zero-port audit. The single new commit lives entirely in the removed `app/src/ai/agent_sdk/` driver: it makes Warp-hosted cloud agent runs fall back to Warp's own agent git identity (`Warp <agent@warp.dev>`) when the server returns no forge credential identity. Its runtime owner, inputs, and dependencies are all fork-absent (the agent-sdk driver runner, the warp-server `TaskGitCredentials` operation, `warp_graphql`, `warp_isolation_platform`, the server API client), every touched symbol was verified absent from the fork before rejection, and the behavior has no retained local or ACP path to reduce to. No removed product surface was reintroduced; no code changed.

## Commit-by-Commit

### `1c7fda119` Ensure git identity for agent runs without forge credentials (#16992) — rejected

Four paths, all under the removed `app/src/ai/agent_sdk/` (370 insertions, 4 deletions): `driver/git_credentials.rs` (+42/−3), `driver/git_credentials_tests.rs` (+210/−1), `mod.rs` (+19), and `mod_tests.rs` (+99).

Source change. `git_credentials.rs` adds `ensure_default_git_identity()`, which calls `ensure_git_identity_if_unset(identity_of(None))`: it reads `user.name` and `user.email` from the `--global` and `--system` git config scopes and, for each key that has no non-empty value, writes the module constants `DEFAULT_GIT_NAME = "Warp"` and `DEFAULT_GIT_EMAIL = "agent@warp.dev"` through `run_git_config` (`git config --global <key> <value>`). `update_assignment_git_identity` calls it after rotating a standby/assignment identity so a run left without any credential still gets an author. `mod.rs` splits the existing `AgentDriverRunner::bootstrap_git_credentials_for_task` body into `bootstrap_task_git_authentication` and calls `ensure_default_git_identity()` once that succeeds, so the default also covers empty credentials, a disabled credential-refresh flag, and a missing workload token. The remaining hunk is a doc-comment reflow in the same file. Tests: ten new subprocess tests in `git_credentials_tests.rs` plus the `with_assignment_git_home` isolation hardening (`XDG_CONFIG_HOME`, `GIT_CONFIG_SYSTEM` in place of `NOSYSTEM`, author/committer env removal), and two in `mod_tests.rs` (`git_identity_bootstrap_covers_empty_credentials_and_skipped_authentication` driving `git_identity_bootstrap_subprocess`, which asserts `global_git_identity() == Some(("Warp", "agent@warp.dev"))`).

Runtime owner. `AgentDriverRunner::build_driver_options_and_task` calls `bootstrap_git_credentials_for_task` when the agent run carries a `task_id`, after resolving the Factory repository contract (`FeatureFlag::FactoryDeferredRepositories`, `FACTORY_ID_ENV`) and `WARP_GITHUB_CREDENTIAL_MODE`. This is the old Warp Agent SDK driver, removed at the fork baseline.

Service dependencies. The module is self-described as "Git credentials management for cloud agent sandboxes": it consumes server-returned credentials (`crate::server::server_api::ai::{AIClient, GitCredential, TaskGitCredentialsError, TaskGitCredentialsResponse}`), `warp_graphql::queries::task_restricted_git_credentials::GitHubCredentialMode`, and `warp_isolation_platform::IsolationPlatformError`, and the new `mod_tests.rs` subprocess test mocks the warp-server `POST /graphql/v2` `TaskGitCredentials` operation against `ServerApiProvider`.

Fork anchors. Zero hits for the symbols these hunks add or call: `AgentDriverRunner`, `AgentDriverError`, `build_driver_options_and_task`, `bootstrap_git_credentials_for_task`, `bootstrap_task_git_authentication`, `ensure_default_git_identity`, `ensure_git_identity_if_unset`, `update_assignment_git_identity`, `identity_of`, `run_git_config`, `configure_git_identity`, `GitCredential`, `GitHubCredentialMode`, `RestrictedInstallation`, `BlockingCommand`, `DEFAULT_GIT_NAME`/`DEFAULT_GIT_EMAIL`, and `TaskGitCredentials`. The only touched symbol with fork hits is `ModelSpawner`, the generic `warpui_core` spawner type. The directories the module depends on are all fork-absent: `app/src/ai/agent_sdk/`, `app/src/server/server_api/`, `crates/graphql/`, `crates/warp_graphql_schema/`, and `crates/isolation_platform/`. The fork's lookalike hits are unrelated: `WARP_WORKLOAD_TOKEN` belongs to the retained agent-hooks runner scrub list (`crates/ai/src/hooks/runner/env.rs`) and its tests, and `warp_graphql` matches only the binary `restored_settings.sqlite` integration fixture.

Not reducible to a retained local utility. The behavior is a fallback author identity for Warp-hosted cloud agent runs. The fork's ACP client launches a user-configured agent process that owns its own git configuration, and substituting Warp's agent identity into the user's global gitconfig would misattribute the user's commits. The fork's only git-identity-aware retained path, the local code-review commit dialog (`app/src/code_review/git_dialog/mod.rs`), maps git's "please tell me who you are" failure to a toast asking the user to set `user.name` and `user.email`; this commit does not touch that path.

Cross-cutting engineering review: no manifest, lockfile, toolchain, formatter/lint configuration, build-script, or workspace-metadata paths are touched.

## Restored-Surface Scans

The three skill scans were run with the literal `rg` commands against the unchanged tree: scan 1 returns 234 hits, scan 2 returns 7, scan 3 returns 16 — identical to the 2026-11-26 documented baselines. A `git grep` baseline-delta of `main` against the working tree (`app crates script Cargo.toml`), with binary-file match lines and output ordering excluded, shows zero text-hit differences in all three scans (228/7/16 text lines on both sides). The only changes are under `docs/agents-wiki/`, outside the scan pathspec.

## Verification

No code changed, so the tree is identical to `v2026.11.26`. It was post-`cargo clean`, so the suite compiled from scratch. All commands ran with `SDKROOT` pointed at the full Xcode SDK and `CARGO_PROFILE_DEV_DEBUG=0`, and the compile commands with `--locked` (as `.github/workflows/ci.yml` does):

- `cargo fmt -- --check` — passed repo-wide.
- `cargo check -p warp --all-targets --locked --message-format short` — passed (64s; zero errors, pre-existing dead-code and unused-variable warnings only).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 157 passed, 3075 skipped (83s).
- `cargo check --workspace --all-targets --locked --message-format short` — passed (53s, zero errors).
- `cargo build -p warp --all-targets --locked --message-format short` — passed (3s on top of the test-binary build, zero errors; `target/debug/warply` linked as an arm64 Mach-O). `cargo clean` runs after the push.

No errors surfaced, so nothing needed fixing. The warnings are the same pre-existing set as the 2026-11-26 run: unused slash-command handler bindings and dead-code sites in `warp` (34 in the lib), the 8 `unsafe_op_in_unsafe_fn` warnings in `warpui`, and one each in `remote_server` and `ai`.

## Tag

`v2026.11.27` on `main` (zero-port audit; full branch shape — `merge/upstream-2026-11-27` merged `--no-ff`).
