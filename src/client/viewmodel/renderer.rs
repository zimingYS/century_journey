use crate::client::renderer::held_renderer::{
    HeldItemConfig, HeldRenderDefinition, block_renderer::HeldBlockRenderer,
    flat_item_renderer::HeldFlatItemRenderer, hand_renderer::HandRenderer,
};
use crate::client::renderer::mesh_cache::HeldMeshCache;
use crate::client::viewmodel::{HeldItemEntity, ViewModelRenderState, ViewModelRoot};
use crate::content::block::registry::BlockRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::engine::asset::manager::AssetManager;
use crate::game::inventory::state::InventoryState;
use crate::shared::item_id::ItemId;
use bevy::prelude::*;

/// 第一人称模型同步渲染
pub fn view_model_sync_system(
    inventory: Res<InventoryState>,
    item_registry: Option<Res<ItemRegistry>>,
    item_textures: Res<ItemTextureRegistry>,
    block_registry: Option<Res<BlockRegistry>>,
    mut asset: ResMut<AssetManager>,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut render_state: ResMut<ViewModelRenderState>,
    mut mesh_cache: ResMut<HeldMeshCache>,
    vm_query: Query<Entity, With<ViewModelRoot>>,
    held_query: Query<Entity, With<HeldItemEntity>>,
) {
    // 获取视图模型根节点实体
    let vm_root = match vm_query.iter().next() {
        Some(e) => e,
        None => return,
    };

    // 从热栏中获取当前选中格子的物品ID
    let item = inventory
        .hotbar
        .get_stack(inventory.hotbar.active_index)
        .map(|s| s.item.clone())
        .unwrap_or(ItemId::air());

    // 将物品ID转为字符串标识（用于日志和纹理查找）
    let item_identifier = item.identifier();
    let is_air = item.is_air();

    // 若当前渲染的物品与选中物品一致，直接跳过
    if render_state.current_item.as_ref() == Some(item_identifier)
        && render_state.held_entity.is_some()
    {
        return;
    }

    // 清除旧物品
    if let Some(old_e) = render_state.held_entity.take() {
        if let Ok(e) = held_query.get(old_e) {
            commands.entity(e).despawn();
        }
    }

    // 始终确保手部存在
    ensure_hand(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut asset,
        &asset_server,
        &mut render_state,
        vm_root,
    );

    // 更新渲染状态中的当前物品标记
    if is_air {
        render_state.current_item = Some(item_identifier.clone());
        return;
    }

    // 根据物品类型解析手持渲染配置
    let config = resolve_held_config(&item, item_registry.as_deref());
    let Some(config) = config else {
        return;
    };
    let transform = config.to_transform();

    info!(
        "[视图模型] 生成: item={} render={:?} pos={:.2?}",
        item_identifier, config.render, transform.translation,
    );

    // 根据类型，生成对应手持实体
    let item_str = item_identifier.to_string();
    let held_entity = match &config.render {
        // 方块
        HeldRenderDefinition::Block => {
            if let Some(reg) = block_registry.as_ref() {
                let block_id = item_str.strip_prefix("block:").unwrap_or(&item_str);
                spawn_block_item(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut mesh_cache,
                    reg,
                    vm_root,
                    block_id,
                    &transform,
                )
            } else {
                None
            }
        }
        // 物品
        HeldRenderDefinition::FlatItem {
            thickness: _thickness,
        } => {
            let tex_key = item_str.strip_prefix("item:").unwrap_or(&item_str);
            spawn_flat_item(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut mesh_cache,
                &item_textures,
                &images,
                vm_root,
                tex_key,
                &config,
                &item_str,
                &transform,
            )
        }
        // 特殊模型
        HeldRenderDefinition::Model { .. } => {
            info!("[视图模型] 特殊模型渲染尚未可用，跳过：{}", item_str);
            None
        }
        HeldRenderDefinition::Empty => None,
    };

    // 为生成的手持实体添加标记组件
    if let Some(e) = held_entity {
        commands.entity(e).insert(HeldItemEntity {
            item_identifier: item_identifier.clone(),
        });

        render_state.current_item = Some(item_identifier.clone());
        render_state.held_entity = Some(e);
    } else {
        // 图片可能尚未加载，下一帧继续尝试
        render_state.current_item = None;
        render_state.held_entity = None;
    }

    render_state.held_entity = held_entity;
}

// 确保手部模型实体存在
fn ensure_hand(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    mut asset: &mut AssetManager,
    asset_server: &AssetServer,
    render_state: &mut ResMut<ViewModelRenderState>,
    vm_root: Entity,
) {
    // 已存在手部实体则直接跳过
    if render_state.hand_entity.is_some() {
        return;
    }

    // 生成手部实体
    let hand_mesh = HandRenderer::build_hand_mesh(meshes);
    let hand_mat = HandRenderer::create_hand_material(materials, &mut asset, &asset_server);
    let hand = commands
        .spawn((
            Name::new("HandMesh"),
            Mesh3d(hand_mesh),
            MeshMaterial3d(hand_mat),
            HandRenderer::default_hand_transform(),
        ))
        .id();
    commands.entity(vm_root).add_child(hand);
    render_state.hand_entity = Some(hand);
}

