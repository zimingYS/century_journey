use crate::inventory::container::creative::CreativeCategory;
use crate::inventory::item::id::ItemId;
use crate::inventory::state::InventoryState;
use crate::tag::identifier::TagRegistryType;
use crate::tag::registry::TagRegistry;
use crate::ui::components::{CreativeCategoryPanel, CreativeHotbarPanel, CreativeInventoryOverlay, CreativeInventoryRoot, CreativeItemGrid, CreativeRecentPanel, CreativeSearchBox};
use crate::ui::resources::ui_font::UiFont;
use crate::ui::theme::category_theme::CategoryTheme;
use crate::ui::theme::ui_theme::UiTheme;
use crate::ui::widgets::slot::{CategoryTab, CreativeSearchInput, SlotClickedEvent, SlotKind};
use crate::ui::widgets::tab::spawn_category_tab;
use crate::voxel::registry::BlockRegistry;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

/// 生成创造模式物品栏
pub fn spawn_creative_inventory_system(
    mut commands: Commands,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
) {
    commands.spawn((
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
    )).with_children(|overlay| {
        overlay.spawn((
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
        )).with_children(|root| {
            build_header(root, &ui_font, &theme);
            build_body(root, &ui_font, &theme);
            build_recent_panel(root, &ui_font, &theme);
            build_hotbar_panel(root, &theme);
        });
    });
}

/// 创造模式物品栏头部（包含搜索框）
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
        // 头部标签
        header.spawn((
            Text::new("创造模式"),
            TextFont {
                font: FontSource::from(ui_font.default.clone()),
                font_size: FontSize::Px(theme.title_font_size),
                ..default()
            },
            TextColor(theme.text_primary),
        ));
        // 搜索框
        header.spawn((
            CreativeSearchBox,
            CreativeSearchInput,
            Button,
            Pickable::default(),
            Interaction::default(),
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
            // 搜索框内文本
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

/// 创造模式物品栏中间部分
fn build_body(root: &mut ChildSpawnerCommands, ui_font: &UiFont, theme: &UiTheme) {
    let _ = ui_font;
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            ..default()
        },
    )).with_children(|body| {
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

        // 物品网格
        body.spawn((
            CreativeItemGrid,
            Node {
                flex_grow: 1.0,
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex(theme.grid_columns as u16, 1.0),
                grid_auto_rows: vec![GridTrack::px(theme.slot_size + theme.slot_gap)],
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

/// 创造模式物品栏下侧最近使用
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
    )).with_children(|panel| {
        panel.spawn((
            Text::new("最近使用:"),
            TextFont {
                font: FontSource::from(ui_font.default.clone()),
                font_size: FontSize::Px(theme.small_font_size),
                ..default()
            },
            TextColor(theme.text_secondary),
            Node { margin: UiRect::right(Val::Px(8.0)), ..default() },
        ));
    });
}

/// 创造模式物品栏内的快捷栏
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

/// 切换创造模式物品栏
pub fn toggle_creative_inventory_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<InventoryState>,
    mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) { return; }
    state.toggle();

    let Ok(mut cursor) = cursor_query.single_mut() else { return; };
    if state.opened {
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
    } else {
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
        state.cursor.clear();
    }
}

/// 更新创造模式物品栏可见性
pub fn update_creative_visibility_system(
    state: Res<InventoryState>,
    mut query: Query<&mut Visibility, With<CreativeInventoryOverlay>>,
) {
    let Ok(mut vis) = query.single_mut() else { return; };
    let target = if state.opened { Visibility::Visible } else { Visibility::Hidden };
    if *vis != target {
        *vis = target;
    }
}

/// 构建创造模式物品栏分类栏
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
    let Some(tag_reg) = tag_registry else { return; };
    let Some(block_reg) = block_registry else { return; };
    if !state.creative.categories.is_empty() { return; }

    state.creative.categories.push(
        CreativeCategory::virtual_category("全部", "")
    );

    let tags = tag_reg.all_tags(&TagRegistryType::Block);
    for tag in tags {
        let entries = tag_reg.get_block_tag_entries(tag);
        let items: Vec<ItemId> = entries
            .into_iter()
            .filter(|id| block_reg.get_id_by_identifier(id).is_some())
            .map(ItemId::block)
            .collect();

        let display_name = cat_theme.display_name(&tag.to_full());
        let icon = cat_theme.icon(&tag.to_full());

        state.creative.categories.push(
            CreativeCategory::from_tag(
                tag.clone(), display_name, icon, items,
            )
        );
    }

    state.creative.categories.push(
        CreativeCategory::virtual_category("收藏", "")
    );

    let Ok(panel_entity) = category_panel.single() else { return; };
    commands.entity(panel_entity).with_children(|panel| {
        for (idx, cat) in state.creative.categories.iter().enumerate() {
            let is_active = idx == state.creative.selected_tab;
            spawn_category_tab(panel, &cat.display_name, &cat.icon, idx, is_active, &ui_font, &theme);
        }
    });
}

