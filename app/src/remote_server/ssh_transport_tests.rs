use super::*;

fn static_identity_context() -> Arc<RemoteServerIdentityContext> {
    Arc::new(RemoteServerIdentityContext::new(
        || "user id/with spaces".to_string(),
        String::new(),
        String::new(),
    ))
}

#[test]
fn remote_proxy_command_quotes_identity_key() {
    let transport = SshTransport::new(
        PathBuf::from("/tmp/control-master.sock"),
        static_identity_context(),
    );

    let command = transport.remote_proxy_command();

    assert!(command.contains("remote-server-proxy --identity-key"));
    assert!(command.contains("'user id/with spaces'"));
}
