pub mod app;
pub mod mac;

pub mod headless;

pub mod current {
    pub use super::mac::*;
}

pub use warpui_core::platform::*;

pub use app::AppBuilder;

/// Returns whether the current device is a mobile device with touch input.
pub fn is_mobile_device() -> bool {
    false
}

/// A trait for accessing internal per-platform concrete implementations
/// through a wrapper type.
#[allow(dead_code)]
trait AsInnerMut<Inner: ?Sized> {
    fn as_inner_mut(&mut self) -> &mut Inner;
}
