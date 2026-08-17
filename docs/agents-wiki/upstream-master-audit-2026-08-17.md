# Upstream Master Audit 2026-08-17 — Zero-Port Audit

## Scope

- Current fork before this audit: `9072cd02f` (`main`, `v2026.08.16`).
- Upstream source reviewed: `e72fd7aacb..upstream/master` (3 commits, tip `5071a868ce`).
- Result: no upstream code ported this cycle; all three commits triaged below with path-level provenance.

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `19dc50535` | Add bundled Factory files skill and schemas (REMOTE-2727) (#15039) | **Reject** | Bundled-skill restoration. All content paths are new files in removed areas: `resources/bundled/skills/factory-files/**` (bundled-skills directory must remain absent), `app/src/ai/skills/bundled_tests.rs` (app skills module removed), `script/test_factory_files_skill.py`, and `specs/REMOTE-2727/TECH.md` (upstream specs rejected). The only hunk in a retained file (`script/presubmit`) solely invokes the rejected `test_factory_files_skill.py`, so it is rejected with it. |
| `c841dc4f5` | Fix add-on credits copy to describe the team's shared pool (REV-2182) (#15207) | **Reject** | Billing/credits copy fix. Only touches `app/src/settings_view/billing_and_usage_page_v2.rs`, which is absent in this fork (billing & usage surface removed). No anchor symbol exists; not applicable. |
| `5071a868ce` | Migrate client off Workspace.inviteCode onto Team.inviteLink (REV-2173) (#15192) | **Reject** | Teams/workspace GraphQL migration. Touches `crates/graphql/`, `crates/warp_graphql_schema/` (both removed), `app/src/workspaces/**`, `app/src/settings_view/teams_page.rs`, and Teams/billing test fixtures (all removed). The one fork-existing file, `app/src/integration_testing/assertions.rs`, has none of the hunk's anchors (`invite_code`, `join_a_workspace`, team fixtures) — verified by search — so the commit is fully not applicable. |

## Provenance details

- `19dc50535` is the second consecutive upstream attempt to add the Factory bundled skill (the 2026-08-08 audit already rejected the `factory-mcp` bundled-skill variant). The decision boundary is unchanged: app-bundled skills belong to the ACP agent process, and `resources/bundled/skills/` must remain absent, not merely excluded from bundling scripts.
- `c841dc4f5` and `5071a868ce` are pure removed-surface changes (billing copy, Teams invite-link GraphQL schema/client). Neither contains any local terminal, ACP, editor, or macOS-host subsumed hunk.

## Verification

- No code changes this cycle; worktree clean before and after triage (no cherry-pick trials were needed — every touched path is either a new file in a removed area or has no anchor in the fork).
- `cargo fmt -- --check`: clean.
- `cargo build -p warp --all-targets --message-format short`: succeeded (pre-existing warnings only), `cargo clean` to follow after the release push per merge workflow.
- Deletion-surface scans: MCP/skills scan 0 hits; platform scan matches the 2026-08-16 baseline (pre-existing retained items only: `#[cfg(windows)]` in `local_or_remote_path_tests.rs`, ConPTY notes in `zsh_body.sh`, `ForwardX11=no` in SSH/remote paths); broad removed-area scan file set identical to the v2026.08.16 baseline (zero code delta this cycle). This audit introduces no new hits.
