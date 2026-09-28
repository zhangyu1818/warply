# Upstream Master Audit 2026-09-28

Range under review: `5af88f49f..upstream/master` (1 commit)

Previous audited upstream tip: `5af88f49f Preserve model preference across team catalog refreshes (#16029)`

Current upstream tip: `cb2416204 Fetch substituted benchmark branches without live default refs (#16172)`

Total upstream commits in this incremental range: 1

Status: triage complete. **Zero-port audit** — the lone commit is rejected with no fork anchors. No fork code changes.

## Per-Commit Triage

### `cb2416204` — Fetch substituted benchmark branches without live default refs (#16172)

Decision: **reject**. The commit is entirely cloud-agent benchmark-trial machinery for the removed `app/src/ai/agent_sdk/` driver stack:

- `app/src/ai/agent_sdk/driver/environment.rs` and `environment_tests.rs` — adds a `fetch_branch_only` flag on `RepositoryCloneRequest` plus the embedded `SUBSTITUTED_BRANCH_SETUP` shell script (git init + fetch of only the substituted branch, mapping plain `git fetch origin` onto the frozen branch via the copy remote's symbolic default `HEAD`). `app/src/ai/agent_sdk/` is listed in the fork contract's Removed Systems; the directory does not exist in this fork. The branch `Head` input is produced by the paired warp-server PR (warpdotdev/warp-server#18330), so the feature is server-coupled end to end.
- `crates/warp_cli/src/agent.rs` and `lib_tests.rs` — relaxes `RepositoryIdentity` branch validation in the `oz agent` substituted-repo checkout CLI surface. The fork's `crates/warp_cli/src/` carries only `completions.rs`, `config_file.rs`, `json_filter*.rs`, and `lib.rs`; there is no `agent.rs`, no `lib_tests.rs`, and zero `RepositoryIdentity`/`clone_from`/benchmark-substitution references under `crates/warp_cli/` or `app/src/ai/`.

Anchor checks: `rg "RepositoryIdentity|fetch_branch_only|SUBSTITUTED_BRANCH_SETUP|clone_from"` has no matches in the fork outside this rejected path family; the only `benchmark`/`substitut` substring hits under `app/`/`crates/` are unrelated (vim-handler tests, tab configs, grep tests, parsers). This continues the standing rejection family for agent-SDK benchmark repo overrides first recorded in the 2026-09-15 audit.

Nothing is separable: the generic git mechanics (init + single-branch fetch + symbolic-HEAD discovery) exist only inside the removed driver environment setup, with no local terminal or ACP consumer in this fork.

## Verification

Zero ports; verification confirms the released tree is healthy. Run on `merge/upstream-2026-09-28` (= `main` + this audit doc, with `CARGO_PROFILE_DEV_DEBUG=0`):

- `cargo check -p warp --all-targets --message-format short` — pass.
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — pass.
- `cargo build -p warp --all-targets --message-format short` — pass.

Deleted-surface scan not applicable (no code changes); the standing scans from the 2026-09-26 audit remain valid for this tree.

`cargo clean` run after merge/tag/push.
