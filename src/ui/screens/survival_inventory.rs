use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use crate::gameplay::gamemode::PlayerGameMode;
use crate::inventory::container::InventoryContainer;
use crate::inventory::item::id::ItemId;
use crate::inventory::state::InventoryState;
use crate::ui::components::{CreativeHotbarPanel, SurvivalInventoryOverlay, SurvivalInventoryRoot, SurvivalItemGrid};
use crate::ui::theme::ui_theme::UiTheme;
use crate::ui::widgets::slot::{spawn_empty_slot, sync_slot_icon, InventorySlot, SearchInputState, SlotKind, SlotVisual};
use crate::voxel::registry::BlockRegistry;

/// 生成生存背包 UI 结构
pub fn spawn_survival_inventory_system(
    mut commands: Commands,
    theme: Res<UiTheme>,
) {
    commands
        .spawn((
            SurvivalInventoryOverlay,
            Name::new("SurvivalOverlay"),
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
                    SurvivalInventoryRoot,
                    Name::new("SurvivalRoot"),
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
                    // 标题栏
                    root.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(theme.panel_header_h),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(Val::Px(theme.panel_padding)),
                            border: UiRect::bottom(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor::all(theme.border_default),
                    )).with_children(|header| {
                        header.spawn((
                            Text::new("生存模式背包"),
                            TextFont {
                                font_size: FontSize::Px(theme.title_font_size),
                                ..default()
                            },
                            TextColor(theme.text_primary),
                        ));
                    });

                    // 背包网格
                    root.spawn((
                        SurvivalItemGrid,
                        Name::new("SurvivalGrid"),
                        Node {
                            flex_grow: 1.0,
                            display: Display::Grid,
                            grid_template_columns: RepeatedGridTrack::flex(9, 1.0),
                            grid_auto_rows: vec![GridTrack::px(
                                theme.slot_size + theme.slot_gap,
                            )],
                            column_gap: Val::Px(theme.slot_gap),
                            row_gap: Val::Px(theme.slot_gap),
                            padding: UiRect::all(Val::Px(theme.grid_padding)),
                            ..default()
                        },
                        BackgroundColor(theme.bg_content),
                    ));

                    build_survival_hotbar_panel(root, &theme);
                });
        });
}

/// 生存背包底部的快捷栏
fn build_survival_hotbar_panel(root: &mut ChildSpawnerCommands, theme: &UiTheme) {
    root.spawn((
        CreativeHotbarPanel,   // 复用同一个组件标记
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

/// 切换背包打开/关闭（E 键 → 路由到此处）
pub fn toggle_survival_inventory_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    search_state: Res<SearchInputState>,
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
    } else {
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
        state.cursor.clear();
    }
}

/// 更新生存背包覆盖层可见性
pub fn update_survival_visibility_system(
    state: Res<InventoryState>,
    gamemode: Res<PlayerGameMode>,
    mut query: Query<&mut Visibility, With<SurvivalInventoryOverlay>>,
) {
    let Ok(mut vis) = query.single_mut() else { return };
    let target = if state.opened && gamemode.is_survival() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    if *vis != target {
        *vis = target;
    }
}

/// 填充生存背包网格（36 格）
pub fn populate_survival_grid_system(
    grid_query: Query<Entity, With<SurvivalItemGrid>>,
    children_query: Query<&Children>,
    existing_slots: Query<&InventorySlot>,
    mut commands: Commands,
    theme: Res<UiTheme>,
) {
    let Ok(grid_entity) = grid_query.single() else { return };

    // 检查是否已有槽位
    let has_slots = children_query
        .get(grid_entity)
        .map(|children| {
            children.iter().any(|child| existing_slots.get(child).is_ok())
        })
        .unwrap_or(false);

    if has_slots {
        return;
    }

    commands.entity(grid_entity).with_children(|grid| {
        for index in 0..36 {
            spawn_empty_slot(grid, SlotKind::SurvivalBackpack, index, &theme);
        }
    });
}

/// 生存背包视觉同步（从 SurvivalInventory 数据同步到 UI）
pub fn survival_grid_visual_sync_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    grid_query: Query<Entity, With<SurvivalItemGrid>>,
    children_query: Query<&Children>,
    slot_query: Query<(Entity, &InventorySlot, &SlotVisual)>,
    mut commands: Commands,
    mut last_snapshot: Local<Vec<ItemId>>,
) {
    let Some(reg) = block_registry.as_ref() else { return };
    let Ok(grid_entity) = grid_query.single() else { return };

    // 构建当前快照
    let current: Vec<ItemId> = (0..36)
        .map(|i| {
            state.survival.get_stack(i)
                .map(|s| s.item.clone())
                .unwrap_or(ItemId::air())
        })
        .collect();

    if *last_snapshot == current {
        return;
    }
    *last_snapshot = current.clone();

    if let Ok(children) = children_query.get(grid_entity) {
        for child in children.iter() {
            if let Ok((entity, slot, visual)) = slot_query.get(child) {
                if slot.kind != SlotKind::SurvivalBackpack {
                    continue;
                }
                let item = current.get(slot.index).cloned().unwrap_or(ItemId::air());
                if visual.item != item {
                    sync_slot_icon(&mut commands, entity, &item, reg, &children_query);
                }
            }
        }
    }
}

/// 生存背包底部快捷栏可视同步
pub fn survival_hotbar_visual_sync_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    slot_query: Query<(Entity, &InventorySlot, &SlotVisual)>,
    children_query: Query<&Children>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    mut border_query: Query<(&InventorySlot, &mut BorderColor)>,
    mut last_hotbar: Local<Vec<ItemId>>,
    mut last_active: Local<usize>,
) {
    let Some(reg) = block_registry.as_ref() else { return };
    let current = state.hotbar.items().to_vec();

    // 图标同步
    if *last_hotbar != current {
        *last_hotbar = current.clone();
        for (entity, slot, visual) in &slot_query {
            if slot.kind != SlotKind::Hotbar { continue; }
            let item = current.get(slot.index).cloned().unwrap_or(ItemId::air());
            if visual.item != item {
                sync_slot_icon(&mut commands, entity, &item, reg, &children_query);
            }
        }
    }

    // 边框高亮 — 仅在 active_index 变化时更新
    if *last_active != state.hotbar.active_index {
        *last_active = state.hotbar.active_index;
        for (slot, mut border) in &mut border_query {
            if slot.kind != SlotKind::Hotbar { continue; }
            *border = BorderColor::all(
                if slot.index == state.hotbar.active_index {
                    theme.border_selected
                } else {
                    theme.border_default
                }
            );
        }
    }
}