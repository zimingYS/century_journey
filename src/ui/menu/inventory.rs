use bevy::ecs::event::Trigger;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use crate::core::constant::ui::TOTAL_SLOTS;
use crate::ui::components::{CreativeInventoryMenu, PacksHotbarSlot, PaletteSlot};
use crate::ui::resources;
use crate::ui::resources::inventory_ui_state::InventoryUiState;
use crate::voxel::registry::BlockRegistry;

// 初始化背包 UI 状态
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

/// E键打开物品栏
pub fn toggle_inventory_system(
    mut commands: Commands,
    mut inventory_ui_state: ResMut<InventoryUiState>,
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    registry: Option<Res<BlockRegistry>>,
    menu_query: Query<Entity, With<CreativeInventoryMenu>>,
){
    if !keyboard.just_pressed(KeyCode::KeyE) { return; }
    let Some(reg) = registry else { return; };

    inventory_ui_state.is_inventory_open = !inventory_ui_state.is_inventory_open;

    let Ok(mut cursor_options) = cursor_options_query.single_mut() else { return; };

    if inventory_ui_state.is_inventory_open {
        // 打开背包物品栏
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;

        // 渲染创造模式物品栏
        spawn_creative_menu_ui(&mut commands, &inventory_ui_state, &reg);
    }else {
        // 关闭背包物品栏
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;

        for entity in &menu_query {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_creative_menu_ui(
    commands: &mut Commands,
    inventory_ui_state: &InventoryUiState,
    reg: &BlockRegistry,
) {
    // 外部背景
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
        // 半透明黑灰色背景
        if inventory_ui_state.is_inventory_open { Visibility::Inherited } else { Visibility::Hidden },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.85)),
        BorderColor::all(Color::srgb(0.05, 0.05, 0.05)),
    )).with_children(|parent| {
        // 标题
        parent.spawn((
            Text::new("Creative Inventory"),
            TextFont { font_size: FontSize::Px(20.0), ..default() },
            TextColor(Color::WHITE),
            Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() }
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
            // 生成格子
            for index in 0..TOTAL_SLOTS {
                // 硬射物品栏
                let is_hotbar_row = index >= 27;
                let hotbar_idx = if index >= 27 { index - 27 } else { 0 };

                let identifier = if is_hotbar_row {
                    // 映射到快捷栏的 0..9 索引
                    inventory_ui_state.hotbar_items[hotbar_idx].clone()
                } else {
                    // 常规创造模式物品池
                    // 如果有对应的方块就取出来没有就用空气填充
                    inventory_ui_state
                        .creative_palette
                        .get(index)
                        .cloned()
                        .unwrap_or_else(|| "century_journey:air".to_string())
                };

                let is_air = identifier == "century_journey:air";

                let mut entity_commands =grid.spawn((
                    PaletteSlot { identifier: identifier.clone() },
                    // 让节点能够响应点击
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    Pickable::default(),
                    BackgroundColor(Color::srgb(0.08, 0.08, 0.08)),
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
                    if let Ok(slot) = query.get(trigger.entity) {
                        if let Some(hotbar_idx) = index.checked_sub(27) {
                            // 如果点的是背包底部的快捷栏，则清空该格子
                            ui_state.hotbar_items[hotbar_idx] = "century_journey:air".to_string();
                            info!("快捷栏更新：已清空第 {} 格", hotbar_idx + 1);
                        } else {
                            // 如果点的是上方的创造物品池，则替换到当前选中的快捷栏
                            if slot.identifier == "century_journey:air" { return; }
                            let current_slot_idx = ui_state.active_hotbar_index;
                            ui_state.hotbar_items[current_slot_idx] = slot.identifier.clone();
                            info!("快捷栏更新：已将第 {} 格修改为 -> {}", current_slot_idx + 1, slot.identifier);
                        }
                    }
                })
                .with_children(|slot_node| {
                    if is_air {
                        slot_node.spawn((
                            Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                        ));
                    } else if let Some(id) = reg.get_id_by_identifier(&identifier) {
                        // 非空气方块:利用ImageNode结合TextureAtlas切割出对应的方块正面纹理
                        let layer_idx = reg.get_layer(id, 4); // 4代表正面
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
                });
            }
        });
    });
}
