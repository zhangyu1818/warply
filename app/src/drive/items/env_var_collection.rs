use itertools::Itertools;
use warpui::{
    AppContext, Element, SingletonEntity,
    elements::{Clipped, Container, Flex, ParentElement},
    fonts::Weight,
    ui_components::components::{UiComponent, UiComponentStyles},
};

use crate::{
    appearance::Appearance,
    cloud_object::{
        CloudObjectMetadata,
        model::actions::{ObjectActionType, ObjectActions},
    },
    drive::{CloudObjectTypeAndId, DriveObjectType},
    env_vars::{EnvVarValue, SavedEnvVarCollection},
    themes::theme::Fill,
};

use super::{LocalObjectItem, LocalObjectItemId};

#[derive(Clone)]
pub struct LocalObjectEnvVarCollection {
    id: CloudObjectTypeAndId,
    env_var_collection: SavedEnvVarCollection,
}

impl LocalObjectEnvVarCollection {
    pub fn new(id: CloudObjectTypeAndId, env_var_collection: SavedEnvVarCollection) -> Self {
        Self {
            id,
            env_var_collection,
        }
    }
}

impl LocalObjectItem for LocalObjectEnvVarCollection {
    fn display_name(&self) -> Option<String> {
        self.env_var_collection.model().string_model.title.clone()
    }

    fn metadata(&self) -> Option<&CloudObjectMetadata> {
        Some(&self.env_var_collection.metadata)
    }

    fn object_type(&self) -> Option<DriveObjectType> {
        Some(DriveObjectType::EnvVarCollection)
    }

    fn secondary_icon(&self, _color: Option<Fill>) -> Option<Box<dyn Element>> {
        None
    }

    fn preview(&self, appearance: &Appearance) -> Option<Box<dyn Element>> {
        let title_text = self.env_var_collection.model().string_model.title.clone();
        let title_to_render = if let Some(title) = title_text {
            title
        } else {
            "Untitled".to_string()
        };
        let title = appearance
            .ui_builder()
            .wrappable_text(title_to_render, true)
            .with_style(UiComponentStyles {
                font_color: Some(
                    appearance
                        .theme()
                        .main_text_color(appearance.theme().background())
                        .into(),
                ),
                font_size: Some(14.),
                font_weight: Some(Weight::Bold),
                ..Default::default()
            })
            .build()
            .finish();

        let mut text = Flex::column().with_child(Container::new(title).finish());

        if let Some(description) = self
            .env_var_collection
            .model()
            .string_model
            .description
            .clone()
        {
            let description_text = appearance
                .ui_builder()
                .paragraph(description.clone())
                .with_style(UiComponentStyles {
                    font_family_id: Some(appearance.ui_font_family()),
                    font_color: Some(
                        appearance
                            .theme()
                            .sub_text_color(appearance.theme().surface_2())
                            .into(),
                    ),
                    font_size: Some(12.),
                    ..Default::default()
                });

            text.add_child(
                Container::new(description_text.build().finish())
                    .with_margin_top(4.)
                    .finish(),
            )
        }

        let rows = self
            .env_var_collection
            .model()
            .string_model
            .vars
            .iter()
            .map(|var| {
                Clipped::new(
                    appearance
                        .ui_builder()
                        .label(match &var.value {
                            EnvVarValue::Constant(val) => format!("{}: {}", var.name, val),
                            EnvVarValue::Command(cmd) => format!("{}: {}", var.name, cmd.name),
                            EnvVarValue::Secret(sec) => {
                                format!("{}: {}", var.name, sec.get_display_name())
                            }
                        })
                        .with_style(UiComponentStyles {
                            font_family_id: Some(appearance.ui_font_family()),
                            font_size: Some(12.),
                            ..Default::default()
                        })
                        .build()
                        .finish(),
                )
                .finish()
            })
            .collect_vec();

        text.add_child(
            Container::new(Flex::column().with_children(rows).finish())
                .with_margin_top(8.)
                .finish(),
        );

        Some(text.finish())
    }

    fn local_object_id(&self) -> LocalObjectItemId {
        LocalObjectItemId::Object(self.id)
    }

    fn action_summary(&self, app: &AppContext) -> Option<String> {
        ObjectActions::as_ref(app)
            .get_action_history_summary_for_action_type(&self.id.uid(), ObjectActionType::Execute)
    }

    fn clone_box(&self) -> Box<dyn LocalObjectItem> {
        Box::new(self.clone())
    }
}
