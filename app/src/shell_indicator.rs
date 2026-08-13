use std::convert::TryFrom;

use crate::terminal::{ShellLaunchData, shell::ShellType};
use crate::ui_components::icons::Icon;

#[derive(Clone, Copy)]
pub enum ShellIndicatorType {
    Powershell,
    Linux,
}

impl ShellIndicatorType {
    pub fn to_icon(self) -> Icon {
        match self {
            Self::Powershell => Icon::Powershell,
            Self::Linux => Icon::Linux,
        }
    }
}

impl TryFrom<&ShellLaunchData> for ShellIndicatorType {
    type Error = ();

    fn try_from(shell_launch_data: &ShellLaunchData) -> Result<Self, Self::Error> {
        match shell_launch_data {
            ShellLaunchData::Executable { shell_type, .. } => match shell_type {
                ShellType::PowerShell => Ok(Self::Powershell),
                _ => Err(()),
            },
            ShellLaunchData::DockerSandbox { .. } => Ok(Self::Linux),
        }
    }
}
