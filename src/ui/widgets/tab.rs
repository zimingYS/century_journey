use crate::ui::resources::ui_font::UiFont;
use crate::ui::theme::ui_theme::UiTheme;
use crate::ui::widgets::slot::CategoryTab;
use bevy::prelude::*;

pub fn spawn_category_tab(
    parent: &mut ChildSpawnerCommands,
    display_name: &str,
    icon: &str,
    category_index: usize,
    is_active: bool,
    ui_font: &UiFont,
    theme: &UiTheme,
) {
    let bg = if is_active {
        theme.tab_active_bg
    } else {
        Color::NONE
    };

    let text_color = if is_active {
        theme.tab_active_text
    } else {
        theme.tab_inactive_text
    };

    let label = if icon.is_empty() {
        display_name.to_string()
    } else {
        format!("{} {}", icon, display_name)
    };

    parent.spawn((
        CategoryTab {
            category_index,
        },

        Button,
        Pickable::default(),

        Node {
            width: Val::Percent(100.0),
            height: Val::Px(theme.tab_height),

            justify_content: JustifyContent::Start,
            align_items: AlignItems::Center,

            padding: UiRect::left(Val::Px(12.0)),

            ..default()
        },

        BackgroundColor(bg),
    ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),

                TextFont {
                    font: FontSource::from(
                        ui_font.default.clone(),
                    ),
                    font_size: FontSize::Px(
                        theme.tab_font_size,
                    ),
                    ..default()
                },

                TextColor(text_color),
            ));
        });
}