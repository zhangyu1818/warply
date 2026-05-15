# Remote Server SSH: Technical Spec

## Current Fork Contract

SSH remote server is retained terminal functionality. It is not an AI cloud surface and must not be removed as part of agent cleanup. At the same time, this fork must not restore Warp-hosted artifact downloads, Oz CLI installation, old Agent SDK harnesses, feature-flag rollout compatibility, or token-backed service calls.

## Relevant Code

- `crates/remote_server/src/setup.rs` defines remote-server setup state, remote platform parsing, channel-scoped install paths, binary checks, and the install script loader.
- `crates/remote_server/src/install_remote_server.sh` currently exits with "remote server auto-install is unavailable in this build".
- `app/src/remote_server/ssh_transport.rs` uses the existing SSH ControlMaster socket to run checks, run local install scripts, and launch `remote-server-proxy`.
- `app/src/terminal/writeable_pty/remote_server_controller.rs` coordinates SSH init, binary checks, user choice, install attempts, and remote-server connection.
- `app/src/terminal/view/ssh_remote_server_choice_view.rs` renders the retained Warpify SSH extension choice UI.
- `app/src/terminal/warpify/settings.rs` stores local SSH extension install prompt behavior.

## Required Behavior

### Binary Check

The SSH transport checks the expected channel-scoped remote-server binary path over the existing ControlMaster socket. This check is local to the SSH session and must not require Warp login, access tokens, GraphQL, or a hosted API.

### Remote Platform Detection

`uname -sm` parsing may keep Linux and macOS remote-host variants because remote hosts are part of SSH support. This is not permission to restore native Linux or Windows app packaging, local PTY, secure storage, or windowing code.

### Installation

The current fork does not provide hosted auto-install. `install_remote_server.sh` must not call `/download/cli`, `releases.warp.dev`, `SERVER_ROOT_URL`, or other Warp service endpoints.

If remote-server installation is reintroduced later, the source must be a local packaging/deployment mechanism owned by this fork. The client must not silently fetch from Warp infrastructure.

### Connection

When a valid remote-server binary exists, `SshTransport` launches:

```text
{remote_server_binary} remote-server-proxy --identity-key {identity_key}
```

The returned child process stdin/stdout become the protobuf channel used by `RemoteServerClient`.

### Missing Or Unsupported Extension

If the binary is missing, installation is unavailable, or the remote host is unsupported, Warp should continue the SSH terminal session without remote-server extension features. This preserves terminal usability and Warpify behavior; it must not be described or implemented as a legacy cloud-agent compatibility path.

## Rules For Future Changes

- Do not add a `RemoteServerSSH` rollout flag just to preserve an old path.
- Do not reintroduce Oz CLI names or old Agent SDK driver references.
- Do not add Warp service URLs, token auth, GraphQL, or release CDN downloads.
- Do not treat remote Linux/macOS support as local platform support.
- Do not delete Warpify, SSH ControlMaster bootstrap, or remote-server proxy support as AI cleanup.
- Keep diagnostics local. Do not add telemetry events for setup duration, unsupported hosts, install failures, or reconnects.

## Validation

- `rg -n "download/cli|releases\\.warp|SERVER_ROOT_URL|Oz CLI|RemoteServerSSH|Agent SDK|cloud-agent" specs/alokedesai/APP-3797 docs/agents-wiki -g'*.md'` should return only intentional guardrail references outside this spec.
- Existing binary path: verify an SSH session connects to `remote-server-proxy` and receives the remote-server initialize response.
- Missing binary path: verify no Warp service download is attempted and the SSH terminal remains usable.
- Unsupported host path: verify setup is skipped without surfacing a false install prompt.
- Remote Linux/macOS checks: verify they are limited to remote SSH setup and do not restore local Linux/Windows app behavior.