// 根据物品ID解析手持渲染配置
fn resolve_held_config(
    item: &ItemId,
    item_registry: Option<&ItemRegistry>,
) -> Option<HeldItemConfig> {
    use crate::client::renderer::held_renderer::HeldItemConfig;
    use crate::content::item::definition::ItemCategory;

    let Some(reg) = item_registry else {
        return Some(HeldItemConfig::default_flat(0.04));
    };

    // 方块类物品直接返回默认方块配置
    if reg.is_block(item) {
        return Some(HeldItemConfig::default_block());
    }

    if let Some(def) = reg.get(item) {
        return Some(match def.category {
            ItemCategory::Tool | ItemCategory::Weapon => HeldItemConfig::default_tool(0.04),
            _ => HeldItemConfig::default_flat(0.04),
        });
    }

    // 未知物品统一用默认平面配置
    Some(HeldItemConfig::default_flat(0.04))
}

// 生成方块类型的手持物品实体
fn spawn_block_item(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    cache: &mut ResMut<HeldMeshCache>,
    block_registry: &BlockRegistry,
    parent: Entity,
    block_id: &str,
    transform: &Transform,
) -> Option<Entity> {
    // 构建缓存键，按方块ID区分网格
    let cache_key = format!("block:{}", block_id);
    // 优先从缓存取网格
    let mesh_handle = if let Some(h) = cache.get(&cache_key) {
        h.clone()
    } else {
        let mesh = HeldBlockRenderer::build_mesh(block_registry, block_id)?;
        let h = meshes.add(mesh);
        cache.insert(cache_key, h.clone());
        h
    };
    // 创建方块PBR材质
    let mat = HeldBlockRenderer::create_material(materials, block_registry);

    // 生成根节点实体
    let root = commands
        .spawn((Name::new(format!("HeldBlock_{}", block_id)), *transform))
        .id();
    commands.entity(parent).add_child(root);

    let cube = commands
        .spawn((
            Name::new("BlockCube"),
            Mesh3d(mesh_handle),
            MeshMaterial3d(mat),
            Transform::default(),
        ))
        .id();
    commands.entity(root).add_child(cube);
    Some(root)
}

/// 生成平面挤出类型的手持物品实体
fn spawn_flat_item(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    cache: &mut ResMut<HeldMeshCache>,
    item_textures: &ItemTextureRegistry,
    images: &Assets<Image>,
    parent: Entity,
    tex_key: &str,
    config: &HeldItemConfig,
    item_name: &str,
    transform: &Transform,
) -> Option<Entity> {
    // 从配置中提取挤出厚度
    let thickness = match &config.render {
        HeldRenderDefinition::FlatItem { thickness } => *thickness,
        _ => 0.04,
    };

    // 从纹理注册表获取对应物品的纹理句柄
    let image_handle = item_textures
        .get_handle(tex_key)
        .cloned()
        .unwrap_or_default();
    if image_handle == Handle::<Image>::default() {
        warn!("[HeldRender] 纹理不存在: {}", tex_key);
        return None;
    }

    // 构建缓存键
    let cache_key = format!("flat:{}/t={}", tex_key, thickness);
    let mesh_handle = if let Some(handle) = cache.get(&cache_key) {
        handle.clone()
    } else {
        let Some(image) = images.get(&image_handle) else {
            info!("[HeldRender] 等待纹理加载: {}", tex_key);
            return None;
        };

        info!("[HeldRender] 构建厚度模型: {}", tex_key);
        let mesh = HeldFlatItemRenderer::build_mesh(image, thickness);
        let handle = meshes.add(mesh);
        cache.insert(cache_key, handle.clone());

        handle
    };

    // 创建物品材质
    let mat = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle.clone()),
        alpha_mode: AlphaMode::Mask(0.1),
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 1.0,
        unlit: true,
        ..default()
    });

    // 生成根节点实体
    let root = commands
        .spawn((Name::new(format!("HeldFlat_{}", item_name)), *transform))
        .id();
    commands.entity(parent).add_child(root);
    // 生成平面网格子实体
    let plane = commands
        .spawn((
            Name::new("FlatPlane"),
            Mesh3d(mesh_handle),
            MeshMaterial3d(mat),
            Transform::default(),
        ))
        .id();
    commands.entity(root).add_child(plane);
    Some(root)
}

/// 生成3D模型类型的手持物品实体 （预留）
#[allow(dead_code)]
fn spawn_model_item(
    commands: &mut Commands,
    mut asset: &mut AssetManager,
    parent: Entity,
    path: &str,
    transform: &Transform,
) -> Entity {
    // 通过模型渲染器加载GLTF场景
    let scene =
        crate::client::renderer::held_renderer::model_renderer::HeldModelRenderer::load_model(
            &mut asset, path,
        );
    let root = commands
        .spawn((Name::new(format!("HeldModel_{}", path)), *transform))
        .id();
    commands.entity(parent).add_child(root);

    // 使用 Scene 组件直接绑定
    commands
        .spawn((Name::new("ModelScene"), scene, Transform::default()))
        .set_parent_in_place(root);
    root
}
