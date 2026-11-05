# Upstream Master Audit — 2026-11-05

Incremental audit for `1eca3e4ee..upstream/master` (`3fe2ac676`), 6 commits. The previous audit is `upstream-master-audit-2026-11-04.md`.

## Summary

Zero ports. Two commits are not applicable (upstream release-workflow changelog-agent auth over files the fork owns as rewrites or does not carry), and four are rejected (Factory cloud-run skill-reference resolution and its run reporting, a bundled Factory validation-script limit bump, and native staging IAP routing). Every fork-present hunk was verified anchorless: the fork has no `app/src/ai/agent_sdk/`, no `crates/ai/src/skills/`, no `app/src/server/`, no `app/src/terminal/shared_session/`, no `crates/warp_server_client/`, no `crates/http_client` IAP module, no `resources/bundled/skills/`, no `app/src/tracing/`, and repo-wide search finds zero `IapState`/`IapManager`/`IapRoutingSettings`/`iap_config` symbols.

## Commit-by-Commit

### `eba011405` Authenticate changelog metadata across repositories (#16721) — not applicable

Mints two short-lived Octo STS `release-changelog` tokens (internal + public repo scopes) in the upstream `create_release.yml` changelog job and threads them into `.agents/skills/changelog-draft/scripts/fetch_prs.py`. The fork's `.github/workflows/create_release.yml` is the fork-owned Warply Sparkle release workflow with no Oz changelog agent, no Octo STS minting, and no changelog job, and the fork's `.agents/skills/` directory (diverged at baseline) contains no `changelog-draft` skill. No fork file exists to apply either hunk onto.

### `2d5398a4f` Update auth configuration for the changelog generation agent action (#16727) — not applicable

Single hunk in the upstream `create_release.yml` switching the Oz changelog-draft agent to a dedicated staging API key and `oz_channel: dev`. Fork-owned release workflow rewrite; the Oz agent invocation it edits does not exist here.

### `d6b50252c` [REMOTE-3119] Resolve skill references in Factory cloud runs (#16452) — rejected

Client-side resolution of skill reference files in Factory definitions during cloud task runs, behind a new `FactorySkillReferences` `warp_features` flag. Adds `crates/ai/src/skills/references.rs` (reference-file parser plus 26 shared cases from warp-server) and `app/src/ai/agent_sdk/driver/skill_references.rs` (source checkout, name/path/glob resolution, generated per-level skill folders, sandbox-escape rejection), and wires it through agent_sdk driver/environment/environment_checkout/standby/setup_observability, `crates/warp_cli/src/environment_checkout.rs`, `app/src/tracing/child_process_tests.rs`, and manifests (`yaml-rust` workspace entry; `globset`/`warp_cli`/`yaml-rust` for `crates/ai`; `globset` for the app).

This is app-managed skill resolution for cloud agent runs — a doubly removed surface (bundled/scanned skills and Factory cloud runs). The fork has no `app/src/ai/agent_sdk/`, no `crates/ai/src/skills/`, no `crates/warp_cli` `environment_checkout` module, and no `app/src/tracing/`; the manifest additions exist only for this machinery, and the new feature flag would gate dead code. ACP agents own their own skills; nothing is separable into a retained local path.

### `abeaf2a93` [REMOTE-3119] Report resolved skill references on the run (#16453) — rejected

Second half of the REMOTE-3119 stack: reports resolved skill references through `app/src/ai/agent_sdk/driver/skill_references.rs` and `setup_observability.rs` onto `app/src/server/server_api/ai.rs`, and updates the bundled factory-files skill plus its validation scripts. Every touched path is verified absent in this fork (`agent_sdk`, `server/server_api`, `resources/bundled/skills/`, `script/test_factory_files_skill.py`). No anchors, nothing separable.

### `974ad5115` Raise bundled Factory validation file limit to 2,048 (#16739) — rejected

Touches only `resources/bundled/skills/factory-files/scripts/validate_factory_files.py` and `script/test_factory_files_skill.py`. Bundled skills are a removed surface and neither file exists here.

### `3fe2ac676` Add native staging IAP routing opt-in (#16535) — rejected

Warp-server staging Identity-Aware-Proxy routing: a default-off "Use IAP for staging requests" Account-settings switch (`app/src/settings/iap.rs`), `X-Warp-Enable-IAP: 1` request-builder/WebSocket plumbing through `crates/http_client/src/iap.rs` and `crates/warp_server_client`'s base client, server-API object.rs wiring, shared-session sharer/viewer IAP cleanup, `app/src/lib.rs` subscription to the removed `IapState` singleton, and an `install_channel_config` pin. The fork has no `crates/warp_server_client`, no `app/src/server/`, no `app/src/terminal/shared_session/`, no http_client IAP module, no `script/install_channel_config`, and no account settings page; repo-wide search finds no `IapState`/`IapManager`/`IapRoutingSettings`/`ChannelState::iap_config` symbols to hang any hunk on. The fork makes no Warp server requests at all, so the feature has no request path and no settings surface here — server-access infrastructure in the sense of the reject table, with no reducible retained local utility.

## Verification

No code was ported in this cycle, so the fork tree is byte-identical to `v2026.11.04` (`e2229223c`), whose cycle passed `cargo check -p warp --all-targets`, `cargo check --workspace --all-targets`, `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` (156 passed), and `cargo build -p warp --all-targets` followed by `cargo clean`. That build state carries over unchanged.

- `cargo fmt -- --check` — passed.
- Restored-surface scans re-run over the identical tree: cloud/auth/billing/telemetry pattern scan returns only the pre-existing allowed hits (tests, doc comments, weak-handle `upgrade` wording, retained local-model code), MCP/skills pattern scan returns zero hits, and the platform scan returns exactly the six known retained occurrences (`ForwardX11=no` SSH config ×2, the retained ConPTY explanatory comment in `zsh_body.sh`, and `#[cfg(windows)]`-gated tests in `warp_util`) — unchanged from the 2026-11-04 audit baseline.

## Tag

`v2026.11.05` on `main` (zero-port audit commit; no merge branch needed, matching the `v2026.09.07` zero-port precedent).
