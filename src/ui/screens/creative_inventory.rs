use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::inventory::container::creative::CreativeCategory;
use crate::inventory::container::hotbar::HOTBAR_SIZE;
use crate::inventory::item::id::ItemId;
use crate::inventory::item::stack::ItemStack;
use crate::inventory::state::InventoryState;
use crate::tag::identifier::TagRegistryType;
use crate::tag::registry::TagRegistry;
use crate::ui::components::{
    CreativeCategoryPanel, CreativeHotbarPanel, CreativeInventoryOverlay,
    CreativeInventoryRoot, CreativeItemGrid, CreativeRecentPanel, CreativeSearchBox,
};
use crate::ui::resources::ui_font::UiFont;
use crate::ui::theme::category_theme::CategoryTheme;
use crate::ui::theme::ui_theme::UiTheme;
use crate::ui::widgets::slot::{spawn_slot_with_item, sync_slot_icon, CategoryTab, CreativeSearchInput, InventorySlot, SearchInputState, SlotKind, SlotVisual};
use crate::ui::widgets::tab::spawn_category_tab;
use crate::voxel::registry::BlockRegistry;

/// 构造创造模式物品栏UI
pub fn spawn_creative_inventory_system(
    mut commands: Commands,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
) {
    commands
        .spawn((
            CreativeInventoryOverlay,
            Name::new("CreativeOverlay"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            Visibility::Hidden,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    CreativeInventoryRoot,
                    Name::new("CreativeRoot"),
                    Node {
                        width: Val::Px(theme.panel_width),
                        height: Val::Px(theme.panel_height),
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(theme.bg_panel),
                    BorderColor::all(theme.border_default),
                ))
                .with_children(|root| {
                    build_header(root, &ui_font, &theme);
                    build_body(root, &ui_font, &theme);
                    build_recent_panel(root, &ui_font, &theme);
                    build_hotbar_panel(root, &theme);
                });
        });
}
/// 创造模式物品栏UI一部分 创建标题和搜索框
fn build_header(root: &mut ChildSpawnerCommands, ui_font: &UiFont, theme: &UiTheme) {
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(theme.panel_header_h),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(theme.panel_padding)),
            border: UiRect::bottom(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(theme.border_default),
    )).with_children(|header| {
        // 标题
        header.spawn((
            Text::new("创造模式"),
            TextFont {
                font: FontSource::from(ui_font.default.clone()),
                font_size: FontSize::Px(theme.title_font_size),
                ..default()
            },
            TextColor(theme.text_primary),
        ));
        header
            .spawn((
                CreativeSearchBox,
                CreativeSearchInput,
                Button,
                Pickable::default(),
                Node {
                    width: Val::Px(theme.search_width),
                    height: Val::Px(theme.search_height),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(theme.search_bg),
                BorderColor::all(theme.search_border),
            )).with_children(|s| {
            // 搜索框
                s.spawn((
                    Text::new(""),
                    TextFont {
                        font: FontSource::from(ui_font.default.clone()),
                        font_size: FontSize::Px(theme.search_font_size),
                        ..default()
                    },
                    TextColor(theme.text_hint),
                ));
            });
        });
}
/// 创造模式物品栏UI一部分 创建中部标签栏和物品排列显示
fn build_body(root: &mut ChildSpawnerCommands, _ui_font: &UiFont, theme: &UiTheme) {
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            ..default()
        },
    ))
        .with_children(|body| {
            // 左边标签
            body.spawn((
                CreativeCategoryPanel,
                Node {
                    width: Val::Px(theme.tab_sidebar_width),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::right(Val::Px(1.0)),
                    padding: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(theme.bg_sidebar),
                BorderColor::all(theme.border_default),
            ));

            body.spawn((
                // 右边物品网格
                CreativeItemGrid,
                Node {
                    flex_grow: 1.0,
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::flex(
                        theme.grid_columns as u16,
                        1.0,
                    ),
                    grid_auto_rows: vec![GridTrack::px(
                        theme.slot_size + theme.slot_gap,
                    )],
                    column_gap: Val::Px(theme.slot_gap),
                    row_gap: Val::Px(theme.slot_gap),
                    padding: UiRect::all(Val::Px(theme.grid_padding)),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(theme.bg_content),
            ));
        });
}
/// 创造模式物品栏UI一部分 底部最近使用
fn build_recent_panel(root: &mut ChildSpawnerCommands, ui_font: &UiFont, theme: &UiTheme) {
    root.spawn((
        CreativeRecentPanel,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(theme.recent_height),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(theme.panel_padding)),
            border: UiRect::top(Val::Px(1.0)),
            column_gap: Val::Px(theme.slot_gap),
            ..default()
        },
        BorderColor::all(theme.border_default),
    ))
        .with_children(|panel| {
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
        });
}
/// 创造模式物品栏UI一部分 最底部快捷栏
fn build_hotbar_panel(root: &mut ChildSpawnerCommands, theme: &UiTheme) {
    root.spawn((
        CreativeHotbarPanel,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(theme.creative_hotbar_h),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::top(Val::Px(1.0)),
            padding: UiRect::vertical(Val::Px(6.0)),
            column_gap: Val::Px(4.0),
            ..default()
        },
        BorderColor::all(theme.border_default),
    ));
}

