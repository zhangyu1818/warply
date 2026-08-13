use super::{
    SettingsSection,
    settings_page::{
        MatchData, PageType, SettingsPageEvent, SettingsPageMeta, SettingsPageViewHandle,
        SettingsWidget,
    },
};
use crate::{
    appearance::Appearance, channel::ChannelState, themes::theme::ColorScheme,
    workspace::WorkspaceAction,
};
use warpui::{
    AppContext, Entity, View, ViewContext, ViewHandle,
    assets::asset_cache::AssetSource,
    elements::{
        Align, CacheOption, ConstrainedBox, Container, CrossAxisAlignment, Element, Flex, Image,
        MainAxisAlignment, MouseStateHandle, ParentElement, Wrap,
    },
    ui_components::components::UiComponent,
};

pub struct AboutPageView {
    page: PageType<Self>,
}

impl AboutPageView {
    pub fn new(_ctx: &mut ViewContext<AboutPageView>) -> Self {
        AboutPageView {
            page: PageType::new_monolith(AboutPageWidget::default(), None, false),
        }
    }
}

impl Entity for AboutPageView {
    type Event = SettingsPageEvent;
}

impl View for AboutPageView {
    fn ui_name() -> &'static str {
        "AboutPage"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        self.page.render(self, app)
    }
}

#[derive(Default)]
struct AboutPageWidget {
    copy_version_button_mouse_state: MouseStateHandle,
}

impl SettingsWidget for AboutPageWidget {
    type View = AboutPageView;

    fn search_terms(&self) -> &str {
        "about warply version"
    }

    fn render(
        &self,
        _view: &AboutPageView,
        appearance: &Appearance,
        _app: &AppContext,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        let ui_builder = appearance.ui_builder();

        let image_path = if theme.inferred_color_scheme() == ColorScheme::LightOnDark {
            "bundled/svg/warp-logo-with-light-title.svg"
        } else {
            "bundled/svg/warp-logo-with-dark-title.svg"
        };

        let version = app_display_version();

        let version_text = ui_builder
            .span(version.clone())
            .with_soft_wrap()
            .build()
            .with_margin_top(16.)
            .finish();

        let version_to_copy = version.clone();
        let copy_version_icon = appearance
            .ui_builder()
            .copy_button(16., self.copy_version_button_mouse_state.clone())
            .build()
            .on_click(move |ctx, _, _| {
                ctx.dispatch_typed_action(WorkspaceAction::CopyVersion(version_to_copy.clone()));
            })
            .finish();

        let version_row = Wrap::row()
            .with_main_axis_alignment(MainAxisAlignment::Center)
            .with_children([
                version_text,
                Container::new(copy_version_icon)
                    .with_margin_top(16.)
                    .with_padding_left(6.)
                    .finish(),
            ]);

        Align::new(
            Flex::column()
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .with_child(
                    ConstrainedBox::new(
                        Image::new(
                            AssetSource::Bundled { path: image_path },
                            CacheOption::BySize,
                        )
                        .finish(),
                    )
                    .with_max_height(100.)
                    .with_max_width(350.)
                    .finish(),
                )
                .with_child(version_row.finish())
                .with_child(
                    ui_builder
                        .span("Copyright 2026 Warp Open Source")
                        .build()
                        .with_margin_top(16.)
                        .finish(),
                )
                .finish(),
        )
        .finish()
    }
}

fn app_display_version() -> String {
    let release_tag = bundle_info_value("WarplyVersion")
        .or_else(|| ChannelState::app_version().map(ToString::to_string));

    display_version_from_release_tag(release_tag.as_deref())
}

fn display_version_from_release_tag(release_tag: Option<&str>) -> String {
    release_tag
        .map(ToString::to_string)
        .unwrap_or_else(|| "v#.##.###".to_string())
}

#[cfg(all(target_os = "macos", not(test)))]
#[allow(deprecated)]
fn bundle_info_value(key: &str) -> Option<String> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};
    use warpui::platform::mac::{make_nsstring, utils::nsstring_as_str};

    unsafe {
        let bundle: id = msg_send![class!(NSBundle), mainBundle];
        if bundle == nil {
            return None;
        }

        let key = make_nsstring(key);
        let value: id = msg_send![bundle, objectForInfoDictionaryKey: key];
        if value == nil {
            return None;
        }

        nsstring_as_str(value).ok().map(ToString::to_string)
    }
}

#[cfg(any(not(target_os = "macos"), test))]
fn bundle_info_value(_key: &str) -> Option<String> {
    None
}

impl SettingsPageMeta for AboutPageView {
    fn section() -> SettingsSection {
        SettingsSection::About
    }

    fn should_render(&self, _ctx: &AppContext) -> bool {
        true
    }

    fn update_filter(&mut self, query: &str, ctx: &mut ViewContext<Self>) -> MatchData {
        self.page.update_filter(query, ctx)
    }

    fn scroll_to_widget(&mut self, widget_id: &'static str) {
        self.page.scroll_to_widget(widget_id)
    }

    fn clear_highlighted_widget(&mut self) {
        self.page.clear_highlighted_widget();
    }
}

impl From<ViewHandle<AboutPageView>> for SettingsPageViewHandle {
    fn from(view_handle: ViewHandle<AboutPageView>) -> Self {
        SettingsPageViewHandle::About(view_handle)
    }
}
