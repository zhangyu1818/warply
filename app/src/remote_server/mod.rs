// Re-export everything from the `remote_server` crate so existing
// `crate::remote_server::*` imports in `app` continue to work.
pub use remote_server::*;

pub mod identity_context;
pub mod diff_state_proto;
pub(crate) mod diff_state_tracker;
pub mod server_model;
pub mod ssh_transport;
pub mod unix;

/// Run the `remote-server-proxy` subcommand.
pub fn run_proxy(identity_key: String) -> anyhow::Result<()> {
    unix::proxy::run(&identity_key)
}

/// Run the `remote-server-daemon` subcommand.
pub fn run_daemon(identity_key: String) -> anyhow::Result<()> {
    unix::run_daemon(identity_key)
}
