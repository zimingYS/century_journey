use crate::client::renderer::item::{
    ItemDisplayContext, ItemModelCache, ItemRenderContext, ItemRenderer,
};
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::model::ItemModelRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::world::entity::dropped_item::DroppedItem;
use bevy::prelude::*;

/// 标记掉落物视觉模型已经由统一物品渲染器创建。
#[derive(Component)]
pub struct DroppedItemVisualReady;

/// 为尚未具备视觉子实体的掉落物生成模型。
///
/// Game 只维护掉落物逻辑数据；物品模型、贴图和材质全部在 Client 层解析。
pub fn dropped_item_visual_system(
    mut commands: Commands,
    query: Query<(Entity, &DroppedItem), Without<DroppedItemVisualReady>>,
    item_registry: Option<Res<ItemRegistry>>,
    item_model_registry: Option<Res<ItemModelRegistry>>,
    item_textures: Option<Res<ItemTextureRegistry>>,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut item_model_cache: ResMut<ItemModelCache>,
) {
    let Some(item_textures) = item_textures.as_deref() else {
        return;
    };

    let mut render_context = ItemRenderContext {
        item_registry: item_registry.as_deref(),
        item_model_registry: item_model_registry.as_deref(),
        item_textures,
        block_registry: block_registry.as_deref(),
        block_render_assets: block_render_assets.as_deref(),
        images: &mut images,
        meshes: &mut meshes,
        materials: &mut materials,
        model_cache: &mut item_model_cache,
    };

    for (entity, dropped) in &query {
        if dropped.stack.item.is_air() {
            commands.entity(entity).insert(DroppedItemVisualReady);
            continue;
        }

        let item_key = dropped.stack.item.identifier().to_string();
        let spawned = ItemRenderer::spawn_item_entity(
            &mut commands,
            &dropped.stack.item,
            ItemDisplayContext::Ground,
            Some(entity),
            format!("DroppedItemModel_{item_key}"),
            &mut render_context,
        );

        if spawned.is_some() {
            commands.entity(entity).insert(DroppedItemVisualReady);
        }
    }
}
