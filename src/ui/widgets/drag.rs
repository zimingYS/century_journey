use bevy::prelude::*;
use crate::core::constant::world::CHUNK_SIZE;
use crate::inventory::state::InventoryState;
use crate::voxel::registry::BlockRegistry;

/// 鼠标悬浮物品图标(仅视觉渲染)
#[derive(Component)]
pub struct CursorItemIcon;

/// 生成悬浮图标实体
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

/// 更新悬浮图标的可见性
pub fn update_cursor_icon_system(
    state: Res<InventoryState>,
    registry: Option<Res<BlockRegistry>>,
    mut cursor_moved: MessageReader<CursorMoved>,
    mut query: Query<(Entity, &mut Node, &mut Visibility), With<CursorItemIcon>>,
    mut commands: Commands,
) {
    for (entity, mut node, mut visibility) in &mut query {
        // 显隐
        *visibility = if state.cursor.has_item() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        // 跟随鼠标
        for event in cursor_moved.read() {
            node.left = Val::Px(event.position.x + 12.0);
            node.top = Val::Px(event.position.y - 12.0);
        }

        // 纹理
        let Some(reg) = registry.as_ref() else { continue; };
        let Some(item_id) = state.cursor.item() else { continue; };

        if let Some(block_str) = item_id.as_block_id() {
            if let Some(id) = reg.get_id_by_identifier(block_str) {
                let layer_idx = reg.get_layer(id, 4);
                let image = reg.base_texture.clone();
                let layout = reg.atlas_layout.clone();
                let index = (layer_idx as usize) * CHUNK_SIZE * CHUNK_SIZE;
                commands.entity(entity)
                    .queue_silenced(move |mut e: EntityWorldMut| {
                        e.insert(ImageNode {
                            image,
                            texture_atlas: Some(TextureAtlas { layout, index }),
                            ..default()
                        });
                    });
            }
        }
    }
}