/// 搜索分类更新
pub fn update_creative_filter_system(
    block_registry: Option<Res<BlockRegistry>>,
    mut state: ResMut<InventoryState>,
) {
    let Some(reg) = block_registry else { return; };
    if state.creative.categories.is_empty() { return; }

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

/// 根据visible_items填充物品网格
pub fn populate_creative_grid_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    grid_query: Query<Entity, With<CreativeItemGrid>>,
    children_query: Query<&Children>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    mut last_items: Local<Vec<ItemId>>,
) {
    let Some(reg) = block_registry else {
        return;
    };

    let Ok(grid_entity) = grid_query.single() else {
        return;
    };

    if *last_items == state.creative.visible_items {
        return;
    }

    *last_items = state.creative.visible_items.clone();

    // 清空旧内容
    if let Ok(children) = children_query.get(grid_entity) {
        for child in children.iter() {
            commands.entity(child).despawn_related::<Children>();
            commands.entity(child).despawn();
        }
    }

    let items = state.creative.visible_items.clone();

    commands.entity(grid_entity).with_children(|grid| {
        for (index, item) in items.iter().enumerate() {

            crate::ui::widgets::slot::spawn_slot_with_item(
                grid,
                SlotKind::CreativeGrid,
                index,
                item,
                reg.as_ref(),
                &theme,
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
    mut commands: Commands,
    ui_font: Res<UiFont>,
    theme: Res<UiTheme>,
    mut last_items: Local<Vec<ItemId>>,
) {
    let Some(reg) = block_registry else {
        return;
    };

    let Ok(panel_entity) = recent_query.single() else {
        return;
    };

    if *last_items == state.recent.items {
        return;
    }

    *last_items = state.recent.items.clone();

    if let Ok(children) = children_query.get(panel_entity) {
        for child in children.iter() {
            commands.entity(child).despawn_related::<Children>();
            commands.entity(child).despawn();
        }
    }

    let recent_items = state.recent.items.clone();

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

        for (index, item) in recent_items.iter().enumerate() {

            crate::ui::widgets::slot::spawn_slot_with_item(
                panel,
                SlotKind::Recent,
                index,
                item,
                reg.as_ref(),
                &theme,
            );
        }
    });
}

/// 创造物品栏快捷栏纹理同步
pub fn update_creative_hotbar_display_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    hotbar_query: Query<Entity, With<CreativeHotbarPanel>>,
    children_query: Query<&Children>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    mut last_hotbar: Local<Vec<ItemId>>,
) {
    let Some(reg) = block_registry else {
        return;
    };

    let current =
        state.hotbar.items.to_vec();

    // if *last_hotbar == current {
    //     return;
    // }

    *last_hotbar = current.clone();

    let Ok(panel_entity) = hotbar_query.single() else {
        return;
    };

    if let Ok(children) = children_query.get(panel_entity) {
        for child in children.iter() {
            commands.entity(child).despawn_related::<Children>();
            commands.entity(child).despawn();
        }
    }

    commands.entity(panel_entity).with_children(|bar| {

        for (index, item) in current.iter().enumerate() {

            crate::ui::widgets::slot::spawn_slot_with_item(
                bar,
                SlotKind::Hotbar,
                index,
                item,
                reg.as_ref(),
                &theme,
            );
        }
    });
}

/// 更新类别高亮显示系统
pub fn update_category_highlight_system(
    state: Res<InventoryState>,
    theme: Res<UiTheme>,
    mut query: Query<(&CategoryTab, &mut BackgroundColor)>,
) {
    for (tab, mut bg) in &mut query {
        *bg = BackgroundColor(
            if tab.category_index == state.creative.selected_tab {
                theme.tab_active_bg
            } else {
                Color::NONE
            }
        );
    }
}

// pub fn slot_hover_system(
//     mut over_events: MessageReader<Pointer<Over>>,
//     mut out_events: MessageReader<Pointer<Out>>,
//     slot_query: Query<&InventorySlot>,
//     state: Res<InventoryState>,
//     theme: Res<UiTheme>,
//     mut border_query: Query<&mut BorderColor, With<InventorySlot>>,
// ) {
//     for event in over_events.read() {
//         let Ok(mut border) = border_query.get_mut(event.entity) else { continue; };
//         *border = BorderColor::all(theme.border_hover);
//     }
//
//     for event in out_events.read() {
//         let Ok(slot) = slot_query.get(event.entity) else { continue; };
//         let Ok(mut border) = border_query.get_mut(event.entity) else { continue; };
//         let is_selected = slot.kind == SlotKind::Hotbar
//             && slot.index == state.hotbar.active_index;
//         *border = BorderColor::all(
//             if is_selected { theme.border_selected } else { theme.border_default }
//         );
//     }
// }

/// 创造模式点击系统
pub fn handle_creative_click_system(
    mut reader: MessageReader<SlotClickedEvent>,
    mut state: ResMut<InventoryState>,
) {
    for ev in reader.read() {
        if ev.kind != SlotKind::CreativeGrid {
            continue;
        }

        let Some(item) =
            state.creative.visible_items.get(ev.index).cloned()
        else {
            continue;
        };

        let slot = state.hotbar.active_index;
        state.hotbar.items[slot] = item.clone();
        state.add_recent(item);
    }
}