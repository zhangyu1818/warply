use settings::Setting as _;
use warpui::{App, ViewHandle, platform::WindowStyle};

use crate::{
    appearance::{Appearance, AppearanceManager},
    resource_center::TipsCompleted,
    settings::ThemeSettings,
    settings_view::keybindings::KeybindingChangedNotifier,
    test_util::settings::initialize_settings_for_tests,
    themes::theme::{SelectedSystemThemes, ThemeKind},
};

use super::*;

fn add_theme_chooser(app: &mut App) -> ViewHandle<ThemeChooser> {
    let tips_completed = app.add_model(|_| TipsCompleted::default());
    let (_, theme_chooser) = app.add_window(WindowStyle::NotStealFocus, |ctx| {
        ThemeChooser::new(ctx, tips_completed)
    });
    theme_chooser
}

fn setup_app(app: &mut App) {
    initialize_settings_for_tests(app);
    app.add_singleton_model(|_| Appearance::mock());
    app.add_singleton_model(AppearanceManager::new);
    app.add_singleton_model(|_| KeybindingChangedNotifier::mock());
}

#[test]
fn select_and_save_theme_updates_theme_kind() {
    App::test((), |mut app| async move {
        setup_app(&mut app);
        let theme_chooser = add_theme_chooser(&mut app);

        theme_chooser.update(&mut app, |theme_chooser, ctx| {
            theme_chooser.mode = ThemeChooserMode::SystemAgnostic;
            theme_chooser.select_and_save_theme(&ThemeKind::Koi, ctx);
        });

        ThemeSettings::handle(&app).read(&app, |settings, _ctx| {
            assert_eq!(settings.theme_kind.value(), &ThemeKind::Koi);
        });
    });
}

#[test]
fn select_and_save_theme_updates_light_system_theme() {
    App::test((), |mut app| async move {
        setup_app(&mut app);
        ThemeSettings::handle(&app).update(&mut app, |settings, ctx| {
            settings.use_system_theme.set_value(true, ctx).unwrap();
            settings
                .selected_system_themes
                .set_value(
                    SelectedSystemThemes {
                        light: ThemeKind::Light,
                        dark: ThemeKind::Dark,
                    },
                    ctx,
                )
                .unwrap();
        });
        let theme_chooser = add_theme_chooser(&mut app);

        theme_chooser.update(&mut app, |theme_chooser, ctx| {
            theme_chooser.mode = ThemeChooserMode::SystemLight;
            theme_chooser.select_and_save_theme(&ThemeKind::Adeberry, ctx);
        });

        ThemeSettings::handle(&app).read(&app, |settings, _ctx| {
            assert_eq!(
                settings.selected_system_themes.value(),
                &SelectedSystemThemes {
                    light: ThemeKind::Adeberry,
                    dark: ThemeKind::Dark,
                }
            );
        });
    });
}

#[test]
fn select_and_save_theme_updates_dark_system_theme() {
    App::test((), |mut app| async move {
        setup_app(&mut app);
        ThemeSettings::handle(&app).update(&mut app, |settings, ctx| {
            settings.use_system_theme.set_value(true, ctx).unwrap();
            settings
                .selected_system_themes
                .set_value(
                    SelectedSystemThemes {
                        light: ThemeKind::Light,
                        dark: ThemeKind::Dark,
                    },
                    ctx,
                )
                .unwrap();
        });
        let theme_chooser = add_theme_chooser(&mut app);

        theme_chooser.update(&mut app, |theme_chooser, ctx| {
            theme_chooser.mode = ThemeChooserMode::SystemDark;
            theme_chooser.select_and_save_theme(&ThemeKind::Dracula, ctx);
        });

        ThemeSettings::handle(&app).read(&app, |settings, _ctx| {
            assert_eq!(
                settings.selected_system_themes.value(),
                &SelectedSystemThemes {
                    light: ThemeKind::Light,
                    dark: ThemeKind::Dracula,
                }
            );
        });
    });
}
