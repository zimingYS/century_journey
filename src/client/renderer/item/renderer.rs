use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use crate::client::renderer::item::baked_model::BakedItemModel;
use crate::client::renderer::item::baker::{ItemModelBakeContext, ItemModelBaker};
use crate::client::renderer::item::cache::ItemModelCache;
use crate::client::renderer::item::display::ItemDisplayContext;
use crate::client::renderer::item::gui_icon_baker::GuiItemIconBaker;
use crate::client::renderer::item::gui_icon_cache::GuiItemIconCache;
use crate::client::renderer::item::resolver::ItemModelResolver;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::definition::ItemCategory;
use crate::content::item::model::ItemModelRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::icon::IconDefinition;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::shared::identifier::Identifier;
use crate::shared::item_id::ItemId;

/// 一次物品渲染所需的资源视图。
///
/// 调用方只负责提供当前世界里的注册表、资源表和缓存；具体使用方块模型、挤出模型还是原始贴图，全部由 ItemRenderer 统一决定。
pub struct ItemRenderContext<'a> {
    /// 物品定义注册表，用于从 ItemId 查到分类、图标和显式模型引用。
    pub item_registry: Option<&'a ItemRegistry>,
    /// 物品模型定义注册表，用于覆盖默认推导模型。
    pub item_model_registry: Option<&'a ItemModelRegistry>,
    /// 独立物品贴图注册表，GUI 中的普通物品直接从这里取 2D 图。
    pub item_textures: &'a ItemTextureRegistry,
    /// 方块注册表，方块物品烘焙 3D cube 时需要查询贴图层。
    pub block_registry: Option<&'a BlockRegistry>,
    /// 方块 atlas 和材质资源。
    pub block_render_assets: Option<&'a BlockRenderAssets>,
    /// Bevy 图片资源表，GUI 3D 图标会创建渲染目标图片。
    pub images: &'a mut Assets<Image>,
    /// Bevy 网格资源表，Baker 会把 Mesh 放到这里。
    pub meshes: &'a mut Assets<Mesh>,
    /// Bevy 材质资源表，Baker 和 GUI 图标会创建材质。
    pub materials: &'a mut Assets<StandardMaterial>,
    /// 物品模型缓存，避免重复烘焙同一个模型。
    pub model_cache: &'a mut ItemModelCache,
}

/// 物品实体生成后的根节点和所有 mesh 子节点。
pub struct SpawnedItemEntity {
    /// 承载 display transform 的根实体。
    pub root: Entity,
    /// 实际挂载 Mesh3d / Material 的部件实体。
    pub parts: Vec<Entity>,
}

impl SpawnedItemEntity {
    /// 遍历根实体和所有部件实体，方便调用方统一追加标记组件。
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        std::iter::once(self.root).chain(self.parts.iter().copied())
    }
}

/// 物品渲染统一入口。
///
/// 外部只传 ItemId 和显示场景；这里负责解析模型、烘焙模型、生成实体或返回 GUI 图标。
pub struct ItemRenderer;

impl ItemRenderer {
    /// 在 3D 场景中生成一个物品实体。
    pub fn spawn_item_entity(
        commands: &mut Commands,
        item: &ItemId,
        context: ItemDisplayContext,
        parent: Option<Entity>,
        name: impl Into<String>,
        render_context: &mut ItemRenderContext<'_>,
    ) -> Option<SpawnedItemEntity> {
        let model = Self::baked_model(item, render_context)?;
        Some(Self::spawn_baked_entity(
            commands, &model, context, parent, name,
        ))
    }

