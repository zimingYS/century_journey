use std::collections::HashSet;

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::client::ui::components::{
    CreativeCategoryPanel, CreativeHotbarPanel, CreativeInventoryOverlay, CreativeItemGrid,
    CreativeRecentPanel,
};
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::category_theme::CategoryTheme;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::{
    CategoryTab, InventorySlot, SearchInputState, SlotKind, SlotVisual, spawn_slot_with_item,
    sync_hotbar_panel_visuals, sync_slot_icon,
};
use crate::client::ui::widgets::tab::spawn_category_tab;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::definition::ItemCategory;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::game::inventory::container::creative::CreativeCategory;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::state::InventoryState;
use crate::shared::item_id::ItemId;

pub use crate::client::ui::screens::setup::spawn_creative_inventory_system;

/// 切换物品栏状态
pub fn toggle_inventory_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    search_state: Res<SearchInputState>,
    gamemode: Res<crate::game::gameplay::gamemode::PlayerGameMode>,
    mut state: ResMut<InventoryState>,
    mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    if search_state.active {
        return;
    }

    state.toggle();

    let Ok(mut cursor) = cursor_query.single_mut() else {
        return;
    };
    if state.opened {
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
        info!("Opened inventory in {:?} mode", gamemode.mode);
    } else {
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
        if gamemode.is_creative() {
            state.cursor.clear();
        } else {
            // Survival: 尝试放回背包来源槽位
            crate::client::ui::screens::survival_inventory::handle_inventory_close(&mut state);
        }
    }
}

pub fn update_creative_visibility_system(
    state: Res<InventoryState>,
    gamemode: Res<crate::game::gameplay::gamemode::PlayerGameMode>,
    mut query: Query<&mut Visibility, With<CreativeInventoryOverlay>>,
) {
    let Ok(mut vis) = query.single_mut() else {
        return;
    };
    let target = if state.opened && gamemode.is_creative() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    if *vis != target {
        *vis = target;
    }
}

/// 构造标签数据
pub fn build_creative_categories_system(
    tag_registry: Option<Res<RuntimeTagRegistry>>,
    block_registry: Option<Res<BlockRegistry>>,
    mut state: ResMut<InventoryState>,
    category_panel: Query<Entity, With<CreativeCategoryPanel>>,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
    cat_theme: Res<CategoryTheme>,
    mut commands: Commands,
    item_registry: Option<Res<ItemRegistry>>,
) {
    let Some(tag_reg) = tag_registry else { return };
    let Some(block_reg) = block_registry else {
        return;
    };
    if !state.creative.categories.is_empty() {
        return;
    }

    state
        .creative
        .categories
        .push(CreativeCategory::virtual_category("全部", ""));

    let tags: Vec<_> = tag_reg.all_tags().cloned().collect();
    for tag in &tags {
        let items: Vec<ItemId> = tag_reg
            .get_ids(tag)
            .map(|ids| {
                ids.iter()
                    .filter_map(|&id| block_reg.get_identifier_by_id(id))
                    .map(|ident| ItemId::new(ident.clone()))
                    .collect()
            })
            .unwrap_or_default();

        state.creative.categories.push(CreativeCategory::from_tag(
            tag.clone(),
            cat_theme.display_name(&tag.to_full()),
            cat_theme.icon(&tag.to_full()),
            items,
        ));
    }

    // 工具分类
    let categories = &mut state.creative.categories;
    categories.push(CreativeCategory::virtual_category("工具", ""));
    if let Some(item_reg) = item_registry.as_ref() {
        let tool_ids = item_reg.items_by_category(&ItemCategory::Tool);
        if let Some(last) = categories.last_mut() {
            last.items = tool_ids.to_vec();
        }
    }
    // 如果工具分类为空，保留空分类

    state
        .creative
        .categories
        .push(CreativeCategory::virtual_category("收藏", ""));

    let Ok(panel_entity) = category_panel.single() else {
        return;
    };
    commands.entity(panel_entity).with_children(|panel| {
        for (idx, cat) in state.creative.categories.iter().enumerate() {
            spawn_category_tab(
                panel,
                &cat.display_name,
                &cat.icon,
                idx,
                idx == state.creative.selected_tab,
                &ui_font,
                &theme,
            );
        }
    });
}

/// 搜索过滤更新
pub fn update_creative_filter_system(
    block_registry: Option<Res<BlockRegistry>>,
    item_registry: Option<Res<ItemRegistry>>,
    mut state: ResMut<InventoryState>,
) {
    let Some(reg) = block_registry else { return };
    if state.creative.categories.is_empty() {
        return;
    }

    let tab = state.creative.selected_tab;
    let search = state.creative.search_text.clone();

    let mut new_items = if tab == 0 {
        // "全部"标签页：方块 + 物品，去重
        let mut seen = HashSet::new();
        let mut all: Vec<ItemId> = Vec::new();

        // BlockRegistry 中的方块
        for id in reg.identifiers() {
            if id == "century_journey:air" {
                continue;
            }
            let item_id = ItemId::new(id.clone());
            if seen.insert(item_id.clone()) {
                all.push(item_id);
            }
        }

        // ItemRegistry 中的物品（包括自动生成方块物品）
        if let Some(item_reg) = item_registry.as_ref() {
            for def in item_reg.all_items() {
                let item_id = ItemId::new(def.identifier.clone());
                if seen.insert(item_id.clone()) {
                    all.push(item_id);
                }
            }
        }
        all
    } else if let Some(cat) = state.creative.categories.get(tab) {
        if cat.tag_id.is_none() && cat.display_name == "收藏" {
            state.creative.favorites.clone()
        } else {
            cat.items.clone()
        }
    } else {
        Vec::new()
    };

    if !search.is_empty() {
        let keyword = search.to_lowercase();
        new_items.retain(|item| item.to_string().to_lowercase().contains(&keyword));
    }

    if state.creative.visible_items != new_items {
        state.creative.visible_items = new_items;
    }
}

