use crate::client::renderer::held_renderer::{
    HeldItemConfig, HeldRenderDefinition, block_renderer::HeldBlockRenderer,
    flat_item_renderer::HeldFlatItemRenderer,
};
use crate::client::renderer::mesh_cache::HeldMeshCache;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::viewmodel::hand_view::ViewHandBuilder;
use crate::client::viewmodel::{
    HeldItemEntity, ViewModelPart, ViewModelRenderState, ViewModelRoot,
};
use crate::content::block::registry::BlockRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::inventory::state::InventoryState;
use crate::game::player::model::config::PlayerModelConfig;
use crate::shared::item_id::ItemId;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

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
        Some(entity) => entity,
        None => return,
    };

    let item = inventory
        .hotbar
        .get_stack(inventory.hotbar.active_index)
        .map(|stack| stack.item.clone())
        .unwrap_or_default();

    let item_identifier = item.identifier();
    let item_key = item_identifier.to_string();
    let is_air = item.is_air();

    if render_state.current_item.as_ref() == Some(item_identifier)
        && render_state.held_entity.is_some()
    {
        return;
    }

    if let Some(old_entity) = render_state.held_entity.take()
        && let Ok(entity) = held_query.get(old_entity)
    {
        commands.entity(entity).despawn();
    }

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

    let Some(config_item) = resolve_held_config(&item, item_registry.as_deref()) else {
        return;
    };
    let transform = config_item.to_transform();

    let held_entity = match &config_item.render {
        HeldRenderDefinition::Block => {
            if let (Some(registry), Some(render_assets)) =
                (block_registry.as_ref(), block_render_assets.as_ref())
            {
                let block_id = item_key.strip_prefix("block:").unwrap_or(&item_key);
                spawn_block_item(
                    &mut commands,
                    &mut meshes,
                    &mut mesh_cache,
                    registry,
                    render_assets,
                    render_state.hand_entity,
                    block_id,
                    &transform,
                )
            } else {
                None
            }
        }
        HeldRenderDefinition::FlatItem { .. } => {
            let texture_key = item_key.strip_prefix("item:").unwrap_or(&item_key);
            spawn_flat_item(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut mesh_cache,
                &item_textures,
                &images,
                render_state.hand_entity,
                texture_key,
                &config_item,
                &item_key,
                &transform,
            )
        }
        HeldRenderDefinition::Model { .. } | HeldRenderDefinition::Empty => None,
    };

    if let Some(entity) = held_entity {
        commands.entity(entity).insert(HeldItemEntity {
            item_identifier: item_identifier.clone(),
        });
        render_state.current_item = Some(item_identifier.clone());
        render_state.held_entity = Some(entity);
    } else {
        render_state.current_item = None;
        render_state.held_entity = None;
    }
}

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

    let item_anchor = ViewHandBuilder::spawn(commands, meshes, materials, config, vm_root);
    render_state.hand_entity = Some(item_anchor);
}

fn resolve_held_config(
    item: &ItemId,
    item_registry: Option<&ItemRegistry>,
) -> Option<HeldItemConfig> {
    use crate::content::item::definition::ItemCategory;

    let Some(registry) = item_registry else {
        return Some(HeldItemConfig::default_flat(0.05));
    };

    if registry.is_block(item) {
        return Some(HeldItemConfig::default_block());
    }

    if let Some(definition) = registry.get(item) {
        return Some(match definition.category {
            ItemCategory::Tool | ItemCategory::Weapon => HeldItemConfig::default_tool(0.05),
            _ => HeldItemConfig::default_flat(0.05),
        });
    }

    Some(HeldItemConfig::default_flat(0.05))
}

fn spawn_block_item(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    cache: &mut ResMut<HeldMeshCache>,
    block_registry: &BlockRegistry,
    render_assets: &BlockRenderAssets,
    parent: Option<Entity>,
    block_id: &str,
    transform: &Transform,
) -> Option<Entity> {
    let cache_key = format!("block:{block_id}");
    let mesh_handle = if let Some(handle) = cache.get(&cache_key) {
        handle.clone()
    } else {
        let mesh = HeldBlockRenderer::build_mesh(block_registry, block_id)?;
        let handle = meshes.add(mesh);
        cache.insert(cache_key, handle.clone());
        handle
    };
    let material = HeldBlockRenderer::material(block_registry, render_assets, block_id)?;

    let root = commands
        .spawn((
            Name::new(format!("HeldBlock_{block_id}")),
            ViewModelPart,
            *transform,
            Visibility::Inherited,
        ))
        .id();

    if let Some(parent) = parent {
        commands.entity(parent).add_child(root);
    }

    let cube = commands
        .spawn((
            Name::new("BlockCube"),
            ViewModelPart,
            Mesh3d(mesh_handle),
            MeshMaterial3d(material),
            Transform::default(),
            Visibility::Inherited,
            NotShadowCaster,
        ))
        .id();
    commands.entity(root).add_child(cube);
    Some(root)
}

fn spawn_flat_item(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    cache: &mut ResMut<HeldMeshCache>,
    item_textures: &ItemTextureRegistry,
    images: &Assets<Image>,
    parent: Option<Entity>,
    texture_key: &str,
    config: &HeldItemConfig,
    item_name: &str,
    transform: &Transform,
) -> Option<Entity> {
    let thickness = match &config.render {
        HeldRenderDefinition::FlatItem { thickness } => *thickness,
        _ => 0.05,
    };

    let image_handle = item_textures.get_handle(texture_key).cloned()?;
    let cache_key = format!("flat:{texture_key}/t={thickness}");
    let mesh_handle = if let Some(handle) = cache.get(&cache_key) {
        handle.clone()
    } else {
        let image = images.get(&image_handle)?;
        let mesh = HeldFlatItemRenderer::build_mesh(image, thickness);
        let handle = meshes.add(mesh);
        cache.insert(cache_key, handle.clone());
        handle
    };

    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Opaque,
        cull_mode: None,
        unlit: true,
        perceptual_roughness: 1.0,
        ..default()
    });

    let root = commands
        .spawn((
            Name::new(format!("HeldFlat_{item_name}")),
            ViewModelPart,
            *transform,
            Visibility::Inherited,
        ))
        .id();

    if let Some(parent) = parent {
        commands.entity(parent).add_child(root);
    }

    let plane = commands
        .spawn((
            Name::new("FlatItemMesh"),
            ViewModelPart,
            Mesh3d(mesh_handle),
            MeshMaterial3d(material),
            Transform::default(),
            Visibility::Inherited,
            NotShadowCaster,
        ))
        .id();
    commands.entity(root).add_child(plane);
    Some(root)
}
