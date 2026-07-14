use bevy::prelude::*;

use crate::client::renderer::item::baked_model::BakedItemModel;
use crate::client::renderer::item::cache::ItemModelCache;
use crate::client::renderer::item::mesh_builders::{
    BlockCubeMeshBuilder, CustomItemMeshBuilder, GeneratedItemMeshBuilder,
};
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::model::{ItemModelDefinition, ItemModelKind};
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::shared::identifier::Identifier;

/// 单次模型烘焙需要读取的外部资源。
///
/// Baker 不负责从 ItemId 解析模型，也不持有长期状态；它只把 ItemModelDefinition 转成 BakedItemModel。
pub struct ItemModelBakeContext<'a> {
    /// 方块注册表，用于方块物品模型查询运行时方块 ID 和贴图层。
    pub block_registry: Option<&'a BlockRegistry>,
    /// 方块渲染资源，用于取得 atlas 材质。
    pub block_render_assets: Option<&'a BlockRenderAssets>,
    /// 独立物品贴图注册表，用于 generated item 挤出模型读取像素。
    pub item_textures: &'a ItemTextureRegistry,
    /// Bevy 图片资源表，用于从贴图句柄读取像素数据。
    pub images: &'a Assets<Image>,
}

/// 物品模型烘焙器。
///
/// 它只关心 Definition -> BakedItemModel，不处理 GUI 图标、不处理实体生成，也不决定某个 ItemId 应该用哪个模型。
pub struct ItemModelBaker;

impl ItemModelBaker {
    /// 根据模型定义烘焙运行时模型。
    pub fn bake(
        definition: &ItemModelDefinition,
        context: ItemModelBakeContext<'_>,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        cache: &mut ItemModelCache,
    ) -> Option<BakedItemModel> {
        match &definition.kind {
            ItemModelKind::Empty => Some(BakedItemModel::empty(definition.display.clone())),
            ItemModelKind::Block { block } => {
                bake_block_model(block, definition, context, meshes, cache)
            }
            ItemModelKind::Generated { texture, thickness } => bake_generated_model(
                texture, *thickness, definition, context, meshes, materials, cache,
            ),
            ItemModelKind::Custom { path } => {
                bake_custom_model(path, definition, meshes, materials, cache)
            }
        }
    }
}

/// 烘焙方块物品模型。
fn bake_block_model(
    block: &Identifier,
    definition: &ItemModelDefinition,
    context: ItemModelBakeContext<'_>,
    meshes: &mut Assets<Mesh>,
    cache: &mut ItemModelCache,
) -> Option<BakedItemModel> {
    let block_registry = context.block_registry?;
    let block_render_assets = context.block_render_assets?;
    let block_key = block.to_string();
    let mesh_key = format!("block:{block_key}");

    let mesh_handle = if let Some(handle) = cache.mesh(&mesh_key) {
        handle.clone()
    } else {
        let mesh = BlockCubeMeshBuilder::build_mesh(block_registry, &block_key)?;
        let handle = meshes.add(mesh);
        cache.insert_mesh(mesh_key, handle.clone());
        handle
    };

    let material = BlockCubeMeshBuilder::material(block_registry, block_render_assets, &block_key)?;

    Some(BakedItemModel::single(
        "BlockCube",
        mesh_handle,
        material,
        Transform::default(),
        definition.display.clone(),
    ))
}

/// 烘焙普通物品的挤出模型。
///
/// GUI 不会使用这个结果显示工具和材料；它主要服务第一人称、第三人称、掉落物和展示框。
fn bake_generated_model(
    texture: &Identifier,
    thickness: f32,
    definition: &ItemModelDefinition,
    context: ItemModelBakeContext<'_>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    cache: &mut ItemModelCache,
) -> Option<BakedItemModel> {
    let texture_key = texture.to_string();
    let image_handle = context.item_textures.get_handle(&texture_key).cloned()?;
    let mesh_key = format!("generated:{texture_key}/t={thickness}");

    let mesh_handle = if let Some(handle) = cache.mesh(&mesh_key) {
        handle.clone()
    } else {
        let image = context.images.get(&image_handle)?;
        let mesh = GeneratedItemMeshBuilder::build_mesh(image, thickness);
        let handle = meshes.add(mesh);
        cache.insert_mesh(mesh_key, handle.clone());
        handle
    };

    let material = materials.add(generated_item_material());

    Some(BakedItemModel::single(
        "GeneratedItemMesh",
        mesh_handle,
        material,
        Transform::default(),
        definition.display.clone(),
    ))
}

/// 工具和普通物品在 3D 世界中使用受光材质，GUI 烘焙会另外复制并切换为 unlit。
fn generated_item_material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Opaque,
        cull_mode: None,
        unlit: false,
        perceptual_roughness: 1.0,
        ..default()
    }
}

/// 烘焙自定义模型占位实现。
///
/// 以后接入 glTF 或 JSON model 后，这里会变成真正的 custom model 加载入口。
fn bake_custom_model(
    path: &str,
    definition: &ItemModelDefinition,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    _cache: &mut ItemModelCache,
) -> Option<BakedItemModel> {
    let mesh = CustomItemMeshBuilder::build_mesh(path)?;
    let mesh_handle = meshes.add(mesh);
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.85,
        ..default()
    });

    Some(BakedItemModel::single(
        "CustomItemMesh",
        mesh_handle,
        material,
        Transform::default(),
        definition.display.clone(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requested_fix_generated_item_material_receives_world_lighting() {
        assert!(!generated_item_material().unlit);
    }
}
