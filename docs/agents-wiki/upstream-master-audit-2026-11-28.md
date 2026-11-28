# Upstream Master Audit — 2026-11-28

Incremental audit for `1c7fda119..upstream/master` (`6b5d34ec8`, 1 commit). The previous audit is `upstream-master-audit-2026-11-27.md`.

## Summary

Zero-port audit. The single new commit edits code that earlier audits already rejected: it drops the deprecated `features.multi_agent` key from the Codex `config.toml` that the removed `app/src/ai/agent_sdk/` harness writes for Warp-driven Codex runs, and seeds a dummy workload token into the shared-session occurrence-time test that the 2026-11-26 audit rejected. All three paths are fork-absent, every symbol the hunks add, remove, or call was verified absent from the fork, and the fork's own Codex path (the ACP `codex-acp` adapter) never writes Codex configuration, so there is no retained local or ACP behavior to carry the change to. No removed product surface was reintroduced; no code changed.

## Commit-by-Commit

### `6b5d34ec8` Use minimal reliable Codex native-agent config (#17009) — rejected

Three paths (6 insertions, 5 deletions), all under the removed `app/src/ai/agent_sdk/` driver: `driver/harness/codex.rs` (+1/−3), `driver/harness/codex_tests.rs` (−2), and `driver_tests.rs` (+5).

Source change. `disable_codex_native_orchestration` stops writing `features.multi_agent = false` and keeps only `agents.enabled = false` and `features.multi_agent_v2 = false`; its comment is reflowed to the shorter form. The two `codex_tests.rs` hunks drop the matching assertions from `prepare_codex_config_toml_writes_fresh_config` and `prepare_codex_config_toml_disables_existing_native_orchestration`. Per the upstream PR description, `features.multi_agent` is a deprecated Codex setting that warns even when set to `false`; entries already present in a reused config are intentionally left untouched. The edited code is what `cfea9b732` (#16915) introduced, which the 2026-11-19 audit rejected on the same surface.

Runtime owner. `disable_codex_native_orchestration` has one caller, `prepare_codex_config_toml`, whose one production caller is `prepare_codex_environment_config`: the function that fills a harness-owned Codex config directory (`codex_config_dir()`) with `auth.json`, a system-prompt agents override, and the `config.toml` (OpenAI base URL taken from a managed secret, the update-check disable, model and reasoning-effort pins, project trust for the working directory and its child git repositories, and MCP servers). Its callers are the `CodexHarness` setup path and the agent-sdk ACP harness (`harness/acp/mod.rs`). The task tracking that the disabled keys protect is Warp's cloud agent orchestration.

Fork anchors. All three paths are absent (`app/src/ai/agent_sdk/` is a removed system). Zero hits for `disable_codex_native_orchestration`, `prepare_codex_config_toml`, `prepare_codex_environment_config`, `set_codex_check_for_update_on_startup`, `set_codex_model`, `set_codex_project_trust_level`, `write_codex_mcp_servers`, `write_codex_agents_override`, `codex_config_dir`, `CodexHarness`, `ThirdPartyHarness`, `HarnessRunner`, `HarnessModelConfig`, `ManagedSecretValue`, `HarnessSupportClient`, `AgentDriver`, `TerminalDriver`, `TerminalDriverEvent`, `EstablishedSharedSession`, `handle_terminal_driver_event`, and for every configuration key involved (`multi_agent`, `multi_agent_v2`, `agents.enabled`, `check_for_update_on_startup`, `trust_level`, `openai_base_url`, `model_reasoning_effort`). The only hits are the generic test helpers the upstream test also uses (`initialize_app_for_terminal_view`, `add_window_with_terminal`, `assert_eventually`) and the `toml_edit` dependency, which the fork uses only for `warpui_extras` user preferences.

Test hunk. `driver_tests.rs` adds to `delayed_shared_session_report_preserves_occurrence_time`, directly after `initialize_app_for_terminal_view`, a seed of a client-local dummy ambient workload token through `ServerApiProvider::as_ref(ctx).get().set_ambient_workload_token_for_test("test-workload-token", None)`. The test was added by `e3cc805ec`, rejected on 2026-11-26 as the shared-session server-reporting surface; per the upstream PR the seed only keeps the test from requesting a real host workload credential before it reaches its mock server, and the matcher, wait budget, and production authentication are unchanged. Fork anchors: zero hits for the test name, `ServerApiProvider`, `set_ambient_workload_token_for_test`, `ambient_workload_token`, `HOOK_TASK_ID`, and `updateAgentTask`.

Not reducible to a retained local or ACP path. The fork's Codex support is the ACP registry's `codex-acp` entry (`@zed-industries/codex-acp` through npx in `app/src/ai/acp/registry.rs`), launched by `adapter_args` from the registry entry's own `args` and `env`, plus the `CLIAgent::Codex` command-prefix detection for the CLI-agent toolbar. Neither touches Codex's configuration: the fork has no `CODEX_HOME` handling and no Codex `config.toml` writer, so it never writes the deprecated key and has nothing to stop writing. The adapter runs on the user's own Codex configuration, and the fork has no orchestration task tracking for the disabled keys to protect.

Cross-cutting engineering review: no manifest, lockfile, toolchain, formatter/lint configuration, build-script, or workspace-metadata paths are touched.

## Restored-Surface Scans

The three skill scans were run with the literal `rg` commands against the unchanged tree: scan 1 returns 234 hits, scan 2 returns 7, scan 3 returns 16 — identical to the 2026-11-26 and 2026-11-27 documented baselines. `git diff --stat main HEAD -- app crates script Cargo.toml` is empty and the working tree has no changes under those paths, so no scanned file differs from `v2026.11.27`. The only changed files are under `docs/agents-wiki/`, outside the scan pathspec.

## Verification

No code changed, so the tree is identical to `v2026.11.27`. It was post-`cargo clean`, so the suite compiled from scratch. All commands ran with `SDKROOT` pointed at the full Xcode SDK and `CARGO_PROFILE_DEV_DEBUG=0`, and the compile commands with `--locked` (as `.github/workflows/ci.yml` does):

- `cargo fmt -- --check` — passed repo-wide (4s).
- `cargo check -p warp --all-targets --locked --message-format short` — passed (65s; zero errors, pre-existing dead-code and unused-variable warnings only).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 157 passed, 3075 skipped (83s).
- `cargo check --workspace --all-targets --locked --message-format short` — passed (54s, zero errors).
- `cargo build -p warp --all-targets --locked --message-format short` — passed (3s on top of the test-binary build, zero errors; `target/debug/warply` linked as an arm64 Mach-O). `cargo clean` runs after the push.

No errors surfaced, so nothing needed fixing. The warnings are the same pre-existing set as the 2026-11-27 run (the tree and the pinned 1.94.0 toolchain are unchanged, and the package and workspace checks report the same warning sites): unused slash-command handler bindings and dead-code sites in `warp`, the 8 `unsafe_op_in_unsafe_fn` warnings in `warpui`, and one each in `remote_server` and `ai`.

## Tag

`v2026.11.28` on `main` (zero-port audit; full branch shape — `merge/upstream-2026-11-28` merged `--no-ff`).
