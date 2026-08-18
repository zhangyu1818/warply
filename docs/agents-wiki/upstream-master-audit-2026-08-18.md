# Upstream Master Audit 2026-08-18

## Scope

- Current fork before this audit: `15351e788` (`main`, `v2026.08.17`).
- Upstream source reviewed: `5071a868ce..upstream/master` (11 commits, tip `f466967f03`).
- Result: 2 commits ported (`33bb01256` command-signatures bump, `c6609ef23` repo_metadata gitignore cache); 9 rejected or not applicable.

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `bbec37f3d` | Validate Factory files against the server only (REMOTE-2868) (#15219) | **Reject** | Follow-up to the rejected Factory bundled skill. All paths live in removed areas: `resources/bundled/skills/factory-files/**`, `app/src/ai/skills/bundled_tests.rs`, `script/test_factory_files_skill.py`, `specs/REMOTE-2727/TECH.md`. Third consecutive bundled-skill attempt; boundary unchanged. |
| `ee9ca92a7` | Enable well_known_mcp_ids and factory_mcp feature flags in production (#15237) | **Reject** | Production enablement of app-side MCP feature flags (`app/Cargo.toml`, `app/src/features.rs`, `crates/warp_features/src/lib.rs`). App-managed MCP remains rejected; the fork has no `WellKnownMcpIds`/`FactoryMcp` flags. |
| `532667498` | docker/agent-dev: install pgvector via postgresql-16-pgvector (#15241) | **Reject** | warp-server Docker dev infra; `docker/` is removed in the fork. |
| `f4e85a8f1` | Update getsentry/action-release to v3.7.0 (#15242) | **Not applicable** | Fork's `.github/workflows/create_release.yml` has no Sentry release step (verified: no `action-release`/Sentry lines), so the hunk has no anchor. |
| `8eba8ae95` | Teams settings: workspace admins act as team admins, Workspace admin/owner badge (#15123) | **Reject** | Teams/workspace role UI (`app/src/workspaces/**`, `teams_page.rs`, GraphQL schema) — all removed surfaces. |
| `cf23bb390` | Surface team member removal failures in Teams settings UI (#14964) | **Reject** | Teams GraphQL mutation feedback in `user_workspaces.rs` — removed surface. |
| `8ad89e87a` | Gate invite-by-link on team visibility in desktop Teams settings (#15129) | **Reject** | Threads `Team.visibility` through `crates/graphql/`, `warp_graphql_schema`, workspaces, Teams settings — all removed. |
| `33bb01256` | Bump warp-command-signatures pin: mpv/ruff completions, refresh deno (#15248) | **Accept (ported)** | Cargo pin `d79e09c4` → `ac69f9b0` applied exactly to `Cargo.toml` + the two `Cargo.lock` source lines. Conflict resolution omitted fork-deleted `winit`/`x11rb` lines; a `cargo update`-triggered windows-sys 0.59→0.52 downgrade in 12 unrelated packages was reverted so the lock diff matches upstream's 2-line change. |
| `c6609ef23` | APP-4828: share and cache gitignore matchers in repo_metadata file-tree traversal (#15240) | **Adapt (ported)** | See provenance below. |
| `31e9105d9` | Scope team pending email invites to their own team (#15121) | **Reject** | Teams pending-invite scoping through `Workspace.pendingEmailInvites` GraphQL — removed surface. |
| `f466967f0` | Teams settings: native-workspace states for teamless users (#15246) | **Reject** | Teams settings page composition for native workspaces — removed surface. |

## Provenance: `c6609ef23` port detail

Ported from the exact upstream patch (new files copied verbatim; modified files received the upstream hunks with type rewiring `Vec<Gitignore>` → `Vec<Arc<Gitignore>>`):

- `crates/repo_metadata/src/gitignore_cache.rs` and `gitignore_cache_tests.rs`: copied exactly (content-hash keyed LRU cache; 5 new tests all pass).
- `crates/repo_metadata/Cargo.toml` (`parking_lot.workspace = true`), `src/lib.rs` (`mod gitignore_cache;`), `Cargo.lock` (`+ "parking_lot",`).
- `entry.rs`: unconditional `use std::sync::Arc;` (upstream removed a `local_fs` cfg the fork never had), `use crate::gitignore_cache;`, 8 signature rewires, `evaluate_entry` and `gitignores_for_directory` now call `gitignore_cache::get_or_parse` / wrap the global gitignore in `Arc::new`.
- `file_tree_store.rs` (3 sites), `file_tree_store/file_tree_state.rs` (1), `local_model.rs` (2), `repository.rs` (1).
- `crates/ai/src/index/file_outline/mod.rs` (`Outline::gitignores` field + getter) and `native.rs` (`Arc::new` wraps) per the upstream mechanical follow-through.
- Tests: fork's `entry_test.rs` (Arc wrap) and `local_model_test.rs` (7 Arc wraps) received the equivalent of the upstream `entry_tests.rs`/`local_model_tests.rs` hunks; the fork's test files use singular names and lack some upstream tests.

Intentionally omitted upstream paths and hunks:

- `crates/ai/src/index/full_source_code_embedding/{codebase_index,codebase_index_tests,manager}.rs` — the directory is empty in this fork (cloud codebase-index embedding removed); no anchor exists.
- `entry.rs` hunks for `build_tree_with_standing_queries`, `build_tree_with_force_included_paths*`, `should_watch_repo_directory`, `repo_watch_filter` — the fork deleted `standing_queries.rs` and the multi-arg watch-filter architecture; no anchor symbols exist.
- `watcher.rs` hunks adding `gitignores` parameters to `start_watching_directories`/`start_watching_directory` — the fork's watcher uses the single-arg `WatchFilter` architecture (2026-08-03 adaptation) and never passes gitignores.

Fork integration glue (handwritten, not upstream code):

- `crates/ai/src/project_context/model.rs`: the fork-specific `scan_directory_for_rules` caller of `Entry::build_tree` was retyped to `Vec<Arc<Gitignore>>` with an `Arc` import. This is the only call site outside the upstream patch's file set.

## Verification

- `cargo check -p repo_metadata --all-targets` / `-p ai` / `-p warp --all-targets` / `--workspace --all-targets`: all pass (pre-existing warnings only).
- `cargo nextest run -p repo_metadata -p ai`: 201 tests passed (includes the 5 new `gitignore_cache` tests).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: 156 passed.
- `cargo fmt -- --check`: clean after applying rustfmt to the two over-long retyped lines.
- `cargo build -p warp --all-targets --message-format short`: succeeded; `cargo clean` after the release push.
- Deletion-surface scans: MCP/skills scan 0 hits; broad removed-area scan and platform scan file sets identical to the `v2026.08.17` baseline (no new hits introduced).
- Environment note: the Xcode Metal Toolchain component had to be re-downloaded (`xcodebuild -downloadComponent MetalToolchain`) before any crate using `crates/warpui/build.rs` could compile; unrelated to the ported changes.