/// 切换物品栏状态
pub fn toggle_inventory_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    search_state: Res<SearchInputState>,
    gamemode: Res<crate::gameplay::gamemode::PlayerGameMode>,
    mut state: ResMut<InventoryState>,
    mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) { return; }
    if search_state.active { return; }

    state.toggle();

    let Ok(mut cursor) = cursor_query.single_mut() else { return };
    if state.opened {
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
        info!("Opened inventory in {:?} mode", gamemode.mode);
    } else {
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
        state.cursor.clear();
    }
}

pub fn update_creative_visibility_system(
    state: Res<InventoryState>,
    gamemode: Res<crate::gameplay::gamemode::PlayerGameMode>,
    mut query: Query<&mut Visibility, With<CreativeInventoryOverlay>>,
) {
    let Ok(mut vis) = query.single_mut() else { return };
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
    tag_registry: Option<Res<TagRegistry>>,
    block_registry: Option<Res<BlockRegistry>>,
    mut state: ResMut<InventoryState>,
    category_panel: Query<Entity, With<CreativeCategoryPanel>>,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
    cat_theme: Res<CategoryTheme>,
    mut commands: Commands,
) {
    let Some(tag_reg) = tag_registry else { return };
    let Some(block_reg) = block_registry else { return };
    if !state.creative.categories.is_empty() {
        return;
    }

    state
        .creative
        .categories
        .push(CreativeCategory::virtual_category("全部", ""));

    let tags = tag_reg.all_tags(&TagRegistryType::Block);
    for tag in tags {
        let entries = tag_reg.get_block_tag_entries(tag);
        let items: Vec<ItemId> = entries
            .into_iter()
            .filter(|id| block_reg.get_id_by_identifier(id).is_some())
            .map(ItemId::block)
            .collect();

        state.creative.categories.push(CreativeCategory::from_tag(
            tag.clone(),
            cat_theme.display_name(&tag.to_full()),
            cat_theme.icon(&tag.to_full()),
            items,
        ));
    }

    state
        .creative
        .categories
        .push(CreativeCategory::virtual_category("收藏", ""));

    let Ok(panel_entity) = category_panel.single() else { return };
    commands.entity(panel_entity).with_children(|panel| {
        for (idx, cat) in state.creative.categories.iter().enumerate() {
            spawn_category_tab(
                panel, &cat.display_name, &cat.icon, idx,
                idx == state.creative.selected_tab, &ui_font, &theme,
            );
        }
    });
}

