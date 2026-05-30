use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use crate::ui::components::{HudHotbarContainer, HudHotbarSlot, PacksHotbarSlot, PaletteSlot};
use crate::ui::resources::inventory_ui_state::InventoryUiState;
use crate::voxel::registry::BlockRegistry;
use crate::voxel::types::VoxelType;

pub fn spawn_hotbar_ui_system(mut commands: Commands){
    commands.spawn((
        HudHotbarContainer,
        Node {
            position_type: PositionType::Absolute,
            // 左右居中平衡
            left: Val::Percent(35.0),
            bottom: Val::Px(20.0),
            width: Val::Percent(30.0),
            height: Val::Px(60.0),
            // 网格排列
            display: Display::Grid,
            // 平分 9 格
            grid_template_columns: RepeatedGridTrack::flex(9, 1.0),
            row_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(5.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.6)),
        BorderColor::all(Color::srgb(0.2, 0.2, 0.2)),
    )).with_children(|parent| {
        for index in 0..9 {
            parent.spawn((
                HudHotbarSlot { index },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                BorderColor::all(Color::srgb(0.05, 0.05, 0.05)),
            ));
        }
    });
}

pub fn update_hotbar_ui_system(
    registry: Option<Res<BlockRegistry>>,
    inventory_ui_state: Res<InventoryUiState>,
    mut commands: Commands,
    mut bg_query: Query<&Children>,
    mut hud_slot_query: Query<(Entity, &HudHotbarSlot, &mut BorderColor), Without<PacksHotbarSlot>>,
    mut packs_hotbar_query: Query<(Entity, &PacksHotbarSlot, &mut PaletteSlot, &mut BorderColor)>,
    mut image_node_set: ParamSet<(
        Query<(&mut ImageNode, &mut BackgroundColor, &mut Node)>, // p0: 处理底部 HUD 槽位自身的纹理
        Query<(&mut ImageNode, &mut BackgroundColor, &mut Node)>, // p1: 处理背包槽位内部子节点的纹理
    )>,
) {
    let Some(reg) = registry else { return; };

    for (entity, slot, mut border_color) in &mut hud_slot_query {
        let identifier = &inventory_ui_state.hotbar_items[slot.index];

        if identifier == "century_journey:air" {
            commands.entity(entity).remove::<ImageNode>();
            // 使用 .p0() 安全读取并修改
            if let Ok((_, mut bg, _)) = image_node_set.p0().get_mut(entity) {
                *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3));
            }
        } else if let Some(id) = reg.get_id_by_identifier(identifier) {
            let layer_idx = reg.get_layer(id, 4); // 正面贴图

            // 使用 .p0() 安全读取并修改
            if let Ok((mut img_node, mut bg, _)) = image_node_set.p0().get_mut(entity) {
                *bg = BackgroundColor(Color::NONE);
                img_node.image = reg.base_texture.clone();
                if let Some(ref mut atlas) = img_node.texture_atlas {
                    atlas.index = layer_idx as usize;
                } else {
                    img_node.texture_atlas = Some(TextureAtlas { layout: reg.atlas_layout.clone(), index: layer_idx as usize });
                }
            } else {
                commands.entity(entity).insert((
                    ImageNode {
                        image: reg.base_texture.clone(),
                        texture_atlas: Some(TextureAtlas { layout: reg.atlas_layout.clone(), index: layer_idx as usize }),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ));
            }
        }


        if slot.index == inventory_ui_state.active_hotbar_index {
            *border_color = BorderColor::all(Color::srgb(1.0, 1.0, 1.0));
        } else {
            *border_color = BorderColor::all(Color::srgb(0.05, 0.05, 0.05));
        }
    }

    for (entity, slot, mut palette_slot, mut border_color) in &mut packs_hotbar_query {
        let identifier = &inventory_ui_state.hotbar_items[slot.hotbar_index];

        // 1. 数据同步
        if palette_slot.identifier != *identifier {
            palette_slot.identifier = identifier.clone();
        }

        // 2. 纹理和节点同步
        if let Ok(children) = bg_query.get(entity) {
            for child in children.iter() {
                if identifier == "century_journey:air" {
                    commands.entity(child).remove::<ImageNode>();
                    // 💡 通过 .p1() 访问隔离环境中的修改 Query，规避冲突崩溃
                    if let Ok((_, mut bg, _)) = image_node_set.p1().get_mut(child) {
                        *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3));
                    }
                } else if let Some(id) = reg.get_id_by_identifier(identifier) {
                    let layer_idx = reg.get_layer(id, 4);

                    // 💡 通过 .p1() 访问隔离环境中的修改 Query
                    if let Ok((mut img_node, mut bg, _)) = image_node_set.p1().get_mut(child) {
                        *bg = BackgroundColor(Color::NONE);
                        img_node.image = reg.base_texture.clone();
                        if let Some(ref mut atlas) = img_node.texture_atlas {
                            atlas.index = layer_idx as usize;
                        } else {
                            img_node.texture_atlas = Some(TextureAtlas { layout: reg.atlas_layout.clone(), index: layer_idx as usize });
                        }
                    } else {
                        commands.entity(child).insert((
                            ImageNode {
                                image: reg.base_texture.clone(),
                                texture_atlas: Some(TextureAtlas { layout: reg.atlas_layout.clone(), index: layer_idx as usize }),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                        ));
                    }
                }
            }
        }

        // 3. 选框联动高亮
        if slot.hotbar_index == inventory_ui_state.active_hotbar_index {
            *border_color = BorderColor::all(Color::srgb(0.6, 0.6, 0.6));
        } else {
            *border_color = BorderColor::all(Color::srgb(0.15, 0.15, 0.15));
        }
    }
}

pub fn handle_hotbar_switch_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut inventory_ui_state: ResMut<InventoryUiState>,
) {
    // 背包打开时禁止用滚轮和数字键切格子
    if inventory_ui_state.is_inventory_open { return; }

    // 键盘数字键处理
    let num_keys = [
        (KeyCode::Digit1, 0), (KeyCode::Digit2, 1), (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3), (KeyCode::Digit5, 4), (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6), (KeyCode::Digit8, 7), (KeyCode::Digit9, 8),
    ];

    for (key, idx) in num_keys {
        if keyboard.just_pressed(key) {
            inventory_ui_state.active_hotbar_index = idx;
            return;
        }
    }

    // 鼠标滚轮处理
    for event in mouse_wheel_events.read() {
        let current = inventory_ui_state.active_hotbar_index as i32;
        // 滚轮向上往左移，滚轮向下往右移
        let mut next = if event.y > 0.0 { current - 1 } else { current + 1 };

        if next < 0 { next = 8; }
        if next > 8 { next = 0; }

        inventory_ui_state.active_hotbar_index = next as usize;
    }
}