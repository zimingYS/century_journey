use bevy::prelude::*;
use crate::core::constant::world::CHUNK_SIZE;
use crate::inventory::item::id::ItemId;
use crate::inventory::state::InventoryState;
use crate::voxel::registry::BlockRegistry;

/// 鼠标悬浮物品图标标记
#[derive(Component)]
pub struct CursorItemIcon;

/// 光标图标固定子节点 — 永远只有一个
#[derive(Component)]
pub struct CursorItemImage;

/// 光标数量文本标记
#[derive(Component)]
pub struct CursorCountText;

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
    )).with_children(|parent| {
        parent.spawn((
            CursorItemImage,
            ImageNode::default(),
            Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
        ));
        parent.spawn((
            CursorCountText,
            Text::new(""),
            TextFont { font_size: FontSize::Px(12.0), ..default() },
            TextColor(Color::WHITE),
            Node { position_type: PositionType::Absolute, bottom: Val::Px(2.0), right: Val::Px(4.0), ..default() },
            Visibility::Hidden,
        ));
    });
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
        *vis = if state.cursor.has_item() { Visibility::Visible } else { Visibility::Hidden };
    }
}

// 光标纹理和数量更新 (只修改已有子节点, 不新建)
pub fn cursor_texture_system(
    state: Res<InventoryState>,
    registry: Option<Res<BlockRegistry>>,
    cursor_query: Query<&Children, With<CursorItemIcon>>,
    mut image_query: Query<&mut ImageNode, With<CursorItemImage>>,
    mut count_text_query: Query<(&mut Text, &mut Visibility), With<CursorCountText>>,
    mut last_snapshot: Local<Option<(ItemId, u32)>>,
) {
    let Some(reg) = registry.as_ref() else { return };
    let current = state.cursor.stack().map(|s| (s.item.clone(), s.count));
    if *last_snapshot == current { return; }
    *last_snapshot = current.clone();

    for children in &cursor_query {
        // ── 更新图标 (第1个子节点 CursorItemImage) ──
        for child in children.iter() {
            if let Ok(mut img) = image_query.get_mut(child) {
                if let Some((item_id, _count)) = &current {
                    if let Some(block_str) = item_id.as_block_id() {
                        if let Some(id) = reg.get_id_by_identifier(block_str) {
                            let layer_idx = reg.get_layer(id, 4);
                            let atlas_idx = (layer_idx as usize) * CHUNK_SIZE * CHUNK_SIZE;
                            img.image = reg.base_texture.clone();
                            if let Some(ref mut atlas) = img.texture_atlas {
                                atlas.index = atlas_idx;
                            } else {
                                img.texture_atlas = Some(TextureAtlas {
                                    layout: reg.atlas_layout.clone(), index: atlas_idx,
                                });
                            }
                        }
                    }
                }
            }
        }

        // ── 更新数量文本 (第2个子节点 CursorCountText) ──
        for child in children.iter() {
            if let Ok((mut text, mut vis)) = count_text_query.get_mut(child) {
                if let Some((_, count)) = &current {
                    if *count > 1 {
                        *vis = Visibility::Inherited;
                        *text = Text::new(count.to_string());
                    } else {
                        *vis = Visibility::Hidden;
                    }
                } else {
                    *vis = Visibility::Hidden;
                }
            }
        }
    }
}