/// 创造模式网格填充
pub fn populate_creative_grid_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    grid_query: Query<Entity, With<CreativeItemGrid>>,
    children_query: Query<&Children>,
    existing_slots: Query<(Entity, &InventorySlot)>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    theme: Res<UiTheme>,
    mut commands: Commands,
    mut last_items: Local<Vec<ItemId>>,
) {
    let Some(reg) = block_registry.as_ref() else {
        return;
    };
    let Ok(grid_entity) = grid_query.single() else {
        return;
    };

    if *last_items == state.creative.visible_items {
        return;
    }
    *last_items = state.creative.visible_items.clone();

    // 收集现有 CreativeGrid 槽位
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

    // 数量匹配 → 原地更新图标
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
                &children_query,
                item_registry.as_deref(),
                item_texture_registry.as_deref(),
            );
        }
        return;
    }

    // 数量不匹配 → 重建
    if let Ok(children) = children_query.get(grid_entity) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    commands.entity(grid_entity).with_children(|grid| {
        for (index, item) in new_items.iter().enumerate() {
            spawn_slot_with_item(
                grid,
                SlotKind::CreativeGrid,
                index,
                item,
                reg,
                &theme,
                item_registry.as_deref(),
                item_texture_registry.as_deref(),
            );
        }
    });
}

/// 最近使用面板填充
pub fn populate_recent_panel_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    recent_query: Query<Entity, With<CreativeRecentPanel>>,
    children_query: Query<&Children>,
    existing_slots: Query<(Entity, &InventorySlot)>,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    mut commands: Commands,
    mut last_items: Local<Vec<ItemStack>>,
) {
    let Some(reg) = block_registry.as_ref() else {
        return;
    };
    let Ok(panel_entity) = recent_query.single() else {
        return;
    };

    if *last_items == state.recent.items {
        return;
    }
    *last_items = state.recent.items.clone();

    // 收集现有 Recent 槽位（跳过第一子节点 "最近使用:" 文本）
    let mut slot_entities: Vec<(Entity, usize)> = Vec::new();
    if let Ok(children) = children_query.get(panel_entity) {
        for child in children.iter().skip(1) {
            // skip label
            if let Ok(slot) = existing_slots.get(child)
                && slot.1.kind == SlotKind::Recent
            {
                slot_entities.push((child, slot.1.index));
            }
        }
    }

    let new_items = &state.recent.items;

    if slot_entities.len() == new_items.len() {
        for (entity, idx) in slot_entities {
            let air = ItemId::air();
            let (item, count) = new_items
                .get(idx)
                .map(|s| (&s.item, s.count))
                .unwrap_or((&air, 0u32));
            sync_slot_icon(
                &mut commands,
                entity,
                item,
                count,
                reg,
                &children_query,
                item_registry.as_deref(),
                item_texture_registry.as_deref(),
            );
        }
        return;
    }

    // 重建
    if let Ok(children) = children_query.get(panel_entity) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    commands.entity(panel_entity).with_children(|panel| {
        panel.spawn((
            Text::new("最近使用:"),
            TextFont {
                font: FontSource::from(ui_font.default.clone()),
                font_size: FontSize::Px(theme.small_font_size),
                ..default()
            },
            TextColor(theme.text_secondary),
            Node {
                margin: UiRect::right(Val::Px(8.0)),
                ..default()
            },
        ));
        for (index, stack) in new_items.iter().enumerate() {
            spawn_slot_with_item(
                panel,
                SlotKind::Recent,
                index,
                &stack.item,
                reg,
                &theme,
                item_registry.as_deref(),
                item_texture_registry.as_deref(),
            );
        }
    });
}

/// 创造模式快捷栏
pub fn init_creative_hotbar_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    hotbar_query: Query<Entity, With<CreativeHotbarPanel>>,
    children_query: Query<&Children>,
    slot_query: Query<&InventorySlot>,
    theme: Res<UiTheme>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    mut commands: Commands,
) {
    let Some(reg) = block_registry.as_ref() else {
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

    commands.entity(panel_entity).with_children(|bar| {
        for (index, item) in state.hotbar.items().iter().enumerate() {
            spawn_slot_with_item(
                bar,
                SlotKind::Hotbar,
                index,
                item,
                reg,
                &theme,
                item_registry.as_deref(),
                item_texture_registry.as_deref(),
            );
        }
    });
}

/// 创造模式快捷栏视觉同步
pub fn creative_hotbar_visual_sync_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    hotbar_query: Query<Entity, With<CreativeHotbarPanel>>,
    children_query: Query<&Children>,
    theme: Res<UiTheme>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    mut slot_query: Query<(Entity, &InventorySlot, &mut SlotVisual)>,
    mut commands: Commands,
    mut border_query: Query<(&InventorySlot, &mut BorderColor)>,
    mut last_hotbar: Local<Option<Vec<(ItemId, u32)>>>,
    mut was_opened: Local<bool>,
) {
    let Some(reg) = block_registry.as_ref() else {
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

/// 关闭物品栏时清理创造模式快捷栏子实体
pub fn cleanup_creative_hotbar_system(
    state: Res<InventoryState>,
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

///  分类标签高亮
pub fn update_category_highlight_system(
    state: Res<InventoryState>,
    theme: Res<UiTheme>,
    mut query: Query<(&CategoryTab, &mut BackgroundColor)>,
) {
    for (tab, mut bg) in &mut query {
        *bg = BackgroundColor(if tab.category_index == state.creative.selected_tab {
            theme.tab_active_bg
        } else {
            Color::NONE
        });
    }
}
