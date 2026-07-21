use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::SlotVisual;
use crate::content::item::definition::{ItemCategory, ItemDefinition};
use crate::content::item::registry::registry::ItemRegistry;

const TOOLTIP_WIDTH: f32 = 292.0;
const CURSOR_OFFSET: f32 = 16.0;

#[derive(Component)]
pub struct ItemTooltip;

#[derive(Component)]
pub(crate) struct ItemTooltipTitle;

#[derive(Component)]
pub(crate) struct ItemTooltipBody;

pub fn spawn_item_tooltip_system(
    mut commands: Commands,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
) {
    commands
        .spawn((
            ItemTooltip,
            Name::new("ItemTooltip"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(TOOLTIP_WIDTH),
                left: Val::Px(-1000.0),
                top: Val::Px(-1000.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(theme.spacing_md)),
                border: UiRect::all(Val::Px(1.0)),
                row_gap: Val::Px(theme.spacing_sm),
                ..default()
            },
            BackgroundColor(theme.tooltip_bg),
            BorderColor::all(theme.border_hover),
            ZIndex(20_000),
            Pickable::IGNORE,
            Visibility::Hidden,
        ))
        .with_children(|tooltip| {
            tooltip.spawn((
                ItemTooltipTitle,
                Text::new(""),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(theme.body_font_size + 3.0),
                    ..default()
                },
                TextColor(theme.text_primary),
            ));
            tooltip.spawn((
                ItemTooltipBody,
                Text::new(""),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(theme.small_font_size + 1.0),
                    ..default()
                },
                TextColor(theme.text_secondary),
            ));
        });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn item_tooltip_system(
    mut cursor_events: MessageReader<CursorMoved>,
    ui_scale: Res<UiScale>,
    item_registry: Option<Res<ItemRegistry>>,
    slot_query: Query<(&Interaction, &SlotVisual)>,
    mut tooltip_query: Query<(&mut Node, &mut Visibility), With<ItemTooltip>>,
    mut title_query: Query<&mut Text, (With<ItemTooltipTitle>, Without<ItemTooltipBody>)>,
    mut body_query: Query<&mut Text, (With<ItemTooltipBody>, Without<ItemTooltipTitle>)>,
    mut cursor_position: Local<Option<Vec2>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    for event in cursor_events.read() {
        *cursor_position = Some(event.position);
    }
    let hovered_item = slot_query
        .iter()
        .find(|(interaction, visual)| {
            **interaction == Interaction::Hovered && !visual.item.is_air()
        })
        .map(|(_, visual)| visual.item.clone());
    let Ok((mut node, mut visibility)) = tooltip_query.single_mut() else {
        return;
    };
    let Some(item) = hovered_item else {
        *visibility = Visibility::Hidden;
        return;
    };
    let (title, body) = item_registry
        .as_deref()
        .and_then(|registry| registry.get(&item))
        .map(tooltip_text)
        .unwrap_or_else(|| {
            (
                item.display_name().to_string(),
                format!("类别  未分类\n标识  {item}"),
            )
        });
    if let Ok(mut text) = title_query.single_mut() {
        *text = Text::new(title);
    }
    if let Ok(mut text) = body_query.single_mut() {
        *text = Text::new(body);
    }
    if let Some(cursor) = *cursor_position {
        let virtual_position = cursor / ui_scale.0.max(0.01);
        let viewport = window_query
            .single()
            .map(|window| Vec2::new(window.width(), window.height()) / ui_scale.0.max(0.01))
            .unwrap_or(Vec2::new(1920.0, 1080.0));
        node.left = Val::Px(
            (virtual_position.x + CURSOR_OFFSET)
                .min(viewport.x - TOOLTIP_WIDTH - CURSOR_OFFSET)
                .max(0.0),
        );
        node.top = Val::Px(
            (virtual_position.y + CURSOR_OFFSET)
                .min(viewport.y - 150.0)
                .max(0.0),
        );
    }
    *visibility = Visibility::Visible;
}

fn tooltip_text(definition: &ItemDefinition) -> (String, String) {
    let mut lines = vec![
        format!("类别  {}", category_name(definition.category)),
        format!("最大堆叠  {}", definition.max_stack),
    ];
    if definition.is_placeable() {
        lines.push("属性  可放置".to_string());
    }
    if let Some(tool) = definition.tool_data() {
        lines.push(format!("工具类型  {:?}", tool.tool_type));
        lines.push(format!("工具等级  {:?}", tool.tier));
        lines.push(format!("效率  {:.1}x", tool.efficiency));
        lines.push(format!("耐久上限  {}", tool.max_durability));
    }
    if !definition.tags.is_empty() {
        lines.push(format!("标签  {}", definition.tags.join("、")));
    }
    (definition.display_name.clone(), lines.join("\n"))
}

fn category_name(category: ItemCategory) -> &'static str {
    match category {
        ItemCategory::Block => "方块",
        ItemCategory::Material => "材料",
        ItemCategory::Tool => "工具",
        ItemCategory::Weapon => "武器",
        ItemCategory::Armor => "护甲",
        ItemCategory::Accessory => "饰品",
        ItemCategory::Consumable => "消耗品",
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/ui/widgets/tooltip.rs"]
mod tests;
