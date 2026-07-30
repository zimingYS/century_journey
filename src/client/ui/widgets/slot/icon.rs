//! 解析物品 GUI 图标，并同步槽位的图片与数量文本。

use bevy::prelude::*;

use super::components::SlotIcon;
use crate::client::renderer::constants::BLOCK_ATLAS_TILES_PER_LAYER;
use crate::client::renderer::item_model::{ItemModelRenderAssets, ItemModelRenderer};
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::shared::item_id::ItemId;
/// 生成槽位图标子节点。
///
/// UI 层不判断方块或贴图类型，只向 ItemModelRenderer 查询 GUI 所需图片。
/// 当 3D 方块图标仍在离屏烘焙时，临时回退到方块 atlas 图标，避免空槽位。
pub(super) fn spawn_icon_child(
    parent: &mut ChildSpawnerCommands,
    item: &ItemId,
    block_registry: &BlockRegistry,
    render_assets: &BlockRenderAssets,
    item_model_assets: &ItemModelRenderAssets,
    item_registry: Option<&ItemRegistry>,
    item_texture_registry: Option<&ItemTextureRegistry>,
) {
    if let Some(image) = ItemModelRenderer::item_icon_image(
        item,
        item_registry,
        item_texture_registry,
        item_model_assets,
    ) {
        parent.spawn((SlotIcon, plain_image_node(image), icon_node()));
    } else if let Some(image_node) =
        block_atlas_fallback_image(item, block_registry, render_assets, item_registry)
    {
        parent.spawn((SlotIcon, image_node, icon_node()));
    } else {
        parent.spawn((SlotIcon, icon_node(), Visibility::Hidden));
    }
}

/// 原地同步槽位图标和数量文本。
/// 单个槽位图标可走模型、图标或方块图集三条降级路径，依赖保持显式。
#[allow(clippy::too_many_arguments)]
pub fn sync_slot_icon(
    commands: &mut Commands,
    slot_entity: Entity,
    item: &ItemId,
    count: u32,
    block_registry: &BlockRegistry,
    render_assets: &BlockRenderAssets,
    item_model_assets: &ItemModelRenderAssets,
    children_query: &Query<&Children>,
    item_registry: Option<&ItemRegistry>,
    item_texture_registry: Option<&ItemTextureRegistry>,
) {
    let Ok(children) = children_query.get(slot_entity) else {
        return;
    };

    if let Some(&icon_entity) = children.first() {
        if item.is_air() {
            commands.entity(icon_entity).insert(Visibility::Hidden);
        } else if let Some(image) = ItemModelRenderer::item_icon_image(
            item,
            item_registry,
            item_texture_registry,
            item_model_assets,
        ) {
            commands
                .entity(icon_entity)
                .insert((Visibility::Inherited, plain_image_node(image)));
        } else if let Some(image_node) =
            block_atlas_fallback_image(item, block_registry, render_assets, item_registry)
        {
            commands
                .entity(icon_entity)
                .insert((Visibility::Inherited, image_node));
        } else {
            commands.entity(icon_entity).insert(Visibility::Hidden);
        }
    }

    if let Some(&count_entity) = children.get(1) {
        if count > 1 {
            commands
                .entity(count_entity)
                .insert((Visibility::Inherited, Text::new(count.to_string())));
        } else {
            commands.entity(count_entity).insert(Visibility::Hidden);
        }
    }

    if let Some(&placeholder_entity) = children.get(2) {
        commands
            .entity(placeholder_entity)
            .insert(if item.is_air() {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
    }
}

/// 创建统一尺寸的槽位图标节点。
pub(super) fn icon_node() -> Node {
    Node {
        width: Val::Percent(80.0),
        height: Val::Percent(80.0),
        ..default()
    }
}

/// 创建普通图片节点。
fn plain_image_node(image: Handle<Image>) -> ImageNode {
    ImageNode {
        image,
        texture_atlas: None,
        ..default()
    }
}

/// 当 3D 方块图标尚未就绪时，回退到方块 atlas 中的 2D 图标。
fn block_atlas_fallback_image(
    item: &ItemId,
    block_registry: &BlockRegistry,
    render_assets: &BlockRenderAssets,
    item_registry: Option<&ItemRegistry>,
) -> Option<ImageNode> {
    let block_id = item_registry
        .and_then(|registry| registry.get(item))
        .and_then(|definition| {
            definition
                .placeable_block
                .as_ref()
                .or_else(|| definition.icon.as_block_id())
        })
        .cloned()
        .unwrap_or_else(|| item.identifier().clone());

    let runtime_id = block_registry.get_id(&block_id)?;
    let layer = block_registry.get_layer(runtime_id, 4) as usize;
    let atlas_index = layer * BLOCK_ATLAS_TILES_PER_LAYER;
    Some(ImageNode {
        image: render_assets.base_texture().clone(),
        texture_atlas: Some(TextureAtlas {
            layout: render_assets.atlas_layout().clone(),
            index: atlas_index,
        }),
        ..default()
    })
}