    /// 确保 GUI 中需要的图标已经可用。
    ///
    /// 方块物品会生成 3D BakedModel 图标；工具、材料和普通物品直接使用原始 2D 贴图。
    pub fn ensure_gui_icon(
        commands: &mut Commands,
        item: &ItemId,
        render_context: &mut ItemRenderContext<'_>,
        icon_cache: &mut GuiItemIconCache,
    ) -> Option<Handle<Image>> {
        if let Some(image) = Self::gui_icon_image(
            item,
            render_context.item_registry,
            render_context.item_textures,
            icon_cache,
        ) {
            return Some(image);
        }

        if !Self::uses_3d_gui_icon(item, render_context.item_registry) {
            return None;
        }

        if let Some(block_identifier) =
            Self::gui_block_identifier(item, render_context.item_registry)
            && let (Some(block_registry), Some(block_render_assets)) = (
                render_context.block_registry,
                render_context.block_render_assets,
            )
            && let Some(image) = GuiItemIconBaker::bake_block_cube_icon(
                item.identifier(),
                &block_identifier,
                block_registry,
                block_render_assets,
                icon_cache,
                render_context.images,
            )
        {
            return Some(image);
        }

        let model = Self::baked_model(item, render_context)?;
        GuiItemIconBaker::bake(
            item.identifier(),
            &model,
            icon_cache,
            commands,
            render_context.images,
            render_context.materials,
        )
    }

    /// 获取 GUI 应显示的图片。
    ///
    /// 该方法隐藏了方块 3D 图标和普通物品 2D 贴图之间的差异。
    pub fn gui_icon_image(
        item: &ItemId,
        item_registry: Option<&ItemRegistry>,
        item_textures: &ItemTextureRegistry,
        icon_cache: &GuiItemIconCache,
    ) -> Option<Handle<Image>> {
        if item.is_air() {
            return None;
        }

        if Self::uses_3d_gui_icon(item, item_registry) {
            return icon_cache.icon_image(item.identifier());
        }

        Self::texture_icon_image(item, item_registry, item_textures)
    }

    /// 获取或烘焙用于 3D 场景的 BakedModel。
    pub fn baked_model(
        item: &ItemId,
        render_context: &mut ItemRenderContext<'_>,
    ) -> Option<BakedItemModel> {
        let resolved = ItemModelResolver::resolve(
            item,
            render_context.item_registry,
            render_context.item_model_registry,
        )?;

        if let Some(model) = render_context.model_cache.get_model(&resolved.model_id) {
            return Some(model.clone());
        }

        let bake_context = ItemModelBakeContext {
            block_registry: render_context.block_registry,
            block_render_assets: render_context.block_render_assets,
            item_textures: render_context.item_textures,
            images: &*render_context.images,
        };

        let baked = ItemModelBaker::bake(
            &resolved.definition,
            bake_context,
            render_context.meshes,
            render_context.materials,
            render_context.model_cache,
        )?;
        render_context
            .model_cache
            .insert_model(resolved.model_id, baked.clone());
        Some(baked)
    }

    /// 判断某个物品在 GUI 中是否应该使用 3D 图标。
    fn uses_3d_gui_icon(item: &ItemId, item_registry: Option<&ItemRegistry>) -> bool {
        item_registry
            .and_then(|registry| registry.get(item))
            .is_some_and(|definition| {
                definition.category == ItemCategory::Block
                    || definition.placeable_block.is_some()
                    || matches!(definition.icon, IconDefinition::Block(_))
            })
    }

    /// 获取普通物品的原始 2D 贴图。
    /// 解析 GUI 方块图标实际应该采样的方块 ID。
    fn gui_block_identifier(
        item: &ItemId,
        item_registry: Option<&ItemRegistry>,
    ) -> Option<Identifier> {
        item_registry.and_then(|registry| {
            let definition = registry.get(item)?;
            definition
                .placeable_block
                .clone()
                .or_else(|| definition.icon.as_block_id().cloned())
                .or_else(|| {
                    (definition.category == ItemCategory::Block)
                        .then(|| definition.identifier.clone())
                })
        })
    }
    fn texture_icon_image(
        item: &ItemId,
        item_registry: Option<&ItemRegistry>,
        item_textures: &ItemTextureRegistry,
    ) -> Option<Handle<Image>> {
        let texture_key = item_registry
            .and_then(|registry| registry.get(item))
            .and_then(|definition| match &definition.icon {
                IconDefinition::Texture(path) => Some(path.as_str()),
                IconDefinition::Block(_) => None,
            })
            .unwrap_or_else(|| item.identifier().path());

        item_textures.get_handle(texture_key).cloned()
    }

