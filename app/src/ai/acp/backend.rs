use command::blocking::Command;

use crate::settings::AcpAgentBackend;

pub fn adapter_is_available(backend: AcpAgentBackend) -> bool {
    Command::new(backend.adapter_command())
        .arg("--version")
        .output()
        .is_ok()
}
