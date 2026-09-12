# Upstream Master Audit 2026-09-12

Range under review: `1012545632..upstream/master` (12 commits)

Previous audited upstream tip: `1012545632 Shared-session cancel terminates the in-flight agent command (#15906)`

Current upstream tip detected: `4143c09ff Bump command signatures for du --time (#15950)`

Total upstream commits in this incremental range: 12

Status: triage complete. Four ports landed on `merge/upstream-2026-09-12`:

- `33c3bf6b7` — full port of the duplicate Code Review discard-confirmation crash guard (view fix + regression test), with the fork's `&Path` `discard_file` signature preserved (guard ends the borrow via `.first().cloned()` instead of upstream's owned `to_standardized_path` conversion, which belongs to the fork's un-refactored call shape).
- `3529ae637` — full port of the About-page typed-action parentage fix (About is created via `add_typed_action_view` so a tab drag cannot orphan the hidden page); the upstream hunk context containing the Shared blocks page is not applicable.
- `4143c09ff` — full port of the `warp-command-signatures` rev bump (`d69b340e` → `48fd7aa5`, `du --time` completions); manifest and both Cargo.lock source lines match upstream exactly with no extra lock churn.
- `b0ed2d4a7` — adapted port of the `actions/upload-artifact` v6 → v7.0.1 security bump onto this fork's single fork-authored `Upload DMG artifact` usage in `create_release.yml`.

