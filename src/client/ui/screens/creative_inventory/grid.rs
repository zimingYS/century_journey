//! 生成并增量同步创造物品网格和最近使用面板。

use bevy::prelude::*;

use super::skin::attach_creative_slot_skin;
use crate::client::renderer::item::GuiItemIconCache;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::ui::components::{CreativeItemGrid, CreativeRecentGrid};
use crate::client::ui::resources::creative_assets::CreativeUiAssets;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::screens::setup::CREATIVE_SLOT_GAP;
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

/// 右侧最近使用面板固定显示的槽位数量：设计稿为 2 列 x 6 行。
const RECENT_SLOT_COUNT: usize = 12;
/// 创造物品网格槽位的固定像素尺寸（设计稿 91px，面板 0.752 倍等比）。
pub(super) const CREATIVE_SLOT_SIZE: f32 = 68.0;
/// 快捷栏槽位尺寸（设计稿 97px 略大于网格槽位）。
pub(super) const CREATIVE_HOTBAR_SLOT_SIZE: f32 = 73.0;
const CREATIVE_RECENT_SLOT_SIZE: f32 = 68.0;

/// 为创造物品栏生成局部槽位主题，避免影响 HUD 和生存背包。
///
/// 槽位边框宽度清零：边框由物品框纹理自带，叠加主题边框会破坏设计稿观感。
pub(super) fn creative_slot_theme(theme: &UiTheme, slot_size: f32) -> UiTheme {
    let mut theme = theme.clone();
    theme.slot_size = slot_size;
    theme.slot_gap = CREATIVE_SLOT_GAP;
    theme.slot_border = 0.0;
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
    creative_assets: Res<CreativeUiAssets>,
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
            let slot = spawn_slot_with_item(
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
            attach_creative_slot_skin(grid, slot, &creative_assets, false);
        }
    });
}

/// 最近使用面板填充：固定补齐 12 个槽位，保持右侧栏稳定。
/// 标题和底部箱子按钮由 setup 布局一次性构建，这里只填充槽位容器。
#[allow(clippy::too_many_arguments)]
pub fn populate_recent_panel_system(
    state: LocalInventory,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    gui_item_icons: Res<GuiItemIconCache>,
    recent_query: Query<Entity, With<CreativeRecentGrid>>,
    children_query: Query<&Children>,
    existing_slots: Query<(Entity, &InventorySlot)>,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    creative_assets: Res<CreativeUiAssets>,
    mut commands: Commands,
    mut last_items: Local<Option<(Vec<ItemStack>, u64)>>,
) {
    let Some(reg) = block_registry.as_ref() else {
        return;
    };
    let Some(render_assets) = block_render_assets.as_ref() else {
        return;
    };
    let Ok(grid_entity) = recent_query.single() else {
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
    if let Ok(children) = children_query.get(grid_entity) {
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

    if let Ok(children) = children_query.get(grid_entity) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    let recent_theme = creative_slot_theme(theme.as_ref(), CREATIVE_RECENT_SLOT_SIZE);

    commands.entity(grid_entity).with_children(|grid| {
        for index in 0..RECENT_SLOT_COUNT {
            let air = ItemId::air();
            let item = state
                .recent
                .items
                .get(index)
                .map(|stack| &stack.item)
                .unwrap_or(&air);
            let slot = spawn_slot_with_item(
                grid,
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
            attach_creative_slot_skin(grid, slot, &creative_assets, false);
        }
    });
}
