//! Renders the user query portion of the AI block, if there is one.
//!
//! Queries are not rendered in blocks corresponding to requested command or requested action responses.

use chrono::{DateTime, Local};
use pathfinder_geometry::vector::vec2f;
use warp_core::ui::color::Opacity;
use warp_core::ui::theme::color::internal_colors;
use warpui::{
    AppContext, Element, SingletonEntity,
    elements::{
        Border, ChildAnchor, Container, CornerRadius, DropShadow, Flex, MainAxisAlignment,
        MainAxisSize, MouseStateHandle, ParentAnchor, ParentElement, Radius, Shrinkable, Wrap,
    },
    fonts::{Properties, Style, Weight},
    ui_components::{
        chip::Chip,
        components::{Coords, UiComponent, UiComponentStyles},
    },
};

use super::common::{FindContext, render_query_text};
use crate::ai::blocklist::AttachmentType;
use crate::ai::blocklist::block::view_impl::common::UserQueryProps;
use crate::appearance::Appearance;
use crate::util::time_format::format_message_timestamp;
use crate::{
    ai::blocklist::block::{DetectedLinksState, SecretRedactionState},
    ui_components::{blended_colors, icons::Icon},
};

/// Width of the accent ring drawn around the user query while agent-view transcript
/// navigation targets this query.
const NAVIGATION_RING_BORDER_WIDTH: f32 = 2.;
/// Blur radius of the accent halo behind the navigation ring.
const NAVIGATION_HALO_BLUR_RADIUS: f32 = 6.;
/// How far the accent halo extends beyond the query.
const NAVIGATION_HALO_SPREAD_RADIUS: f32 = 1.5;
/// Opacity (in percent) of the accent halo.
const NAVIGATION_HALO_OPACITY: Opacity = 60;

/// Data required to render the AI block query component.
#[derive(Copy, Clone, Debug)]
pub(super) struct Props<'a> {
    pub(super) query_sent_at: Option<DateTime<Local>>,
    pub(super) query_timestamp_tooltip_handle: &'a MouseStateHandle,
    pub(super) query_and_index: Option<(&'a str, usize)>,
    pub(super) query_prefix_highlight_len: Option<usize>,
    pub(super) detected_links_state: &'a DetectedLinksState,
    pub(super) secret_redaction_state: &'a SecretRedactionState,
    pub(super) is_selecting_text: bool,
    pub(super) is_ai_input_enabled: bool,
    pub(super) attachments: &'a [(AttachmentType, String)],
    pub(super) find_context: Option<FindContext<'a>>,
    pub(super) is_agent_transcript_navigation_target: bool,
}

pub(super) fn maybe_render(props: Props, app: &AppContext) -> Option<Box<dyn Element>> {
    props.query_and_index.map(|(query, input_index)| {
        render_query(
            query,
            props.query_sent_at,
            props.query_timestamp_tooltip_handle.clone(),
            props.detected_links_state,
            props.secret_redaction_state,
            input_index,
            props.query_prefix_highlight_len,
            props.is_selecting_text,
            props.is_ai_input_enabled,
            props.attachments,
            props.find_context,
            props.is_agent_transcript_navigation_target,
            app,
        )
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_query(
    query: &str,
    query_sent_at: Option<DateTime<Local>>,
    query_timestamp_tooltip_handle: MouseStateHandle,
    detected_links_state: &DetectedLinksState,
    secret_redaction_state: &SecretRedactionState,
    input_index: usize,
    query_prefix_highlight_len: Option<usize>,
    is_selecting: bool,
    is_ai_input_enabled: bool,
    attachments: &[(AttachmentType, String)],
    find_context: Option<FindContext>,
    is_agent_transcript_navigation_target: bool,
    app: &AppContext,
) -> Box<dyn Element> {
    let properties = Properties {
        style: Style::Normal,
        weight: Weight::Bold,
    };
    // The query already includes the /plan prefix when in plan mode via display_user_query()
    let text_element = render_query_text(
        UserQueryProps {
            text: query.to_owned(),
            query_prefix_highlight_len,
            detected_links_state,
            secret_redaction_state,
            input_index,
            is_selecting,
            is_ai_input_enabled,
            find_context,
            font_properties: &properties,
        },
        app,
    );

    let mut query = Flex::column().with_child(text_element.finish());

    let appearance = Appearance::as_ref(app);
    query = query.with_child(render_attachments(attachments, appearance));

    let mut query_container = Container::new(query.finish());
    if is_agent_transcript_navigation_target {
        let accent = appearance.theme().accent();
        query_container = query_container
            .with_foreground_border(
                Border::all(NAVIGATION_RING_BORDER_WIDTH).with_border_fill(accent),
            )
            .with_drop_shadow(DropShadow {
                color: accent.with_opacity(NAVIGATION_HALO_OPACITY).into_solid(),
                offset: vec2f(0., 0.),
                blur_radius: NAVIGATION_HALO_BLUR_RADIUS,
                spread_radius: NAVIGATION_HALO_SPREAD_RADIUS,
            })
            .with_corner_radius(CornerRadius::with_all(Radius::Pixels(5.)));
    }

    let query_element = if let Some(timestamp) = query_sent_at {
        appearance.ui_builder().overlay_tool_tip_on_element(
            format!("Message sent {}", format_message_timestamp(&timestamp)),
            query_timestamp_tooltip_handle,
            query_container.finish(),
            ParentAnchor::TopLeft,
            ChildAnchor::BottomLeft,
            vec2f(0., -8.),
        )
    } else {
        query_container.finish()
    };

    Flex::row()
        .with_cross_axis_alignment(warpui::elements::CrossAxisAlignment::Start)
        .with_child(Shrinkable::new(1., query_element).finish())
        .finish()
}

fn render_attachments(
    attachments: &[(AttachmentType, String)],
    appearance: &Appearance,
) -> Box<dyn Element> {
    let chips = attachments.iter().map(|(attachment_type, file_name)| {
        let icon = match attachment_type {
            AttachmentType::Image => Icon::Image,
            AttachmentType::File => Icon::File,
        };

        Chip::new(
            file_name.clone(),
            UiComponentStyles {
                margin: Some(Coords {
                    top: 0.,
                    bottom: 0.,
                    left: 0.,
                    right: 6.,
                }),
                font_family_id: Some(appearance.ui_font_family()),
                font_size: Some(appearance.monospace_font_size()),
                font_color: Some(blended_colors::text_sub(
                    appearance.theme(),
                    appearance.theme().background(),
                )),
                border_width: Some(1.),
                border_color: Some(internal_colors::neutral_4(appearance.theme()).into()),
                border_radius: Some(CornerRadius::with_all(Radius::Pixels(5.))),
                ..Default::default()
            },
        )
        .with_icon(icon.to_warpui_icon(
            blended_colors::text_sub(appearance.theme(), appearance.theme().background()).into(),
        ))
        .build()
        .finish()
    });

    if attachments.is_empty() {
        Flex::row().finish()
    } else {
        let wrapping_section = Wrap::row()
            .with_run_spacing(8.)
            .with_main_axis_alignment(MainAxisAlignment::Start)
            .with_main_axis_size(MainAxisSize::Min)
            .with_children(chips)
            .finish();
        Container::new(wrapping_section)
            .with_padding_top(7.)
            .finish()
    }
}
