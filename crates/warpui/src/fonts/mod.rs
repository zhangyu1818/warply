pub mod font_kit;

#[cfg(test)]
#[path = "text_layout_test.rs"]
mod text_layout_tests;

pub use warpui_core::fonts::*;

#[cfg(test)]
pub(crate) use text_layout_tests::{collect_glyph_indices, init_fonts};

#[cfg(test)]
pub(crate) use text_layout_tests::collect_line_caret_position_starts;
