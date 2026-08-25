//! 生成可交互槽位和仅展示槽位的实体层级。

use bevy::prelude::*;

use super::components::{InventorySlot, SlotCountText, SlotIcon, SlotPlaceholder, SlotVisual};
use super::durability::spawn_durability_bar;
use super::icon::{icon_node, spawn_icon_child};
use crate::client::renderer::item::GuiItemIconCache;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::inventory::slot::SlotKind;
use crate::shared::item_id::ItemId;
/// 生成空槽位。
pub fn spawn_empty_slot(
    parent: &mut ChildSpawnerCommands,
    kind: SlotKind,
    index: usize,
    theme: &UiTheme,
    ui_font: &UiFont,
) {
    parent
        .spawn((
            InventorySlot { kind, index },
            SlotVisual {
                item: ItemId::air(),
                count: 0,
            },
            Button,
            Pickable::default(),
            Node {
                width: Val::Px(theme.slot_size),
                height: Val::Px(theme.slot_size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(theme.slot_border)),
                ..default()
            },
            BackgroundColor(theme.bg_slot),
            BorderColor::all(theme.border_default),
        ))
        .with_children(|slot| {
            slot.spawn((
                SlotIcon,
                Node {
                    width: Val::Percent(80.0),
                    height: Val::Percent(80.0),
                    ..default()
                },
                Visibility::Hidden,
            ));
            slot.spawn((
                SlotCountText,
                Text::new(""),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(1.0),
                    right: Val::Px(3.0),
                    ..default()
                },
                Visibility::Hidden,
            ));
            spawn_durability_bar(slot, kind, index);
        });
}

/// 生成带短占位标记的空槽位，用于装备栏和饰品栏。
///
/// 占位文字以本地化键传入，实体带 [`LocalizedText`] 标记以便语言切换后刷新。
pub fn spawn_empty_slot_with_placeholder(
    parent: &mut ChildSpawnerCommands,
    kind: SlotKind,
    index: usize,
    placeholder_key: &str,
    theme: &UiTheme,
    ui_font: &UiFont,
    localization: &crate::engine::localization::Localization,
) {
    parent
        .spawn((
            InventorySlot { kind, index },
            SlotVisual {
                item: ItemId::air(),
                count: 0,
            },
            Button,
            Pickable::default(),
            Node {
                width: Val::Px(theme.slot_size),
                height: Val::Px(theme.slot_size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(theme.slot_border)),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(theme.bg_slot),
            BorderColor::all(theme.border_default),
        ))
        .with_children(|slot| {
            slot.spawn((
                SlotIcon,
                Node {
                    width: Val::Percent(80.0),
                    height: Val::Percent(80.0),
                    ..default()
                },
                Visibility::Hidden,
            ));
            slot.spawn((
                SlotCountText,
                Text::new(""),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(1.0),
                    right: Val::Px(3.0),
                    ..default()
                },
                Visibility::Hidden,
            ));
            slot.spawn((
                SlotPlaceholder,
                crate::client::ui::localization::LocalizedText::new(placeholder_key),
                Text::new(localization.get(placeholder_key)),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(theme.small_font_size),
                    ..default()
                },
                TextColor(theme.text_hint),
            ));
            spawn_durability_bar(slot, kind, index);
        });
}

/// 生成带物品图标的槽位。
/// 创建带初始物品的槽位时复用完整渲染缓存，避免函数内部访问全局资源。
#[allow(clippy::too_many_arguments)]
pub fn spawn_slot_with_item(
    parent: &mut ChildSpawnerCommands,
    kind: SlotKind,
    index: usize,
    item: &ItemId,
    registry: &BlockRegistry,
    render_assets: &BlockRenderAssets,
    gui_item_icons: &GuiItemIconCache,
    theme: &UiTheme,
    ui_font: &UiFont,
    item_registry: Option<&ItemRegistry>,
    item_texture_registry: Option<&ItemTextureRegistry>,
) {
    parent
        .spawn((
            InventorySlot { kind, index },
            SlotVisual {
                item: item.clone(),
                count: 0,
            },
            Button,
            Pickable::default(),
            Node {
                width: Val::Px(theme.slot_size),
                height: Val::Px(theme.slot_size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(theme.slot_border)),
                ..default()
            },
            BackgroundColor(theme.bg_slot),
            BorderColor::all(theme.border_default),
        ))
        .with_children(|slot| {
            spawn_icon_child(
                slot,
                item,
                registry,
                render_assets,
                gui_item_icons,
                item_registry,
                item_texture_registry,
            );
            slot.spawn((
                SlotCountText,
                Text::new(""),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(1.0),
                    right: Val::Px(3.0),
                    ..default()
                },
                Visibility::Hidden,
            ));
            spawn_durability_bar(slot, kind, index);
        });
}

/// 生成仅展示用槽位。
pub fn spawn_display_only_slot(
    parent: &mut ChildSpawnerCommands,
    kind: SlotKind,
    index: usize,
    theme: &UiTheme,
    ui_font: &UiFont,
) {
    parent
        .spawn((
            InventorySlot { kind, index },
            SlotVisual {
                item: ItemId::air(),
                count: 0,
            },
            Node {
                width: Val::Px(theme.slot_size),
                height: Val::Px(theme.slot_size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(theme.slot_border)),
                ..default()
            },
            BackgroundColor(theme.bg_slot),
            BorderColor::all(theme.border_default),
        ))
        .with_children(|slot| {
            slot.spawn((SlotIcon, icon_node(), Visibility::Hidden));
            slot.spawn((
                SlotCountText,
                Text::new(""),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(1.0),
                    right: Val::Px(3.0),
                    ..default()
                },
                Visibility::Hidden,
            ));
            spawn_durability_bar(slot, kind, index);
        });
}
