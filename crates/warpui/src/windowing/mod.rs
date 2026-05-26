pub use warpui_core::windowing::*;

#[cfg(any(test, feature = "integration_tests"))]
pub const MIN_WINDOW_WIDTH: f32 = 124.;
#[cfg(not(any(test, feature = "integration_tests")))]
pub const MIN_WINDOW_WIDTH: f32 = 480.;

#[cfg(any(test, feature = "integration_tests"))]
pub const MIN_WINDOW_HEIGHT: f32 = 34.;
#[cfg(not(any(test, feature = "integration_tests")))]
pub const MIN_WINDOW_HEIGHT: f32 = 192.;
