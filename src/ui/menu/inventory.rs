use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use crate::core::constant::ui::TOTAL_SLOTS;
use crate::ui::components::{CreativeInventoryMenu, PacksHotbarSlot, PaletteSlot};
use crate::core::state::inventory_ui_state::InventoryUiState;
use crate::voxel::registry::BlockRegistry;

/// 初始化背包 UI 状态（从 BlockRegistry 提取方块列表）
pub(crate) fn init_inventory_ui_system(
    mut commands: Commands,
    registry: Res<BlockRegistry>,
) {
    let mut ui_state = InventoryUiState::default();

    for identifier in registry.identifier_to_id.keys() {
        if identifier != "century_journey:air" {
            ui_state.creative_palette.push(identifier.clone());
        }
    }
    ui_state.creative_palette.sort();

    commands.insert_resource(ui_state);
}

/// E键切换物品栏开关
pub fn toggle_inventory_system(
    mut commands: Commands,
    mut inventory_ui_state: ResMut<InventoryUiState>,
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    registry: Option<Res<BlockRegistry>>,
    menu_query: Query<Entity, With<CreativeInventoryMenu>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) { return; }
    let Some(reg) = registry else { return; };

    inventory_ui_state.is_inventory_open = !inventory_ui_state.is_inventory_open;

    let Ok(mut cursor_options) = cursor_options_query.single_mut() else { return; };

    if inventory_ui_state.is_inventory_open {
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
        spawn_creative_menu_ui(&mut commands, &inventory_ui_state, &reg);
    } else {
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;

        for entity in &menu_query {
            commands.entity(entity).queue_silenced(|e: EntityWorldMut| { e.despawn(); });
        }
    }
}

/// 构建完整的创造模式物品栏 UI
fn spawn_creative_menu_ui(
    commands: &mut Commands,
    inventory_ui_state: &InventoryUiState,
    reg: &BlockRegistry,
) {
    commands.spawn((
        CreativeInventoryMenu,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(30.0),
            top: Val::Percent(25.0),
            width: Val::Percent(40.0),
            height: Val::Percent(50.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        if inventory_ui_state.is_inventory_open { Visibility::Inherited } else { Visibility::Hidden },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.75)),
        BorderColor::all(Color::srgb(0.2, 0.2, 0.2)),
    )).with_children(|parent| {
        // 标题
        parent.spawn((
            Text::new("Creative Inventory"),
            TextFont { font_size: FontSize::Px(20.0), ..default() },
            TextColor(Color::WHITE),
            Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
        ));

        // 物品网格
        parent.spawn(Node {
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::flex(9, 1.0),
            grid_auto_rows: GridTrack::px(50.0),
            row_gap: Val::Px(4.0),
            width: Val::Percent(100.0),
            ..default()
        }).with_children(|grid| {
            for index in 0..TOTAL_SLOTS {
                let is_hotbar_row = index >= 27;
                let hotbar_idx = if is_hotbar_row { index - 27 } else { 0 };

                let identifier = if is_hotbar_row {
                    inventory_ui_state.hotbar_items[hotbar_idx].clone()
                } else {
                    inventory_ui_state
                        .creative_palette
                        .get(index)
                        .cloned()
                        .unwrap_or_else(|| "century_journey:air".to_string())
                };

                spawn_palette_slot(grid, index, identifier, is_hotbar_row, hotbar_idx, reg);
            }
        });
    });
}

/// 生成单个物品格子（含纹理 + 悬停/点击事件）
fn spawn_palette_slot(
    grid: &mut ChildSpawnerCommands,
    index: usize,
    identifier: String,
    is_hotbar_row: bool,
    hotbar_idx: usize,
    reg: &BlockRegistry,
) {
    let mut entity_commands = grid.spawn((
        PaletteSlot { identifier: identifier.clone() },
        Button,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        Pickable::default(),
        BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.6)),
        BorderColor::all(Color::srgb(0.15, 0.15, 0.15)),
    ));

    if is_hotbar_row {
        entity_commands.insert(PacksHotbarSlot { hotbar_index: hotbar_idx });
    }

    entity_commands
        .observe(|trigger: On<Pointer<Over>>, mut query: Query<&mut BorderColor>| {
            if let Ok(mut border) = query.get_mut(trigger.entity) {
                *border = BorderColor::all(Color::srgb(0.9, 0.9, 0.2));
            }
        })
        .observe(|trigger: On<Pointer<Out>>, mut query: Query<&mut BorderColor>| {
            if let Ok(mut border) = query.get_mut(trigger.entity) {
                *border = BorderColor::all(Color::srgb(0.15, 0.15, 0.15));
            }
        })
        .observe(move |trigger: On<Pointer<Click>>, mut query: Query<&mut PaletteSlot>, mut ui_state: ResMut<InventoryUiState>| {
            if let Ok(slot) = query.get_mut(trigger.entity) {
                if let Some(hotbar_idx) = index.checked_sub(27) {
                    ui_state.hotbar_items[hotbar_idx] = "century_journey:air".to_string();
                    info!("快捷栏更新：已清空第 {} 格", hotbar_idx + 1);
                } else {
                    if slot.identifier == "century_journey:air" { return; }
                    let current_slot_idx = ui_state.active_hotbar_index;
                    ui_state.hotbar_items[current_slot_idx] = slot.identifier.clone();
                    info!("快捷栏更新：已将第 {} 格修改为 -> {}", current_slot_idx + 1, slot.identifier);
                }
            }
        })
        .with_children(|slot_node| {
            spawn_slot_icon(slot_node, &identifier, reg);
        });
}

/// 在格子节点内生成方块图标（空气显示暗色占位，其他显示纹理）
fn spawn_slot_icon(
    slot_node: &mut ChildSpawnerCommands,
    identifier: &str,
    reg: &BlockRegistry,
) {
    if identifier == "century_journey:air" {
        slot_node.spawn((
            Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
        ));
    } else if let Some(id) = reg.get_id_by_identifier(identifier) {
        let layer_idx = reg.get_layer(id, 4);
        slot_node.spawn((
            ImageNode {
                image: reg.base_texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: reg.atlas_layout.clone(),
                    index: layer_idx as usize,
                }),
                ..default()
            },
            Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
        ));
    }
}
