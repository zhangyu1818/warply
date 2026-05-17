use command::blocking::Command;

use super::registry::AcpAgentLaunch;

pub fn adapter_args(launch: &AcpAgentLaunch, path_env_var: Option<&str>) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(path_env_var) = path_env_var
        .map(str::trim)
        .filter(|path_env_var| !path_env_var.is_empty())
    {
        args.push(format!("PATH={path_env_var}"));
    }
    args.extend(
        launch
            .env
            .iter()
            .map(|(name, value)| format!("{name}={value}")),
    );
    args.extend(launch.command_line.iter().cloned());
    args
}

pub fn adapter_is_available(launch: &AcpAgentLaunch, path_env_var: Option<&str>) -> bool {
    let Some(executable) = launch.command_line.first() else {
        return false;
    };

    let mut command = Command::new("/usr/bin/which");
    if let Some(path_env_var) = path_env_var
        .map(str::trim)
        .filter(|path_env_var| !path_env_var.is_empty())
    {
        command.env("PATH", path_env_var);
    }
    for (name, value) in &launch.env {
        command.env(name, value);
    }
    command
        .arg(executable)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
