//! 生成并增量同步创造物品网格和最近使用面板。

use bevy::prelude::*;

use crate::client::renderer::item::GuiItemIconCache;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::ui::components::{CreativeItemGrid, CreativeRecentPanel};
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::{
    InventorySlot, SlotKind, spawn_slot_with_item, sync_slot_icon,
};
use crate::content::block::registry::BlockRegistry;
use crate::content::item::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::state::LocalInventory;
use crate::shared::item_id::ItemId;

/// 右侧最近使用面板固定显示的槽位数量。
const RECENT_SLOT_COUNT: usize = 12;
/// 创造物品网格槽位的固定像素尺寸。
pub(super) const CREATIVE_SLOT_SIZE: f32 = 74.0;
const CREATIVE_RECENT_SLOT_SIZE: f32 = 58.0;
const CREATIVE_SLOT_GAP: f32 = 6.0;
/// 为创造物品栏生成局部槽位主题，避免影响 HUD 和生存背包。
pub(super) fn creative_slot_theme(theme: &UiTheme, slot_size: f32) -> UiTheme {
    let mut theme = theme.clone();
    theme.slot_size = slot_size;
    theme.slot_gap = CREATIVE_SLOT_GAP;
    theme
}

/// 创造模式物品网格填充。
/// 槽位生成需要显式读取全部模型和图标缓存，但不持有这些资源。
#[allow(clippy::too_many_arguments)]
pub fn populate_creative_grid_system(
    state: LocalInventory,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    gui_item_icons: Res<GuiItemIconCache>,
    grid_query: Query<Entity, With<CreativeItemGrid>>,
    children_query: Query<&Children>,
    existing_slots: Query<(Entity, &InventorySlot)>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    theme: Res<UiTheme>,
    ui_font: Res<UiFont>,
    mut commands: Commands,
    mut last_items: Local<Option<(Vec<ItemId>, u64)>>,
) {
    let Some(reg) = block_registry.as_ref() else {
        return;
    };
    let Some(render_assets) = block_render_assets.as_ref() else {
        return;
    };
    let Ok(grid_entity) = grid_query.single() else {
        return;
    };

    let revision = gui_item_icons.revision();
    if last_items.as_ref().is_some_and(|(items, cached_revision)| {
        items == &state.creative.visible_items && *cached_revision == revision
    }) {
        return;
    }
    *last_items = Some((state.creative.visible_items.clone(), revision));

    let mut slot_indices: Vec<(Entity, usize)> = Vec::new();
    if let Ok(children) = children_query.get(grid_entity) {
        for child in children.iter() {
            if let Ok(slot) = existing_slots.get(child)
                && slot.1.kind == SlotKind::CreativeGrid
            {
                slot_indices.push((child, slot.1.index));
            }
        }
    }

    let new_items = &state.creative.visible_items;

    if slot_indices.len() == new_items.len() {
        for (entity, idx) in slot_indices {
            let air = &ItemId::air();
            let item = new_items.get(idx).unwrap_or(air);
            sync_slot_icon(
                &mut commands,
                entity,
                item,
                0,
                reg,
                render_assets,
                &gui_item_icons,
                &children_query,
                item_registry.as_deref(),
                item_texture_registry.as_deref(),
            );
        }
        return;
    }

    if let Ok(children) = children_query.get(grid_entity) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    let creative_theme = creative_slot_theme(theme.as_ref(), CREATIVE_SLOT_SIZE);

    commands.entity(grid_entity).with_children(|grid| {
        for (index, item) in new_items.iter().enumerate() {
            spawn_slot_with_item(
                grid,
                SlotKind::CreativeGrid,
                index,
                item,
                reg,
                render_assets,
                &gui_item_icons,
                &creative_theme,
                &ui_font,
                item_registry.as_deref(),
                item_texture_registry.as_deref(),
            );
        }
    });
}

