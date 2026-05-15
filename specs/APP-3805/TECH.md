# TECH.md - Remote Server Version Skew

Linear: [APP-3805](https://linear.app/warpdotdev/issue/APP-3805/client-server-version-skew)

## Fork Contract

Remote-server version skew handling is retained because SSH remote server is part of Warpify and the terminal product. Hosted artifact installation is not retained. This fork must not download remote-server artifacts from `warp-server`, `/download/cli`, release CDN URLs, GraphQL, token-backed APIs, or old Oz/Agent SDK paths.

## Problem

The local client and the remote-server binary communicate through a protobuf protocol. If the client auto-updates but the remote host still has an older binary, the proxy can connect to a server whose protocol or behavior no longer matches the client.

The fork needs deterministic version matching without restoring the old Warp service download path.

## Retained Behavior

### Channel-Scoped Binary Paths

`crates/remote_server/src/setup.rs::remote_server_binary()` keeps the current path rule:

- `Channel::Local` and `Channel::Oss` use the bare `{remote_server_dir}/{binary_name}` path.
- `Stable`, `Preview`, `Dev`, and `Integration` use `{remote_server_dir}/{binary_name}-{version}`.

The version is `ChannelState::app_version()` when present, otherwise `CARGO_PKG_VERSION`. This fallback only keeps the path deterministic; it must not imply that a hosted artifact exists.

### Local/Oss Development Slot

`Channel::Local` and `Channel::Oss` keep the unversioned slot so local deployment tools such as `script/deploy_remote_server` can place a binary that the client can find without a network install.

### Handshake Version Check

`crates/remote_server/src/manager.rs::version_is_compatible()` remains the defense-in-depth check after `InitializeResponse`:

- Matching non-empty client/server versions are compatible.
- Both sides unknown or empty are compatible for local development.
- Mismatched or one-sided versions are incompatible.

On mismatch, the client removes the expected remote binary path before marking the session disconnected. This prevents reconnect loops where the same bad file passes the existence check again.

### Install Script

`crates/remote_server/src/install_remote_server.sh` currently reports that remote-server auto-install is unavailable in this build. That is the correct fork behavior until a local packaging/deployment flow is added.

Any future install script must install from local fork-controlled artifacts. It must not call Warp hosted download endpoints or silently fetch release artifacts from external Warp infrastructure.

## Non-Goals

- Hosted download/version pinning through `warp-server`.
- Release CDN fallback.
- Oz CLI artifact naming.
- Token, login, GraphQL, or Warp service dependencies.
- Cleanup of old binaries as a migration shim.
- Local Linux or Windows app packaging support.

## Validation

- Unit tests for `remote_server_binary()` cover versioned paths for release channels and bare paths for `Local`/`Oss`.
- Unit tests for `version_is_compatible()` cover matching versions, both-empty local development, mismatched versions, and one-sided versions.
- Manual SSH validation with a pre-deployed matching remote-server binary should connect successfully.
- Manual skew validation with a mismatched pre-deployed binary should disconnect and remove the bad expected binary path.
- Missing binary validation should not attempt a Warp service, CDN, or GraphQL download.
