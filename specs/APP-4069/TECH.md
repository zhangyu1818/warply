# APP-4069 — SSH Initialization UX

Linear: [APP-4069 — Initialization UX](https://linear.app/warpdotdev/issue/APP-4069/initialization-ux)

## Fork Contract

SSH remote server and Warpify are retained terminal functionality. The SSH initialization UI must keep remote and nested sessions in full Warp mode when possible, while never restoring Warp service downloads, old Oz CLI naming, rollout compatibility branches, telemetry, account tokens, or cloud-agent setup.

## Required Behavior

When a user starts an SSH session, Warp may need to decide whether to use a remote-server binary or continue with the ControlMaster-backed Warpify terminal path. The initialization flow should:

1. Hold the shell bootstrap until the remote-server binary check has produced a decision.
2. If a valid remote-server binary already exists, flush the bootstrap, launch `remote-server-proxy` over the existing SSH ControlMaster channel, and complete setup only after the protocol handshake succeeds.
3. If the binary is missing, unsupported, or cannot be used, flush the bootstrap and continue the SSH terminal through the retained ControlMaster-backed Warpify path.
4. If this fork later provides a local deployment path for the remote-server binary, the choice UI may offer “Use remote server” and “Continue without remote server”. It must not call Warp hosted download endpoints.
5. Any check, local-deployment, launch, or handshake error must leave the SSH terminal usable.

Continuing without the remote-server extension is retained terminal behavior, not a legacy cloud-agent compatibility path.

## Current Code Map

- `app/src/terminal/writeable_pty/remote_server_controller.rs` owns the per-pane SSH init state machine.
- `app/src/terminal/writeable_pty/pty_controller.rs` writes the shell bootstrap when the controller releases it.
- `app/src/terminal/model_events.rs` carries SSH init, bootstrap, remote-server connection, and skipped-extension events.
- `app/src/terminal/view/ssh_remote_server_choice_view.rs` renders the retained SSH extension choice UI.
- `crates/remote_server/src/manager.rs` checks binary state, records setup state, and manages proxy sessions.
- `app/src/remote_server/ssh_transport.rs` runs remote checks and launches `remote-server-proxy` through SSH.
- `app/src/terminal/warpify/` and `app/src/terminal/ssh/` keep the Warpify SSH/subshell terminal path.

## Merge Rules

- Do not reintroduce a `SshRemoteServer` rollout flag or disabled branch.
- Do not call `/download/cli`, release CDN URLs, `SERVER_ROOT_URL`, GraphQL, token-backed APIs, or old Warp service endpoints.
- Do not restore Oz CLI names, Agent SDK harnesses, cloud-agent setup, orchestration, or telemetry around SSH initialization.
- Do not remove Warpify, tmux checks, ControlMaster bootstrap, or `remote-server-proxy` as part of AI cleanup.
- Remote Linux/macOS host checks are allowed only for SSH remote-server setup. They are not local Linux app support.

## Validation

- Existing remote-server binary: SSH setup launches `remote-server-proxy` and completes only after handshake.
- Missing binary with no local deployment path: SSH terminal remains usable through Warpify/ControlMaster and no hosted endpoint is contacted.
- Unsupported remote host: no install prompt loop; SSH terminal remains usable.
- User chooses to continue without remote server: bootstrap is flushed and the terminal initializes normally.
- Error during check, deployment, launch, or handshake: terminal is usable and diagnostics remain local.
