use crate::client::ui::widgets::slot::resolve_item_icon;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::icon::IconDefinition;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::inventory::state::InventoryState;
use crate::shared::item_id::ItemId;
use bevy::prelude::*;

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
    commands
        .spawn((
            CursorItemIcon,
            Name::new("CursorItemIcon"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(64.0),
                height: Val::Px(64.0),
                left: Val::Px(-100.0),
                top: Val::Px(-100.0),
                ..default()
            },
            ZIndex(9999),
            Pickable::IGNORE,
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                CursorItemImage,
                ImageNode::default(),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ));
            parent.spawn((
                CursorCountText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(2.0),
                    right: Val::Px(4.0),
                    ..default()
                },
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
        *vis = if state.cursor.has_item() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

// 光标纹理和数量更新 (只修改已有子节点, 不新建)
pub fn cursor_texture_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    cursor_query: Query<&Children, With<CursorItemIcon>>,
    mut image_query: Query<&mut ImageNode, With<CursorItemImage>>,
    mut count_text_query: Query<(&mut Text, &mut Visibility), With<CursorCountText>>,
    mut last_snapshot: Local<Option<(ItemId, u32)>>,
) {
    let Some(block_reg) = block_registry.as_ref() else {
        return;
    };
    let current = state.cursor.stack().map(|s| (s.item.clone(), s.count));
    if *last_snapshot == current {
        return;
    }
    *last_snapshot = current.clone();

    for children in &cursor_query {
        // 更新图标
        for child in children.iter() {
            if let Ok(mut img) = image_query.get_mut(child) {
                if let Some((item_id, _count)) = &current {
                    let icon_def = resolve_item_icon(item_id, item_registry.as_deref());

                    if let Some(icon) = icon_def {
                        match icon {
                            IconDefinition::Block(id) => {
                                if let Some(atlas_idx) = block_reg.get_icon_atlas_index(&id) {
                                    img.image = block_reg.base_texture().clone();
                                    img.texture_atlas = Some(TextureAtlas {
                                        layout: block_reg.atlas_layout().clone(),
                                        index: atlas_idx,
                                    });
                                }
                            }
                            IconDefinition::Texture(path) => {
                                if let Some(ireg) = item_texture_registry.as_ref() {
                                    if let Some(handle) = ireg.get_handle(&path) {
                                        img.image = handle.clone();
                                        img.texture_atlas = None;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 更新数量文本
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
