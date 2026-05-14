use typed_path::{TypedPath, TypedPathBuf};
use warp_terminal::shell::ShellLaunchData;

pub fn join_paths(paths: &[&str], _shell: Option<&ShellLaunchData>) -> String {
    let base_path = TypedPathBuf::unix();
    paths
        .iter()
        .fold(base_path, |acc, path| acc.join(path))
        .to_string_lossy()
        .into_owned()
}

fn shell_native_absolute_path_internal(
    file_path: &str,
    _shell: Option<&ShellLaunchData>,
    current_working_directory: &str,
) -> TypedPathBuf {
    let expanded_path = shellexpand::tilde(file_path).into_owned();

    let cwd = TypedPathBuf::from_unix(current_working_directory);
    let file_path = TypedPath::unix(&expanded_path);
    cwd.join(file_path).normalize()
}

/// Returns the absolute path of the path in the shell's native format.
///
/// On macOS, this will always be Unix encoded paths.
pub fn shell_native_absolute_path(
    file_path: &str,
    shell: Option<&ShellLaunchData>,
    current_working_directory: Option<&String>,
) -> String {
    let Some(cwd) = current_working_directory else {
        return shellexpand::tilde(file_path).into_owned();
    };
    shell_native_absolute_path_internal(file_path, shell, cwd)
        .to_string_lossy()
        .into_owned()
}

/// Returns the absolute path of the path in the host's native format.
pub fn host_native_absolute_path(
    file_path: &str,
    shell: &Option<ShellLaunchData>,
    current_working_directory: &Option<String>,
) -> String {
    let Some(cwd) = current_working_directory.as_ref() else {
        return shellexpand::tilde(file_path).into_owned();
    };
    let normalized_path = shell_native_absolute_path_internal(file_path, shell.as_ref(), cwd);

    normalized_path.to_string_lossy().into_owned()
}

#[cfg(test)]
#[path = "paths_tests.rs"]
mod tests;
