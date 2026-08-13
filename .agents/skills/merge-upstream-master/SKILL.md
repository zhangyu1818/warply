---
name: merge-upstream-master
description: Reviews and ports commits or changes from upstream Warp master into this ACP-only macOS fork. Use for merges, cherry-picks, rebases, restacks, syncs, or selective upstream ports.
---

# Merge Upstream Master

Review and port upstream Warp changes without restoring product areas removed by this ACP-only macOS fork. Product boundaries live in the required project documents below; this skill defines the merge procedure.

## Required Reading

Read these before touching code for upstream merge work:

- `AGENTS.md`
- `docs/agents-wiki/README.md`
- `docs/agents-wiki/fork-contract.md`
- `docs/agents-wiki/upstream-merge-guide.md`
- `docs/agents-wiki/change-map.md`
- The latest `docs/agents-wiki/upstream-master-audit-*.md` file for the upstream range being reviewed

## Decision Rules

- Treat retained areas as directions, not a closed allowlist. New local or retained-provider behavior remains in scope even when it postdates the baseline or lacks a fork feature flag.
- Keep local UI, state, persistence, and behavior from mixed commits while removing only incompatible service, account, sync, telemetry, tracking, MCP/skills, or native-platform plumbing.
- Reject behavior only when its core requires a removed Warp service or unsupported local platform and cannot run through a retained local or provider boundary. Size, conflicts, missing prerequisites, or absent rollout flags are not rejection reasons.

## ACP Agent Ownership

- The ACP client is the sole owner of agent execution, sessions, and provider semantics in this fork. AgentView and terminal code are host/UI layers around ACP, not a second Warp Agent backend.
- Treat every upstream Warp Agent change as a migration candidate, not as automatically retained or automatically deleted. Inspect its runtime owner, call graph, persistence, and service dependencies.
- Accept or adapt ACP protocol/client behavior, local AgentView/terminal behavior, local context, and provider-neutral settings. Route adapted behavior through the current ACP path.
- Reject old Warp Agent SDK/service APIs, cloud-agent orchestration, account or billing semantics, and legacy settings that do not control a live ACP or local path. A setting name or surviving field is not proof that the setting belongs in the fork.
- For example, `memory_enabled` only changes the legacy rules-pane state; the ACP request path independently adds project rules found by `ProjectContextModel`. Do not restore it as an ACP memory toggle unless the setting is explicitly wired to an ACP context path.

## Source-First Workflow

1. Inspect the upstream commit, its feature context, and any dependent upstream commits before applying it.

```bash
git show --stat --summary <commit>
git show --name-status <commit>
git diff <commit>^ <commit> -- <path>
git log --oneline --reverse <base>..<commit>
```

2. Classify every touched path and identify required upstream prerequisites.
3. Materialize the authoritative upstream implementation. Use `git cherry-pick --no-commit <commit>` for a fully retained commit; for a mixed commit, apply exact retained paths with `git diff <commit>^ <commit> -- <paths> | git apply --3way`, or copy exact upstream files and hunks.
4. Resolve conflicts on the applied source. Preserve upstream core structure, algorithms, tests, assets, and behavior; limit handwritten changes to necessary fork integration or provider glue.
5. Compare the result with the upstream patch. Record every omitted path or meaningful hunk and its concrete reason in `docs/agents-wiki/`. Never reconstruct upstream core behavior from descriptions, screenshots, observed behavior, or memory.
6. Run focused verification, then the required build and cleanup below.

## Workspace Migrations

Treat Rust edition, minimum-toolchain, resolver, formatter, build-profile, and similar workspace migrations independently from product triage:

- Inspect manifests, lockfiles, toolchain and formatter/lint configuration, Cargo configuration, build scripts, and affected Rust sources.
- Apply the exact upstream migration to every retained crate and build path, even when the same commit contains rejected product paths.
- Adopt the upstream edition when supported; do not rewrite retained upstream code into an older Rust dialect.
- Do not mark the commit complete until every retained path is migrated or a concrete compiler/toolchain blocker is recorded.

## Verification

Run focused tests and checks first. Every independently ported feature must pass the package build before moving to the next feature; clean immediately after a successful build.

```bash
cargo check -p warp --all-targets --message-format short
cargo check --workspace --all-targets --message-format short
cargo fmt -- --check
cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'
cargo build -p warp --all-targets --message-format short
cargo clean
```

For workspace migrations, replace the final package build with:

```bash
cargo build --workspace --all-targets --message-format short
cargo clean
```

Record the build and cleanup results in `docs/agents-wiki/`.

Before finishing major upstream merge work, scan for restored deleted surfaces:

```bash
rg -n "access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment" app crates script Cargo.toml
rg -n "mcp.*capab|capab.*mcp|mcp_server|mcpServers|bundled skills|channel-gated-skills|ReadSkill|InvokeSkill" app crates
rg -n "target_os = \"linux\"|target_os = \"windows\"|cfg\\(windows\\)|WSL|MSYS2|ConPTY|Wayland|X11|winreg|x11rb" app crates Cargo.toml
```

Allowed hits must be documentation, tests, retained remote-terminal behavior, retained local-object schema names, or retained ACP protocol code.
