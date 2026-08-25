//! 根据悬停槽位构建、定位并隐藏物品提示框。

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::SlotVisual;
use crate::content::item::ItemRegistry;
use crate::content::item::definition::tool::{ToolTier, ToolType};
use crate::content::item::definition::{ItemCategory, ItemDefinition};
use crate::engine::localization::Localization;

const TOOLTIP_WIDTH: f32 = 292.0;
const CURSOR_OFFSET: f32 = 16.0;

/// 全局物品提示框根节点。
#[derive(Component)]
pub struct ItemTooltip;

/// 物品提示框标题文本。
#[derive(Component)]
pub(crate) struct ItemTooltipTitle;

/// 物品提示框属性正文文本。
#[derive(Component)]
pub(crate) struct ItemTooltipBody;

/// 创建默认隐藏的全局物品提示框。
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

/// 根据悬停槽位和指针位置更新提示框内容与屏幕内布局。
///
/// 查询保持分离以避免可变文本和节点访问冲突，因此保留显式系统参数。
#[allow(clippy::too_many_arguments)]
pub(crate) fn item_tooltip_system(
    mut cursor_events: MessageReader<CursorMoved>,
    ui_scale: Res<UiScale>,
    item_registry: Option<Res<ItemRegistry>>,
    localization: Res<Localization>,
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
        .map(|definition| tooltip_text(definition, &localization))
        .unwrap_or_else(|| {
            let name_key = format!(
                "item.{}.{}",
                item.identifier().namespace(),
                item.identifier().path()
            );
            let title = localization
                .get_or(&name_key, item.identifier().path())
                .to_owned();
            let body = format!(
                "{}\n{}",
                localization.format(
                    "tooltip.category",
                    &[("name", localization.get("tooltip.category-none"))]
                ),
                localization.format("tooltip.identifier", &[("item", &item.to_string())]),
            );
            (title, body)
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

fn tooltip_text(definition: &ItemDefinition, localization: &Localization) -> (String, String) {
    let name_key = definition.name_key();
    let title = localization
        .get_or(&name_key, &definition.display_name)
        .to_owned();
    let mut lines = vec![
        localization.format(
            "tooltip.category",
            &[("name", category_name(definition.category, localization))],
        ),
        localization.format(
            "tooltip.max-stack",
            &[("count", &definition.max_stack.to_string())],
        ),
    ];
    if definition.is_placeable() {
        lines.push(localization.get("tooltip.placeable").to_owned());
    }
    if let Some(tool) = definition.tool_data() {
        lines.push(localization.format(
            "tooltip.tool-type",
            &[("type", tool_type_name(tool.tool_type, localization))],
        ));
        lines.push(localization.format(
            "tooltip.tool-tier",
            &[("tier", tool_tier_name(tool.tier, localization))],
        ));
        lines.push(localization.format(
            "tooltip.efficiency",
            &[("value", &format!("{:.1}", tool.efficiency))],
        ));
        lines.push(localization.format(
            "tooltip.durability",
            &[("count", &tool.max_durability.to_string())],
        ));
    }
    if !definition.tags.is_empty() {
        lines.push(localization.format("tooltip.tags", &[("tags", &definition.tags.join("、"))]));
    }
    (title, lines.join("\n"))
}

/// 查询物品类别名称的本地化译文。
fn category_name(category: ItemCategory, localization: &Localization) -> &str {
    let key = match category {
        ItemCategory::Block => "tooltip.categories.block",
        ItemCategory::Material => "tooltip.categories.material",
        ItemCategory::Tool => "tooltip.categories.tool",
        ItemCategory::Weapon => "tooltip.categories.weapon",
        ItemCategory::Armor => "tooltip.categories.armor",
        ItemCategory::Accessory => "tooltip.categories.accessory",
        ItemCategory::Consumable => "tooltip.categories.consumable",
    };
    localization.get(key)
}

/// 查询工具类型名称的本地化译文。
fn tool_type_name(tool_type: ToolType, localization: &Localization) -> &str {
    let key = match tool_type {
        ToolType::Pickaxe => "tooltip.tool-types.pickaxe",
        ToolType::Axe => "tooltip.tool-types.axe",
        ToolType::Shovel => "tooltip.tool-types.shovel",
        ToolType::Hoe => "tooltip.tool-types.hoe",
    };
    localization.get(key)
}

/// 查询工具等级名称的本地化译文。
fn tool_tier_name(tier: ToolTier, localization: &Localization) -> &str {
    let key = match tier {
        ToolTier::Wood => "tooltip.tier.wood",
        ToolTier::Stone => "tooltip.tier.stone",
        ToolTier::Iron => "tooltip.tier.iron",
        ToolTier::Diamond => "tooltip.tier.diamond",
    };
    localization.get(key)
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/ui/widgets/tooltip.rs"]
mod tests;
