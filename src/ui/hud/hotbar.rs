use crate::core::state::inventory_ui_state::InventoryUiState;
use crate::ui::components::{HudHotbarContainer, HudHotbarSlot, PacksHotbarSlot, PaletteSlot};
use crate::voxel::registry::BlockRegistry;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

pub fn spawn_hotbar_ui_system(mut commands: Commands) {
    commands.spawn((
        HudHotbarContainer,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(35.0),
            bottom: Val::Px(20.0),
            width: Val::Percent(30.0),
            height: Val::Px(60.0),
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::flex(9, 1.0),
            row_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(5.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.75)),
        BorderColor::all(Color::srgb(0.2, 0.2, 0.2)),
    )).with_children(|parent| {
        for index in 0..9 {
            parent.spawn((
                HudHotbarSlot { index },
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
            ))
            .observe(|trigger: On<Pointer<Over>>, mut query: Query<&mut BorderColor>| {
                if let Ok(mut border) = query.get_mut(trigger.entity) {
                    *border = BorderColor::all(Color::srgb(0.9, 0.9, 0.2));
                }
            })
            .observe(|trigger: On<Pointer<Out>>, mut query: Query<&mut BorderColor, With<HudHotbarSlot>>| {
                if let Ok(mut border) = query.get_mut(trigger.entity) {
                    *border = BorderColor::all(Color::srgb(0.15, 0.15, 0.15));
                }
            });
        }
    });
}

/// 通用的槽位纹理同步：根据方块 identifier 设置实体的 ImageNode + BackgroundColor
/// 消除 HUD 槽位和背包槽位之间重复的纹理更新逻辑
fn apply_slot_texture(
    commands: &mut Commands,
    entity: Entity,
    identifier: &str,
    registry: &BlockRegistry,
    image_node_query: &mut Query<(&mut ImageNode, &mut BackgroundColor, &mut Node)>,
) {
    if identifier == "century_journey:air" {
        commands.entity(entity)
            .queue_silenced(|mut e: EntityWorldMut| { e.remove::<ImageNode>(); });
        if let Ok((_, mut bg, _)) = image_node_query.get_mut(entity) {
            *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3));
        }
    } else if let Some(id) = registry.get_id_by_identifier(identifier) {
        let layer_idx = registry.get_layer(id, 4);

        if let Ok((mut img_node, mut bg, _)) = image_node_query.get_mut(entity) {
            *bg = BackgroundColor(Color::NONE);
            img_node.image = registry.base_texture.clone();
            if let Some(ref mut atlas) = img_node.texture_atlas {
                atlas.index = layer_idx as usize;
            } else {
                img_node.texture_atlas = Some(TextureAtlas {
                    layout: registry.atlas_layout.clone(),
                    index: layer_idx as usize,
                });
            }
        } else {
            let image = registry.base_texture.clone();
            let layout = registry.atlas_layout.clone();
            let index = layer_idx as usize;
            commands.entity(entity)
                .queue_silenced(move |mut e: EntityWorldMut| {
                    e.insert((
                        ImageNode {
                            image,
                            texture_atlas: Some(TextureAtlas { layout, index }),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ));
                });
        }
    }
}

pub fn update_hotbar_ui_system(
    registry: Option<Res<BlockRegistry>>,
    inventory_ui_state: Res<InventoryUiState>,
    mut commands: Commands,
    bg_query: Query<&Children>,
    mut hud_slot_query: Query<(Entity, &HudHotbarSlot, &mut BorderColor), Without<PacksHotbarSlot>>,
    mut packs_hotbar_query: Query<(Entity, &PacksHotbarSlot, &mut PaletteSlot, &mut BorderColor)>,
    mut image_node_set: ParamSet<(
        Query<(&mut ImageNode, &mut BackgroundColor, &mut Node)>, // p0: HUD 槽位
        Query<(&mut ImageNode, &mut BackgroundColor, &mut Node)>, // p1: 背包槽位子节点
    )>,
) {
    let Some(reg) = registry else { return; };

    // HUD 快捷栏槽位
    for (entity, slot, mut border_color) in &mut hud_slot_query {
        let identifier = &inventory_ui_state.hotbar_items[slot.index];
        apply_slot_texture(&mut commands, entity, identifier, &reg, &mut image_node_set.p0());

        if slot.index == inventory_ui_state.active_hotbar_index {
            *border_color = BorderColor::all(Color::srgb(1.0, 1.0, 1.0));
        } else {
            *border_color = BorderColor::all(Color::srgb(0.15, 0.15, 0.15));
        }
    }

    // 背包物品栏槽位
    for (entity, slot, mut palette_slot, mut border_color) in &mut packs_hotbar_query {
        let identifier = &inventory_ui_state.hotbar_items[slot.hotbar_index];

        // 数据同步
        if palette_slot.identifier != *identifier {
            palette_slot.identifier = identifier.clone();
        }

        // 纹理同步：背包槽位的纹理设置在子节点上
        if let Ok(children) = bg_query.get(entity) {
            for child in children.iter() {
                apply_slot_texture(&mut commands, child, identifier, &reg, &mut image_node_set.p1());
            }
        }

        // 选框联动高亮
        if slot.hotbar_index == inventory_ui_state.active_hotbar_index {
            *border_color = BorderColor::all(Color::srgb(1.0, 1.0, 1.0));
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
    if inventory_ui_state.is_inventory_open { return; }

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

    for event in mouse_wheel_events.read() {
        let current = inventory_ui_state.active_hotbar_index as i32;
        let mut next = if event.y > 0.0 { current - 1 } else { current + 1 };

        if next < 0 { next = 8; }
        if next > 8 { next = 0; }

        inventory_ui_state.active_hotbar_index = next as usize;
    }
}
