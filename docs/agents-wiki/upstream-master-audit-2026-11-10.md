# Upstream Master Audit — 2026-11-10

Incremental zero-port audit for `abb3fe33e..upstream/master` (`8246c1e1b`), 1 commit. The previous audit is `upstream-master-audit-2026-11-07.md`.

## Summary

Zero ports. The single upstream commit edits upstream's own `.github/workflows/create_release.yml` changelog-agent step: the step posts the draft changelog to the Warp cloud Agent API with `warp_api_key: ${{ secrets.CHANGELOG_AGENT_WARP_API_KEY }}` and `share: team`, and the commit just switches that key from the staging/dev-channel pair to a prod "Warp Stable" team key. Every element of the change is a removed surface: upstream release/changelog plumbing was rejected at baseline in favor of the fork-owned Sparkle updater, and the changelog agent itself runs on the removed Warp cloud Agent API with account-team sharing. The fork's `.github/workflows/create_release.yml` is the fork-owned 66-line Sparkle/GitHub-Release workflow and repo-wide search over `.github/`, `app`, `crates`, `script`, and `Cargo.toml` finds zero `warp_api_key`, `oz_channel`, `CHANGELOG_AGENT`, or changelog-agent symbols. Nothing is separable into a retained local path: the fix's value (which team API key the cloud changelog agent authenticates with) only exists for Warp-hosted agent execution.

## Commit-by-Commit

### `8246c1e1b` Use prod API key for the changelog agent. (#16777) — rejected

Follow-up to upstream PR #16727, which added an LLM changelog-draft step to upstream's release workflow (invoking a named Warp cloud agent with a Warp API key and `share: team`, then uploading `changelog-draft.md` as an artifact). Because the action cannot use the dev channel, this commit returns the step to the default stable channel and swaps `secrets.CHANGELOG_AGENT_WARP_STAGING_API_KEY` for the new zero-secret prod key `secrets.CHANGELOG_AGENT_WARP_API_KEY`, dropping the `oz_channel: dev` override. The only touched path is `.github/workflows/create_release.yml` at the changelog-agent step (~line 2980 of upstream's ~3000-line workflow).

Rejected on two independent grounds. First, the fork replaces upstream's release/changelog machinery wholesale: the change map rejects Warp update/changelog APIs and old release tag formats around the Sparkle 2 updater, and the fork's `create_release.yml` is a different, fork-owned file with no changelog-agent step. Second, even in isolation the step exercises removed product surfaces — cloud agent execution, account API-key auth, and Teams sharing. There is no fork anchor: no upstream changelog-agent step was ever ported, and no retained local code references these symbols.

## Verification

No code was ported in this cycle, so the fork tree is identical to `v2026.11.07` (`e954bfa64`, zero commits between tag and `main`), whose cycle carried over the `v2026.11.06` build state that passed `cargo check -p warp --all-targets`, `cargo check --workspace --all-targets`, `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` (156 passed), and `cargo build -p warp --all-targets` followed by `cargo clean`. That build state carries over unchanged.

- `cargo fmt -- --check` — passed (fresh run).
- Restored-surface scans re-run over the identical tree: the MCP/skills pattern scan returns zero hits, and the platform scan returns exactly the six known retained occurrences (`ForwardX11=no` SSH config in `remote_server/src/ssh.rs` and `remote_command_executor.rs`, the retained ConPTY explanatory comments in `zsh_body.sh` ×2, and `#[cfg(windows)]`-gated tests in `warp_util` ×2) — unchanged from the 2026-11-05/06/07 audit baselines. The cloud/auth/billing/telemetry pattern scan returns only the same pre-existing allowed hits as the prior audits (tests, doc comments, weak-handle `upgrade` wording, retained local-model code).

## Tag

`v2026.11.10` on `main` (zero-port audit commit; no merge branch needed, matching the `v2026.11.05`/`v2026.11.07` zero-port precedent).
