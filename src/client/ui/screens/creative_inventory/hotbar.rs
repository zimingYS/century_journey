//! 创建并同步创造物品栏中的快捷栏视图。

use bevy::prelude::*;

use super::grid::{CREATIVE_SLOT_SIZE, creative_slot_theme};
use crate::client::renderer::item::GuiItemIconCache;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::ui::components::CreativeHotbarPanel;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::{
    InventorySlot, SlotKind, SlotVisual, spawn_slot_with_item, sync_hotbar_panel_visuals,
};
use crate::content::block::registry::BlockRegistry;
use crate::content::item::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::inventory::state::LocalInventory;
use crate::shared::item_id::ItemId;
/// 创造模式快捷栏。
/// 快捷栏初始化显式读取各类渲染缓存，以便内容未就绪时安全跳过。
#[allow(clippy::too_many_arguments)]
pub fn init_creative_hotbar_system(
    state: LocalInventory,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    gui_item_icons: Res<GuiItemIconCache>,
    hotbar_query: Query<Entity, With<CreativeHotbarPanel>>,
    children_query: Query<&Children>,
    slot_query: Query<&InventorySlot>,
    theme: Res<UiTheme>,
    ui_font: Res<UiFont>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    mut commands: Commands,
) {
    let Some(reg) = block_registry.as_ref() else {
        return;
    };
    let Some(render_assets) = block_render_assets.as_ref() else {
        return;
    };
    let Ok(panel_entity) = hotbar_query.single() else {
        return;
    };

    let has_hotbar_slots = children_query
        .get(panel_entity)
        .map(|children| {
            children.iter().any(|child| {
                slot_query
                    .get(child)
                    .is_ok_and(|s| s.kind == SlotKind::Hotbar)
            })
        })
        .unwrap_or(false);

    if has_hotbar_slots {
        return;
    }

    let creative_theme = creative_slot_theme(theme.as_ref(), CREATIVE_SLOT_SIZE);

    commands.entity(panel_entity).with_children(|bar| {
        for (index, stack) in state.hotbar.stacks.iter().enumerate() {
            let item = stack
                .as_ref()
                .map_or_else(ItemId::air, |stack| stack.item.clone());
            spawn_slot_with_item(
                bar,
                SlotKind::Hotbar,
                index,
                &item,
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

/// 创造模式快捷栏视觉同步。
/// 本地快照与渲染缓存共同决定是否刷新，保持参数显式便于审查失效条件。
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn creative_hotbar_visual_sync_system(
    state: LocalInventory,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    gui_item_icons: Res<GuiItemIconCache>,
    hotbar_query: Query<Entity, With<CreativeHotbarPanel>>,
    children_query: Query<&Children>,
    theme: Res<UiTheme>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    mut slot_query: Query<(Entity, &InventorySlot, &mut SlotVisual)>,
    mut commands: Commands,
    mut border_query: Query<(&InventorySlot, &mut BorderColor)>,
    mut last_hotbar: Local<Option<(Vec<(ItemId, u32)>, u64)>>,
    mut was_opened: Local<bool>,
) {
    let Some(reg) = block_registry.as_ref() else {
        return;
    };
    let Some(render_assets) = block_render_assets.as_ref() else {
        return;
    };
    let Ok(hotbar_entity) = hotbar_query.single() else {
        return;
    };

    let force_reset = state.opened && !*was_opened;
    *was_opened = state.opened;

    sync_hotbar_panel_visuals(
        &state,
        reg,
        render_assets,
        &gui_item_icons,
        hotbar_entity,
        &children_query,
        item_registry.as_deref(),
        item_texture_registry.as_deref(),
        &mut slot_query,
        &mut border_query,
        &theme,
        &mut commands,
        &mut last_hotbar,
        force_reset,
    );
}

/// 关闭物品栏时清理创造模式快捷栏子实体。
pub fn cleanup_creative_hotbar_system(
    state: LocalInventory,
    hotbar_query: Query<Entity, With<CreativeHotbarPanel>>,
    children_query: Query<&Children>,
    mut commands: Commands,
    mut was_opened: Local<bool>,
) {
    if *was_opened
        && !state.opened
        && let Ok(panel_entity) = hotbar_query.single()
        && let Ok(children) = children_query.get(panel_entity)
    {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }
    *was_opened = state.opened;
}