Five dependabot workflow bumps were not applicable (fork retains only `ci.yml` and `create_release.yml`, and the fork's copies do not reference the bumped actions), and three commits were rejected with zero fork anchors.

## Per-Commit Triage

### `b0ed2d4a7` — bump actions/upload-artifact from 6.0.0 to 7.0.1 (#15900)

Decision: **adapt** (one retained build path). Upstream bumps nine `upload-artifact` pins across its 2600-line `create_release.yml`; this fork's release workflow is fork-authored (Warply macOS arm64 packaging + Sparkle appcast, `8a14a62f3c`/`d29e9604a2f`) and contains exactly one `actions/upload-artifact` usage. That pin is repinned to the upstream v7.0.1 commit `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` with the upstream `# v7.0.1` comment style. The other eight upstream job contexts (Windows/Linux/TUI/schema artifact steps) do not exist here.

### `a58e8bb5b` — bump namespacelabs/nscloud-checkout-action (#15898)

Decision: **not applicable**. Touched workflows `changelog_draft.yml` and `feature_flag_cleanup.yml` are not retained in this fork.

### `76761c22e` — bump azure/artifact-signing-action 1.2.0 → 2.0.0 (#15899)

Decision: **not applicable**. The fork's `create_release.yml` has no artifact-signing job (codesigning runs inside `script/bundle`); the upstream hunks target jobs absent here.

### `eeca17f00` — bump taiki-e/install-action 2.85.13 → 2.86.8 (#15901)

Decision: **not applicable**. The fork's `ci.yml` contains no `taiki-e/install-action` usage.

### `728c02f84` — bump slackapi/slack-github-action 3.0.5 → 4.0.0 (#15902)

Decision: **not applicable**. `feature_flag_cleanup.yml` is not retained; the fork's `create_release.yml` has no Slack notification step.

### `bd5f518f3` — Keep the run's URL when you open a subagent in the web session viewer (#15317)

Decision: **reject** (cloud web session viewer surface, no anchors). The implementation lives in `app/src/uri/browser_url_handler.rs`, `browser_url_resolution.rs`, and `web_intent_parser.rs`, all absent from this fork (its `app/src/uri/` retains only launch-config, settings-deeplink, and docker handling). `uri_tests.rs` is absent (fork uses `uri_test.rs`). Searches for `browser_url_handler`, `web_intent`, `session_viewer`, `subagent_run`, and `open_subagent` under `app crates` return no hits. The bundled `specs/QUALITY-1764/**` planning docs are rejected by contract.

### `7bfa9da6b` — Add docker-registry secret type to `oz secret create` CLI (#15913)

Decision: **reject** (managed secrets / agent SDK, no anchors). Touched paths `app/src/ai/agent_sdk/secret*.rs` and `crates/warp_cli/src/secret*.rs` are both absent; `rg` for docker-registry secret symbols returns nothing. Warp-managed secrets are removed product surface.

### `7f353655f` — Validate Factory benchmark resources (#15893)

Decision: **reject** (app-bundled skills, no anchors). Touched paths `resources/bundled/skills/factory-files/**` and `script/test_factory_files_skill.py` are absent from this fork; bundled skills directories are rejected by contract even when inert.

### `33c3bf6b7` — Guard duplicate Code Review discard confirmations (#15884)

Decision: **accept/adapt** (retained code-review UI crash fix). Upstream turns the stale second `ConfirmDiscardFile`'s `discard_file_paths[0]` index into a `first()`-guarded no-op. Both touched files exist here. Fork divergence: this fork's `discard_file` takes `&Path` (fork lineage `33cd12bcf` kept the pre-refactor signature while upstream moved to `StandardizedPath` plus a `to_standardized_path` call-site wrapper). Ported resolution keeps the fork's `Path::new(&file_path)` call and ends the immutable borrow with `.first().cloned()` (a direct borrow of `first()` conflicts with `&mut self` under E0502; upstream avoids this by producing an owned `StandardizedPath`). The upstream regression test `test_duplicate_single_file_discard_confirmation_is_ignored` is ported verbatim between the same neighboring anchor tests.

### `30494f457` — Fall back to a forge identity only when nothing else claimed a repo (#15869)

Decision: **reject** (agent SDK driver, no anchors). All three touched paths (`app/src/ai/agent_sdk/driver/environment.rs`, `git_credentials.rs`, `git_credentials_tests.rs`) are absent; `ForgeIdentity`/credential-claiming symbols have no fork hits. This is cloud-environment credential orchestration, not ACP behavior.

### `3529ae637` — Parent About page to Settings during tab transfer (#15948)

Decision: **accept/adapt** (retained macOS settings UI crash fix). `AboutPageView` gains `impl TypedActionView { type Action = (); }` and is created through `ctx.add_typed_action_view(AboutPageView::new)` like every other settings page, preventing the hidden About page from being orphaned during a tab drag. Fork adaptations: the import edit lands inside the fork's single grouped `use warpui::{...}` tree (`TypedActionView` sorted into the uppercase segment), and the upstream hunk's surrounding context (the Shared blocks page with `ServerApiProvider::get_block_client`) belongs to a removed surface and is omitted — only the About call change applies. `add_typed_action_view` is already the standard creation path in this fork's `settings_view/mod.rs`.

### `4143c09ff` — Bump command signatures for du --time (#15950)

Decision: **accept** (retained completions data dependency). `warp-command-signatures` rev `d69b340ef622540ac28340ef565dae45287d51d8` → `48fd7aa5b7b4d99f5db57d1c5a25c47c05567fcf` in `Cargo.toml`, plus the two lock `source` lines (`warp-command-signatures`, `warp-completion-metadata`). Applied as the exact upstream string replacement; `cargo metadata` confirms the lock stays consistent with no additional dependency churn, matching the upstream Cargo.lock diff (3 insertions / 3 deletions).

## Verification

Run on `merge/upstream-2026-09-12` after all ports:

- `cargo nextest run -p warp -E 'test(test_duplicate_single_file_discard_confirmation_is_ignored)'` — pass (1/1).
- `cargo nextest run -p warp -E 'test(code_review)'` — pass (110/110).
- `cargo nextest run -p warp -E 'test(settings_view) | test(about_page)'` — pass (12/12).
- `cargo fmt -- --check` — pass.
- `cargo check -p warp --all-targets --message-format short` — pass (pre-existing warnings only).
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — pass (156/156).
- `cargo build -p warp --all-targets --message-format short` — pass; `cargo clean` run after the merge/tag/push.
- Deleted-surface scans (`rg` for cloud/auth/billing/telemetry, MCP/skills, and Linux/Windows host patterns) — all remaining hits are pre-existing: `WeakViewHandle::upgrade`-style Rust API names, fork notes/test comments mentioning Warp Drive in prose, BERT tokenizer vocabulary JSON, retained `ForwardX11=no` SSH options, `#[cfg(windows)]` guards in the retained `local_or_remote_path` tests, and ConPTY mentions inside the bundled zsh bootstrap comments. The `telemetry` word in `code_review/git_dialog/mod.rs` is a stale upstream comment on a fork-stripped path (no telemetry call sites in the module); it predates this merge and was not touched by it.

## Notes

- This cycle's dependabot batch shows upstream CI churn is otherwise confined to workflows this fork does not retain.
- Re-evaluate the shared-session/web-viewer stream (`bd5f518f3` lineage) only if upstream splits a local URI-handling fix out of the cloud session-viewer feature.
