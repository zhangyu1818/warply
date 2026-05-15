use command::r#async::Command;

/// A wrapper around `path_env_var` that produces correctly-configured commands.
///
/// Callers construct commands through the executor, which transparently sets
/// the PATH environment variable.
#[derive(Clone)]
pub struct CommandBuilder {
    path_env_var: Option<String>,
}

impl CommandBuilder {
    /// Creates a new CommandBuilder with the given PATH environment variable.
    pub fn new(path_env_var: Option<String>) -> Self {
        Self { path_env_var }
    }

    /// Returns the PATH environment variable, if set.
    pub fn path_env_var(&self) -> Option<&str> {
        self.path_env_var.as_deref()
    }

    /// Creates a new Command with PATH already set.
    ///
    /// Use this when you need to run a command. The returned Command has the
    /// same API as `command::r#async::Command`, so callers don't need to change
    /// how they construct commands.
    ///
    pub fn command(&self, program: impl AsRef<std::ffi::OsStr>) -> Command {
        let mut cmd = Command::new(program);
        if let Some(path) = &self.path_env_var {
            cmd.env("PATH", path);
        }
        cmd
    }
}