/// 搜索过滤更新
pub fn update_creative_filter_system(
    block_registry: Option<Res<BlockRegistry>>,
    mut state: ResMut<InventoryState>,
) {
    let Some(reg) = block_registry else { return };
    if state.creative.categories.is_empty() {
        return;
    }

    let tab = state.creative.selected_tab;
    let search = state.creative.search_text.clone();

    let mut new_items = if tab == 0 {
        reg.identifier_to_id
            .keys()
            .filter(|id| *id != "century_journey:air")
            .map(|id| ItemId::block(id.as_str()))
            .collect::<Vec<_>>()
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
    mut commands: Commands,
    theme: Res<UiTheme>,
    mut last_items: Local<Vec<ItemId>>,
) {
    let Some(reg) = block_registry.as_ref() else { return };
    let Ok(grid_entity) = grid_query.single() else { return };

    if *last_items == state.creative.visible_items {
        return;
    }
    *last_items = state.creative.visible_items.clone();

    // 收集现有 CreativeGrid 槽位
    let mut slot_indices: Vec<(Entity, usize)> = Vec::new();
    if let Ok(children) = children_query.get(grid_entity) {
        for child in children.iter() {
            if let Ok(slot) = existing_slots.get(child) {
                if slot.1.kind == SlotKind::CreativeGrid {
                    slot_indices.push((child, slot.1.index));
                }
            }
        }
    }

    let new_items = &state.creative.visible_items;

    // 数量匹配 → 原地更新图标
    if slot_indices.len() == new_items.len() {
        for (entity, idx) in slot_indices {
            let air = &ItemId::air();
            let item = new_items.get(idx).unwrap_or(air);
            sync_slot_icon(&mut commands, entity, item, 0, reg, &children_query);
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
            spawn_slot_with_item(grid, SlotKind::CreativeGrid, index, item, reg, &theme);
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
    mut commands: Commands,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
    mut last_items: Local<Vec<ItemStack>>,
) {
    let Some(reg) = block_registry.as_ref() else { return };
    let Ok(panel_entity) = recent_query.single() else { return };

    if *last_items == state.recent.items {
        return;
    }
    *last_items = state.recent.items.clone();

    // 收集现有 Recent 槽位（跳过第一子节点 "最近使用:" 文本）
    let mut slot_entities: Vec<(Entity, usize)> = Vec::new();
    if let Ok(children) = children_query.get(panel_entity) {
        for child in children.iter().skip(1) {
            // skip label
            if let Ok(slot) = existing_slots.get(child) {
                if slot.1.kind == SlotKind::Recent {
                    slot_entities.push((child, slot.1.index));
                }
            }
        }
    }

    let new_items = &state.recent.items;

    if slot_entities.len() == new_items.len() {
        for (entity, idx) in slot_entities {
            let air = ItemId::air();
            let (item, count) = new_items.get(idx).map(|s| (&s.item, s.count)).unwrap_or((&air, 0u32));
            sync_slot_icon(&mut commands, entity, item, count, reg, &children_query);
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
            spawn_slot_with_item(panel, SlotKind::Recent, index, &stack.item, reg, &theme);
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
    mut commands: Commands,
    theme: Res<UiTheme>,
) {
    let Some(reg) = block_registry.as_ref() else { return };
    let Ok(panel_entity) = hotbar_query.single() else { return };

    let has_hotbar_slots = children_query
        .get(panel_entity)
        .map(|children| {
            children.iter().any(|child| {
                slot_query.get(child).map_or(false, |s| s.kind == SlotKind::Hotbar)
            })
        })
        .unwrap_or(false);

    if has_hotbar_slots {
        return;
    }

    commands.entity(panel_entity).with_children(|bar| {
        for (index, item) in state.hotbar.items().iter().enumerate() {
            spawn_slot_with_item(bar, SlotKind::Hotbar, index, item, reg, &theme);
        }
    });
}

/// 创造模式快捷栏视觉同步
pub fn creative_hotbar_visual_sync_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    hotbar_query: Query<Entity, With<CreativeHotbarPanel>>,
    children_query: Query<&Children>,
    slot_query: Query<(Entity, &InventorySlot, &SlotVisual)>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    mut border_query: Query<(&InventorySlot, &mut BorderColor)>,
    mut last_hotbar: Local<Vec<(ItemId, u32)>>,
) {
    let Some(reg) = block_registry.as_ref() else { return };

    let current: Vec<(ItemId, u32)> = (0..HOTBAR_SIZE)
        .map(|i| {
            state.hotbar.get_stack(i)
                .map(|s| (s.item.clone(), s.count))
                .unwrap_or((ItemId::air(), 0))
        })
        .collect();

    if *last_hotbar == current {
        return;
    }
    *last_hotbar = current.clone();

    // 原地更新图标
    let Ok(hotbar_entity) = hotbar_query.single() else { return };
    if let Ok(children) = children_query.get(hotbar_entity) {
        for child in children.iter() {
            if let Ok((entity, slot, visual)) = slot_query.get(child) {
                if slot.kind != SlotKind::Hotbar {
                    continue;
                }
                let (item, count) = current.get(slot.index).cloned().unwrap_or((ItemId::air(), 0));
                if visual.item != item || visual.count != count {
                    sync_slot_icon(&mut commands, entity, &item, count, reg, &children_query);
                }
            }
        }
    }

    // 更新选中边框
    for (slot, mut border) in &mut border_query {
        if slot.kind != SlotKind::Hotbar {
            continue;
        }
        *border = BorderColor::all(if slot.index == state.hotbar.active_index {
            theme.border_selected
        } else {
            theme.border_default
        });
    }
}

/// 关闭物品栏时清理创造模式快捷栏子实体
pub fn cleanup_creative_hotbar_system(
    state: Res<InventoryState>,
    hotbar_query: Query<Entity, With<CreativeHotbarPanel>>,
    children_query: Query<&Children>,
    mut commands: Commands,
    mut was_opened: Local<bool>,
) {
    if *was_opened && !state.opened {
        if let Ok(panel_entity) = hotbar_query.single() {
            if let Ok(children) = children_query.get(panel_entity) {
                for child in children.iter() {
                    commands.entity(child).despawn();
                }
            }
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