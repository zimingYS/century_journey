use crate::client::renderer::item::{
    ItemDisplayContext, ItemModelCache, ItemRenderContext, ItemRenderer,
};
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::viewmodel::hand_view::ViewHandBuilder;
use crate::client::viewmodel::{
    HeldItemEntity, ViewModelPart, ViewModelRenderState, ViewModelRoot,
};
use crate::content::block::registry::BlockRegistry;
use crate::content::item::model::ItemModelRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::inventory::state::InventoryState;
use crate::game::player::model::config::PlayerModelConfig;
use bevy::prelude::*;

pub fn view_model_sync_system(
    inventory: Res<InventoryState>,
    item_registry: Option<Res<ItemRegistry>>,
    item_model_registry: Option<Res<ItemModelRegistry>>,
    item_textures: Res<ItemTextureRegistry>,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<PlayerModelConfig>,
    mut render_state: ResMut<ViewModelRenderState>,
    mut item_model_cache: ResMut<ItemModelCache>,
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
        commands
            .entity(entity)
            .despawn_related::<Children>()
            .despawn();
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

    let mut render_context = ItemRenderContext {
        item_registry: item_registry.as_deref(),
        item_model_registry: item_model_registry.as_deref(),
        item_textures: &item_textures,
        block_registry: block_registry.as_deref(),
        block_render_assets: block_render_assets.as_deref(),
        images: &mut images,
        meshes: &mut meshes,
        materials: &mut materials,
        model_cache: &mut item_model_cache,
    };

    let Some(spawned) = ItemRenderer::spawn_item_entity(
        &mut commands,
        &item,
        ItemDisplayContext::FirstPersonRightHand,
        render_state.hand_entity,
        format!("HeldItem_{item_key}"),
        &mut render_context,
    ) else {
        render_state.current_item = None;
        render_state.held_entity = None;
        return;
    };

    for entity in spawned.entities() {
        commands.entity(entity).insert(ViewModelPart);
    }

    commands.entity(spawned.root).insert(HeldItemEntity {
        item_identifier: item_identifier.clone(),
    });
    render_state.current_item = Some(item_identifier.clone());
    render_state.held_entity = Some(spawned.root);
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