/// 最近使用面板填充：固定补齐 12 个槽位，保持右侧栏稳定。
/// 最近物品面板复用完整槽位渲染依赖，并通过本地快照避免重复重建。
#[allow(clippy::too_many_arguments)]
pub fn populate_recent_panel_system(
    state: LocalInventory,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    gui_item_icons: Res<GuiItemIconCache>,
    recent_query: Query<Entity, With<CreativeRecentPanel>>,
    children_query: Query<&Children>,
    existing_slots: Query<(Entity, &InventorySlot)>,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    mut commands: Commands,
    mut last_items: Local<Option<(Vec<ItemStack>, u64)>>,
) {
    let Some(reg) = block_registry.as_ref() else {
        return;
    };
    let Some(render_assets) = block_render_assets.as_ref() else {
        return;
    };
    let Ok(panel_entity) = recent_query.single() else {
        return;
    };

    let revision = gui_item_icons.revision();
    if last_items.as_ref().is_some_and(|(items, cached_revision)| {
        items == &state.recent.items && *cached_revision == revision
    }) {
        return;
    }
    *last_items = Some((state.recent.items.clone(), revision));

    let mut slot_entities: Vec<(Entity, usize)> = Vec::new();
    if let Ok(children) = children_query.get(panel_entity) {
        for child in children.iter() {
            if let Ok((entity, slot)) = existing_slots.get(child)
                && slot.kind == SlotKind::Recent
            {
                slot_entities.push((entity, slot.index));
            }
        }
    }
    slot_entities.sort_by_key(|(_, index)| *index);

    if slot_entities.len() == RECENT_SLOT_COUNT {
        for (entity, idx) in slot_entities {
            let air = ItemId::air();
            let (item, count) = state
                .recent
                .items
                .get(idx)
                .map(|stack| (&stack.item, stack.count))
                .unwrap_or((&air, 0));
            sync_slot_icon(
                &mut commands,
                entity,
                item,
                count,
                reg,
                render_assets,
                &gui_item_icons,
                &children_query,
                item_registry.as_deref(),
                item_texture_registry.as_deref(),
            );
        }
        return;
    }

    if let Ok(children) = children_query.get(panel_entity) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    let recent_theme = creative_slot_theme(theme.as_ref(), CREATIVE_RECENT_SLOT_SIZE);

    commands.entity(panel_entity).with_children(|panel| {
        panel.spawn((
            Text::new("最近使用"),
            TextFont {
                font: FontSource::from(ui_font.default.clone()),
                font_size: FontSize::Px(theme.body_font_size + 6.0),
                ..default()
            },
            TextColor(theme.text_primary),
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(34.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ));

        for index in 0..RECENT_SLOT_COUNT {
            let air = ItemId::air();
            let item = state
                .recent
                .items
                .get(index)
                .map(|stack| &stack.item)
                .unwrap_or(&air);
            spawn_slot_with_item(
                panel,
                SlotKind::Recent,
                index,
                item,
                reg,
                render_assets,
                &gui_item_icons,
                &recent_theme,
                &ui_font,
                item_registry.as_deref(),
                item_texture_registry.as_deref(),
            );
        }

        // 底部箱子按钮是视觉占位，后续可接入保存/加载创造热键栏。
        panel
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(96.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(12.0)),
                    border: UiRect::top(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(Color::srgba(0.20, 0.20, 0.20, 1.0)),
            ))
            .with_children(|footer| {
                footer
                    .spawn((
                        Node {
                            width: Val::Px(64.0),
                            height: Val::Px(64.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.12, 0.11, 0.10, 1.0)),
                        BorderColor::all(Color::srgba(0.34, 0.31, 0.27, 1.0)),
                    ))
                    .with_children(|slot| {
                        slot.spawn((
                            Text::new("箱"),
                            TextFont {
                                font: FontSource::from(ui_font.default.clone()),
                                font_size: FontSize::Px(30.0),
                                ..default()
                            },
                            TextColor(Color::srgba(0.75, 0.46, 0.20, 1.0)),
                        ));
                    });
            });
    });
}
