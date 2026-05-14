//! Module containing the definition of the current windowing [`System`] the application is
//! rendering to.

use std::fmt::{Display, Formatter};

use raw_window_handle::RawDisplayHandle;

#[derive(Copy, Clone, Debug)]
pub enum System {
    AppKit,
}

impl System {
    pub fn allows_programmatic_window_activation(&self) -> bool {
        true
    }
}

impl Display for System {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            System::AppKit => write!(f, "AppKit"),
        }
    }
}

impl TryFrom<RawDisplayHandle> for System {
    type Error = CreateWindowingSystemError;

    fn try_from(raw_display_handle: RawDisplayHandle) -> Result<Self, Self::Error> {
        let display = match raw_display_handle {
            RawDisplayHandle::AppKit(_) => System::AppKit,
            _ => {
                return Err(Self::Error::UnrecognizedDisplayHandle);
            }
        };

        Ok(display)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CreateWindowingSystemError {
    #[error("Unrecognized DisplayHandle")]
    UnrecognizedDisplayHandle,
}