    /// 根据已经烘焙好的模型生成实体层级。
    fn spawn_baked_entity(
        commands: &mut Commands,
        model: &BakedItemModel,
        context: ItemDisplayContext,
        parent: Option<Entity>,
        name: impl Into<String>,
    ) -> SpawnedItemEntity {
        let root = commands
            .spawn((
                Name::new(name.into()),
                model.display_transform(context),
                Visibility::Inherited,
            ))
            .id();

        if let Some(parent) = parent {
            commands.entity(parent).add_child(root);
        }

        let mut parts = Vec::with_capacity(model.parts.len());
        for part in &model.parts {
            let part_entity = commands
                .spawn((
                    Name::new(part.name.clone()),
                    Mesh3d(part.mesh.clone()),
                    MeshMaterial3d(part.material.clone()),
                    part.transform,
                    Visibility::Inherited,
                ))
                .id();

            if !context.casts_shadows() {
                commands.entity(part_entity).insert(NotShadowCaster);
            }

            commands.entity(root).add_child(part_entity);
            parts.push(part_entity);
        }

        SpawnedItemEntity { root, parts }
    }
}

/// 预热方块物品的 GUI 3D 图标。
///
/// 普通物品不在这里创建离屏相机，GUI 会直接使用原始贴图，避免额外渲染开销。
pub fn prepare_item_model_render_assets_system(
    mut commands: Commands,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    item_registry: Option<Res<ItemRegistry>>,
    model_registry: Option<Res<ItemModelRegistry>>,
    item_textures: Option<Res<ItemTextureRegistry>>,
    mut previews: ResMut<GuiItemIconCache>,
    mut model_cache: ResMut<ItemModelCache>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(item_registry_res) = item_registry.as_ref() else {
        return;
    };
    let Some(item_textures_res) = item_textures.as_ref() else {
        return;
    };

    let resources_changed = previews.is_prepared()
        && (item_registry_res.is_changed()
            || item_textures_res.is_changed()
            || model_registry
                .as_ref()
                .is_some_and(|registry| registry.is_changed())
            || block_registry
                .as_ref()
                .is_some_and(|registry| registry.is_changed())
            || block_render_assets
                .as_ref()
                .is_some_and(|assets| assets.is_changed()));

    if resources_changed {
        previews.clear();
        model_cache.clear();
    }

    if previews.is_prepared() {
        return;
    }

    let item_registry: &ItemRegistry = item_registry_res;
    let item_textures: &ItemTextureRegistry = item_textures_res;
    if item_registry.is_empty() {
        return;
    }

    let item_ids: Vec<_> = item_registry
        .all_items()
        .map(|definition| ItemId::new(definition.identifier.clone()))
        .filter(|item| ItemRenderer::uses_3d_gui_icon(item, Some(item_registry)))
        .collect();

    let mut render_context = ItemRenderContext {
        item_registry: Some(item_registry),
        item_model_registry: model_registry.as_deref(),
        item_textures,
        block_registry: block_registry.as_deref(),
        block_render_assets: block_render_assets.as_deref(),
        images: &mut images,
        meshes: &mut meshes,
        materials: &mut materials,
        model_cache: &mut model_cache,
    };

    let mut all_icons_ready = true;
    for item in item_ids {
        let icon =
            ItemRenderer::ensure_gui_icon(&mut commands, &item, &mut render_context, &mut previews);
        all_icons_ready &= icon.is_some();
    }

    previews.set_prepared(all_icons_ready);
}
