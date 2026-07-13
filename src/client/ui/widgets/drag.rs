use crate::client::renderer::item_model::{ItemModelRenderAssets, ItemModelRenderer};
use crate::client::ui::resources::ui_font::UiFont;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::inventory::state::InventoryState;
use crate::shared::item_id::ItemId;
use bevy::prelude::*;

/// 鼠标拖拽物品图标根节点标记。
#[derive(Component)]
pub struct CursorItemIcon;

/// 鼠标拖拽物品图片节点标记。
#[derive(Component)]
pub struct CursorItemImage;

/// 鼠标拖拽物品数量文本标记。
#[derive(Component)]
pub struct CursorCountText;

/// 生成鼠标拖拽物品图标实体。
pub fn spawn_cursor_item_icon(mut commands: Commands, ui_font: Res<UiFont>) {
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
                    font: FontSource::from(ui_font.default.clone()),
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

fn cursor_ui_position(cursor_position: Vec2, ui_scale: f32) -> Vec2 {
    const SCREEN_OFFSET: Vec2 = Vec2::splat(12.0);
    (cursor_position + SCREEN_OFFSET) / ui_scale.max(0.01)
}

/// 让拖拽图标跟随鼠标移动。
pub fn cursor_follow_system(
    mut cursor_moved: MessageReader<CursorMoved>,
    ui_scale: Res<UiScale>,
    mut query: Query<&mut Node, With<CursorItemIcon>>,
) {
    for event in cursor_moved.read() {
        let position = cursor_ui_position(event.position, ui_scale.0);
        for mut node in &mut query {
            node.left = Val::Px(position.x);
            node.top = Val::Px(position.y);
        }
    }
}

/// 根据光标物品状态控制拖拽图标显隐。
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

/// 同步拖拽图标图片和数量。
pub fn cursor_texture_system(
    state: Res<InventoryState>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    item_model_assets: Res<ItemModelRenderAssets>,
    cursor_query: Query<&Children, With<CursorItemIcon>>,
    mut image_query: Query<&mut ImageNode, With<CursorItemImage>>,
    mut count_text_query: Query<(&mut Text, &mut Visibility), With<CursorCountText>>,
    mut last_snapshot: Local<Option<(ItemId, u32, u64)>>,
) {
    let current = state
        .cursor
        .stack()
        .map(|s| (s.item.clone(), s.count, item_model_assets.revision()));
    if *last_snapshot == current {
        return;
    }
    *last_snapshot = current.clone();

    for children in &cursor_query {
        for child in children.iter() {
            if let Ok(mut img) = image_query.get_mut(child)
                && let Some((item_id, _count, _revision)) = &current
            {
                if let Some(image) = ItemModelRenderer::item_icon_image(
                    item_id,
                    item_registry.as_deref(),
                    item_texture_registry.as_deref(),
                    &item_model_assets,
                ) {
                    img.image = image;
                    img.texture_atlas = None;
                } else {
                    img.image = Handle::default();
                    img.texture_atlas = None;
                }
            }
        }

        for child in children.iter() {
            if let Ok((mut text, mut vis)) = count_text_query.get_mut(child) {
                if let Some((_, count, _)) = &current {
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

#[cfg(test)]
mod tests {
    use super::cursor_ui_position;
    use bevy::math::Vec2;

    #[test]
    fn cursor_offset_stays_constant_across_ui_scales() {
        let cursor = Vec2::new(900.0, 480.0);
        for scale in [0.67, 1.0, 4.0 / 3.0] {
            let rendered_position = cursor_ui_position(cursor, scale) * scale;
            std::assert!((rendered_position - cursor - Vec2::splat(12.0)).length() < 0.001);
        }
    }
}
