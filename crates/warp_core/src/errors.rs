mod anyhow;
mod registration;
mod reqwest;
#[cfg(not(target_family = "wasm"))]
mod tokio;
#[cfg(not(target_family = "wasm"))]
mod websocket;

// Re-export for macro use.
#[doc(hidden)]
pub use inventory::submit;

pub use self::anyhow::AnyhowErrorExt;
pub use registration::{ErrorRegistration, RegisteredError};

pub use registration::register_error;

pub trait ErrorExt: RegisteredError + std::error::Error {
    /// Returns whether or not an error is something that is actionable by our
    /// engineering team.
    fn is_actionable(&self) -> bool;
}
