use bevy::prelude::*;
use crate::core::constant::world::CHUNK_SIZE;
use crate::inventory::item::id::ItemId;
use crate::inventory::state::InventoryState;
use crate::voxel::registry::BlockRegistry;

/// 鼠标悬浮物品图标标记
#[derive(Component)]
pub struct CursorItemIcon;

/// 生成光标图标实体
pub fn spawn_cursor_item_icon(mut commands: Commands) {
    commands.spawn((
        CursorItemIcon,
        Name::new("CursorItemIcon"),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(48.0),
            height: Val::Px(48.0),
            left: Val::Px(-100.0),
            top: Val::Px(-100.0),
            ..default()
        },
        ZIndex(9999),
        Pickable::IGNORE,
        Visibility::Hidden,
    ));
}

// 光标位置跟随
pub fn cursor_follow_system(
    mut cursor_moved: MessageReader<CursorMoved>,
    mut query: Query<&mut Node, With<CursorItemIcon>>,
) {
    for event in cursor_moved.read() {
        for mut node in &mut query {
            node.left = Val::Px(event.position.x + 12.0);
            node.top = Val::Px(event.position.y - 12.0);
        }
    }
}

// 光标显隐控制
pub fn cursor_visibility_system(
    state: Res<InventoryState>,
    mut query: Query<&mut Visibility, With<CursorItemIcon>>,
) {
    for mut vis in &mut query {
        *vis = if state.cursor.has_item() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

// 光标纹理更新
pub fn cursor_texture_system(
    state: Res<InventoryState>,
    registry: Option<Res<BlockRegistry>>,
    mut query: Query<(Entity, Option<&Children>), With<CursorItemIcon>>,
    children_query: Query<&Children>,
    mut image_query: Query<&mut ImageNode>,
    mut commands: Commands,
    mut last_item: Local<Option<ItemId>>,
) {
    let Some(reg) = registry.as_ref() else { return };
    let current = state.cursor.item().cloned();

    if *last_item == current {
        return;
    }
    *last_item = current.clone();

    let Some(item_id) = current else { return; };
    for (entity, children_opt) in &mut query {
        let Some(block_str) = item_id.as_block_id() else { continue; };
        let Some(id) = reg.get_id_by_identifier(block_str) else { continue; };
        let layer_idx = reg.get_layer(id, 4);
        let index = (layer_idx as usize) * CHUNK_SIZE * CHUNK_SIZE;

        // 已有图标子节点 → 原地更新
        if let Some(children) = children_opt
            .and_then(|_| children_query.get(entity).ok())
        {
            if let Some(&icon_entity) = children.first() {
                if let Ok(mut img) = image_query.get_mut(icon_entity) {
                    img.image = reg.base_texture.clone();
                    if let Some(ref mut atlas) = img.texture_atlas {
                        atlas.index = index;
                    } else {
                        img.texture_atlas = Some(TextureAtlas {
                            layout: reg.atlas_layout.clone(),
                            index,
                        });
                    }
                    continue;
                }
            }
        }

        // 首次创建图标子节点
        let image = reg.base_texture.clone();
        let layout = reg.atlas_layout.clone();
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                ImageNode {
                    image,
                    texture_atlas: Some(TextureAtlas { layout, index }),
                    ..default()
                },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ));
        });
    }
}