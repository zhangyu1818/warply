use warpui::ViewContext;

use super::view::Workspace;
use crate::pane_group::{AnyPaneContent, WelcomePane};

/// Create a static "home page" pane.
pub fn create_home_pane(ctx: &mut ViewContext<Workspace>) -> Box<dyn AnyPaneContent> {
    Box::new(WelcomePane::new(None, ctx))
}
