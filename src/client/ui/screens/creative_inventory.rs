use std::collections::HashSet;

use bevy::prelude::*;

use crate::client::renderer::item_model::ItemModelRenderAssets;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::ui::components::{
    CreativeCategoryPanel, CreativeCloseButton, CreativeHotbarPanel, CreativeInventoryOverlay,
    CreativeItemGrid, CreativeRecentPanel, CreativeSearchPlaceholder,
};
use crate::client::ui::navigation::{UiNavigation, UiScreen};
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::category_theme::CategoryTheme;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::common::UiControl;
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
use crate::game::inventory::state::{LocalInventory, LocalInventoryMut};
use crate::shared::item_id::ItemId;
use crate::shared::tag::identifier::TagId;

pub use crate::client::ui::screens::setup::spawn_creative_inventory_system;

/// 右侧最近使用面板固定显示的槽位数量。
const RECENT_SLOT_COUNT: usize = 12;
const CREATIVE_SLOT_SIZE: f32 = 74.0;
const CREATIVE_RECENT_SLOT_SIZE: f32 = 58.0;
const CREATIVE_SLOT_GAP: f32 = 6.0;

/// 同步创造物品栏遮罩显隐。
pub fn update_creative_visibility_system(
    state: LocalInventory,
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

/// 点击右上角关闭按钮时关闭创造物品栏。
pub fn creative_close_button_system(
    button_query: Query<&Interaction, (Changed<Interaction>, With<CreativeCloseButton>)>,
    mut writer: MessageWriter<UiNavigation>,
) {
    let pressed = button_query
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    if pressed {
        writer.write(UiNavigation::Close(UiScreen::Inventory));
    }
}

/// 同步搜索框占位文字显隐，避免占位文字参与真实搜索。
pub fn sync_creative_search_placeholder_system(
    state: LocalInventory,
    search_state: Res<SearchInputState>,
    mut query: Query<&mut Visibility, With<CreativeSearchPlaceholder>>,
) {
    let Ok(mut visibility) = query.single_mut() else {
        return;
    };

    *visibility = if state.opened && state.creative.search_text.is_empty() && !search_state.active {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

/// 构造创造模式分类数据，并按截图风格固定分类顺序。
pub fn build_creative_categories_system(
    tag_registry: Option<Res<RuntimeTagRegistry>>,
    block_registry: Option<Res<BlockRegistry>>,
    mut state: LocalInventoryMut,
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

    // 这些标签来自数据层；显示顺序固定，避免 HashMap 顺序导致 UI 抖动。
    let mut categories = vec![
        CreativeCategory::virtual_category("全部", "■"),
        category_from_tag("solid", "固体", "▣", &tag_reg, &block_reg),
        category_from_tag("tree_plantable", "作物", "♧", &tag_reg, &block_reg),
        category_from_tag("natural", "自然", "♣", &tag_reg, &block_reg),
    ];

    let mut tools = CreativeCategory::virtual_category("工具", "⚒");
    if let Some(item_reg) = item_registry.as_ref() {
        tools.items = item_reg.items_by_category(&ItemCategory::Tool).to_vec();
    }
    categories.push(tools);

    // 预留与参考图一致的分类入口，后续有对应数据时只需要填充 items。
    categories.push(CreativeCategory::virtual_category("装饰", "▤"));
    categories.push(CreativeCategory::virtual_category("红石", "◆"));
    categories.push(CreativeCategory::virtual_category("运输", "≡"));
    categories.push(CreativeCategory::virtual_category("杂项", "◒"));
    categories.push(CreativeCategory::virtual_category("收藏", "☆"));

    // 追加未显式列出的数据标签，保证新增标签不会被 UI 吞掉。
    let known = [
        "century_journey:solid",
        "century_journey:tree_plantable",
        "century_journey:natural",
    ];
    for tag in tag_reg.all_tags() {
        let tag_full = tag.to_full();
        if known.contains(&tag_full.as_str()) {
            continue;
        }
        categories.push(CreativeCategory::from_tag(
            tag.clone(),
            cat_theme.display_name(&tag_full),
            cat_theme.icon(&tag_full),
            items_for_tag(tag, &tag_reg, &block_reg),
        ));
    }

    state.creative.categories = categories;

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

/// 从指定方块标签生成创造模式分类。
fn category_from_tag(
    path: &str,
    display_name: &str,
    icon: &str,
    tag_registry: &RuntimeTagRegistry,
    block_registry: &BlockRegistry,
) -> CreativeCategory {
    let tag_id = TagId::new("century_journey", path);
    CreativeCategory::from_tag(
        tag_id.clone(),
        display_name.to_string(),
        icon.to_string(),
        items_for_tag(&tag_id, tag_registry, block_registry),
    )
}

/// 为创造物品栏生成局部槽位主题，避免影响 HUD 和生存背包。
fn creative_slot_theme(theme: &UiTheme, slot_size: f32) -> UiTheme {
    let mut theme = theme.clone();
    theme.slot_size = slot_size;
    theme.slot_gap = CREATIVE_SLOT_GAP;
    theme
}

/// 将方块标签里的运行时方块 ID 转换成物品 ID。
fn items_for_tag(
    tag: &TagId,
    tag_registry: &RuntimeTagRegistry,
    block_registry: &BlockRegistry,
) -> Vec<ItemId> {
    tag_registry
        .get_ids(tag)
        .map(|ids| {
            ids.iter()
                .filter_map(|&id| block_registry.get_identifier_by_id(id))
                .map(|ident| ItemId::new(ident.clone()))
                .collect()
        })
        .unwrap_or_default()
}

/// 搜索过滤更新。
pub fn update_creative_filter_system(
    block_registry: Option<Res<BlockRegistry>>,
    item_registry: Option<Res<ItemRegistry>>,
    mut state: LocalInventoryMut,
) {
    let Some(reg) = block_registry else { return };
    if state.creative.categories.is_empty() {
        return;
    }

    let tab = state.creative.selected_tab;
    let search = state.creative.search_text.clone();

    let mut new_items = if tab == 0 {
        // “全部”分类：方块 + 物品，去重后统一展示。
        let mut seen = HashSet::new();
        let mut all: Vec<ItemId> = Vec::new();

        for id in reg.identifiers() {
            if id == "century_journey:air" {
                continue;
            }
            let item_id = ItemId::new(id.clone());
            if seen.insert(item_id.clone()) {
                all.push(item_id);
            }
        }

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

/// 创造模式物品网格填充。
pub fn populate_creative_grid_system(
    state: LocalInventory,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    item_model_assets: Res<ItemModelRenderAssets>,
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

    let revision = item_model_assets.revision();
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
                &item_model_assets,
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
                &item_model_assets,
                &creative_theme,
                &ui_font,
                item_registry.as_deref(),
                item_texture_registry.as_deref(),
            );
        }
    });
}

/// 最近使用面板填充：固定补齐 12 个槽位，保持右侧栏稳定。
pub fn populate_recent_panel_system(
    state: LocalInventory,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    item_model_assets: Res<ItemModelRenderAssets>,
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

    let revision = item_model_assets.revision();
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
                &item_model_assets,
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
                &item_model_assets,
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

/// 创造模式快捷栏。
pub fn init_creative_hotbar_system(
    state: LocalInventory,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    item_model_assets: Res<ItemModelRenderAssets>,
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
        for (index, item) in state.hotbar.items().iter().enumerate() {
            spawn_slot_with_item(
                bar,
                SlotKind::Hotbar,
                index,
                item,
                reg,
                render_assets,
                &item_model_assets,
                &creative_theme,
                &ui_font,
                item_registry.as_deref(),
                item_texture_registry.as_deref(),
            );
        }
    });
}

/// 创造模式快捷栏视觉同步。
pub fn creative_hotbar_visual_sync_system(
    state: LocalInventory,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    item_model_assets: Res<ItemModelRenderAssets>,
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
        &item_model_assets,
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

/// 分类标签高亮。
pub fn update_category_highlight_system(
    state: LocalInventory,
    mut query: Query<(&CategoryTab, &mut UiControl)>,
) {
    for (tab, mut control) in &mut query {
        control.selected = tab.category_index == state.creative.selected_tab;
    }
}
