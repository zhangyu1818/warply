use crate::themes::theme::WarpTheme;
use warpui::elements::{Empty, MouseStateHandle};
use warpui::platform::FullscreenState;
use warpui::{AppContext, Element, WindowId};

pub fn traffic_light_data(ctx: &AppContext, window_id: WindowId) -> Option<TrafficLightData> {
    if ctx
        .windows()
        .platform_window(window_id)
        .is_some_and(|window| window.uses_native_window_decorations())
    {
        return None;
    }

    Some(TrafficLightData {
        width: 64.,
        side: TrafficLightSide::Left,
        scales_with_zoom: false,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrafficLightSide {
    Left,
    Right,
}

#[derive(Default)]
#[cfg_attr(target_family = "wasm", allow(dead_code))]
pub struct TrafficLightMouseStates {
    pub minimize_window_button: MouseStateHandle,
    pub maximize_window_button: MouseStateHandle,
    pub close_window_button: MouseStateHandle,
}

impl TrafficLightMouseStates {
    pub fn are_traffic_lights_hovered(&self) -> bool {
        [
            &self.minimize_window_button,
            &self.maximize_window_button,
            &self.close_window_button,
        ]
        .into_iter()
        .any(|state| state.lock().is_ok_and(|state| state.is_hovered()))
    }
}

#[derive(Clone, Debug)]
pub struct TrafficLightData {
    width: f32,
    pub side: TrafficLightSide,
    scales_with_zoom: bool,
}

impl TrafficLightData {
    pub fn width(&self, zoom_factor: f32) -> f32 {
        if self.scales_with_zoom {
            self.width
        } else {
            self.width / zoom_factor
        }
    }

    pub fn render(
        &self,
        _fullscreen_state: FullscreenState,
        _mouse_states: &TrafficLightMouseStates,
        _theme: &WarpTheme,
        _app: &AppContext,
    ) -> Box<dyn Element> {
        Empty::new().finish()
    }
}
