use crate::client::renderer::held_renderer::{
    HeldItemConfig, HeldRenderDefinition, block_renderer::HeldBlockRenderer,
    flat_item_renderer::HeldFlatItemRenderer,
};
use crate::client::renderer::mesh_cache::HeldMeshCache;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::viewmodel::hand_view::ViewHandBuilder;
use crate::client::viewmodel::{HeldItemEntity, ViewModelRenderState, ViewModelRoot};
use crate::content::block::registry::BlockRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::inventory::state::InventoryState;
use crate::game::player::model::config::PlayerModelConfig;
use crate::shared::item_id::ItemId;
use bevy::prelude::*;

/// 第一人称模型同步渲染
pub fn view_model_sync_system(
    inventory: Res<InventoryState>,
    item_registry: Option<Res<ItemRegistry>>,
    item_textures: Res<ItemTextureRegistry>,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    images: Res<Assets<Image>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<PlayerModelConfig>,
    mut render_state: ResMut<ViewModelRenderState>,
    mut mesh_cache: ResMut<HeldMeshCache>,
    vm_query: Query<Entity, With<ViewModelRoot>>,
    held_query: Query<Entity, With<HeldItemEntity>>,
) {
    let vm_root = match vm_query.iter().next() {
        Some(e) => e,
        None => return,
    };

    let item = inventory
        .hotbar
        .get_stack(inventory.hotbar.active_index)
        .map(|s| s.item.clone())
        .unwrap_or_default();

    let item_identifier = item.identifier();
    let is_air = item.is_air();

    if render_state.current_item.as_ref() == Some(item_identifier)
        && render_state.held_entity.is_some()
    {
        return;
    }

    // 清除旧物品
    if let Some(old_e) = render_state.held_entity.take()
        && let Ok(e) = held_query.get(old_e)
    {
        commands.entity(e).despawn();
    }

    // 始终确保手部存在（ViewModelRoot 子节点）
    ensure_hand(
        &mut commands,
        &mut meshes,
        &mut materials,
        &config,
        &mut render_state,
        vm_root,
    );

    if is_air {
        render_state.current_item = Some(item_identifier.clone());
        return;
    }

    let config_item = resolve_held_config(&item, item_registry.as_deref());
    let Some(config_item) = config_item else {
        return;
    };
    let transform = config_item.to_transform();

    info!(
        "[视图模型] 生成: item={} render={:?} pos={:.2?}",
        item_identifier, config_item.render, transform.translation,
    );

    let item_str = item_identifier.to_string();

    // 手持物品挂在手掌实体下（hand_entity），而非直接挂 ViewModelRoot
    let held_entity = match &config_item.render {
        HeldRenderDefinition::Block => {
            if let (Some(reg), Some(render_assets)) =
                (block_registry.as_ref(), block_render_assets.as_ref())
            {
                let block_id = item_str.strip_prefix("block:").unwrap_or(&item_str);
                spawn_block_item(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut mesh_cache,
                    reg,
                    render_assets,
                    render_state.hand_entity,
                    block_id,
                    &transform,
                )
            } else {
                None
            }
        }
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
                render_state.hand_entity,
                tex_key,
                &config_item,
                &item_str,
                &transform,
            )
        }
        HeldRenderDefinition::Model { .. } => {
            info!("[视图模型] 特殊模型渲染尚未可用，跳过：{}", item_str);
            None
        }
        HeldRenderDefinition::Empty => None,
    };

    if let Some(e) = held_entity {
        commands.entity(e).insert(HeldItemEntity {
            item_identifier: item_identifier.clone(),
        });
        render_state.current_item = Some(item_identifier.clone());
        render_state.held_entity = Some(e);
    } else {
        render_state.current_item = None;
        render_state.held_entity = None;
    }
}

// 确保手部模型实体存在（ViewModelRoot 子节点）
fn ensure_hand(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    config: &PlayerModelConfig,
    render_state: &mut ResMut<ViewModelRenderState>,
    vm_root: Entity,
) {
    if render_state.hand_entity.is_some() {
        return;
    }
    let hand_entity = ViewHandBuilder::spawn(commands, meshes, materials, config, vm_root);
    render_state.hand_entity = Some(hand_entity);
}

// 根据物品ID解析手持渲染配置
fn resolve_held_config(
    item: &ItemId,
    item_registry: Option<&ItemRegistry>,
) -> Option<HeldItemConfig> {
    use crate::content::item::definition::ItemCategory;

    let Some(reg) = item_registry else {
        return Some(HeldItemConfig::default_flat(0.04));
    };

    if reg.is_block(item) {
        return Some(HeldItemConfig::default_block());
    }

    if let Some(def) = reg.get(item) {
        return Some(match def.category {
            ItemCategory::Tool | ItemCategory::Weapon => HeldItemConfig::default_tool(0.04),
            _ => HeldItemConfig::default_flat(0.04),
        });
    }

    Some(HeldItemConfig::default_flat(0.04))
}

// 生成方块类型的手持物品实体（挂载到 parent 实体下）
fn spawn_block_item(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    cache: &mut ResMut<HeldMeshCache>,
    block_registry: &BlockRegistry,
    render_assets: &BlockRenderAssets,
    parent: Option<Entity>,
    block_id: &str,
    transform: &Transform,
) -> Option<Entity> {
    let cache_key = format!("block:{}", block_id);
    let mesh_handle = if let Some(h) = cache.get(&cache_key) {
        h.clone()
    } else {
        let mesh = HeldBlockRenderer::build_mesh(block_registry, block_id)?;
        let h = meshes.add(mesh);
        cache.insert(cache_key, h.clone());
        h
    };
    let mat = HeldBlockRenderer::create_material(materials, render_assets);

    let root = commands
        .spawn((Name::new(format!("HeldBlock_{}", block_id)), *transform))
        .id();

    // 挂在手掌实体下（有手时），fallback 到 ViewModelRoot
    if let Some(p) = parent {
        commands.entity(p).add_child(root);
    }

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
    parent: Option<Entity>,
    tex_key: &str,
    config: &HeldItemConfig,
    item_name: &str,
    transform: &Transform,
) -> Option<Entity> {
    let thickness = match &config.render {
        HeldRenderDefinition::FlatItem { thickness } => *thickness,
        _ => 0.04,
    };

    let image_handle = item_textures
        .get_handle(tex_key)
        .cloned()
        .unwrap_or_default();
    if image_handle == Handle::<Image>::default() {
        warn!("[HeldRender] 纹理不存在: {}", tex_key);
        return None;
    }

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

    let mat = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle.clone()),
        alpha_mode: AlphaMode::Mask(0.1),
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 1.0,
        unlit: true,
        ..default()
    });

    let root = commands
        .spawn((Name::new(format!("HeldFlat_{}", item_name)), *transform))
        .id();

    // 挂在手掌实体下
    if let Some(p) = parent {
        commands.entity(p).add_child(root);
    }

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
