use bevy::prelude::*;

use crate::client::renderer::item::{
    ItemDisplayContext, ItemModelCache, ItemRenderContext, ItemRenderer,
};
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::model::ItemModelRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::inventory::state::InventoryState;
use crate::game::player::components::LocalPlayer;
use crate::game::player::events::FoodConsumedEvent;
use crate::game::player::model::rig::PlayerRigEntities;
use crate::shared::identifier::Identifier;
use crate::shared::item_id::ItemId;

const FOOD_USE_VISUAL_SECONDS: f32 = 0.5;

/// 真实第一人称手持物品渲染状态。
#[derive(Resource, Default)]
pub struct FullBodyHeldItemRenderState {
    /// 当前挂在真实右手上的物品根实体。
    pub held_entity: Option<Entity>,
    /// 当前已经渲染的物品标识，用于避免每帧重建模型。
    pub current_item: Option<Identifier>,
}

/// 标记由真实 PlayerRig 挂点生成的手持物品实体。
#[derive(Component)]
struct RigHeldItemEntity;

/// 进食会先扣除背包物品；这里暂存被消耗的食物，保证最后一份食物也能完整显示动画。
#[derive(Default)]
struct ConsumedFoodVisual {
    item: Option<ItemId>,
    remaining_seconds: f32,
}

/// 真实第一人称渲染插件。
///
/// 它不再生成独立 ViewModel，而是把本地玩家快捷栏物品挂到真实 PlayerRig 的 HeldItemAnchor 上。
pub struct FullBodyFirstPersonPlugin;

impl Plugin for FullBodyFirstPersonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FullBodyHeldItemRenderState>()
            .add_systems(
                Update,
                sync_full_body_held_item_system
                    .after(crate::game::player::systems::hunger::use_food_system),
            );
    }
}

/// 同步真实右手上的手持物品。
///
/// 外部只提供 ItemId；这里统一走 ItemRenderer 解析、烘焙并生成实体，保证第一人称和第三人称看到同一个物品模型。
fn sync_full_body_held_item_system(
    time: Res<Time>,
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
    mut render_state: ResMut<FullBodyHeldItemRenderState>,
    mut item_model_cache: ResMut<ItemModelCache>,
    rig_query: Query<(Entity, &PlayerRigEntities), With<LocalPlayer>>,
    mut consumed_reader: MessageReader<FoodConsumedEvent>,
    mut consumed_visual: Local<ConsumedFoodVisual>,
) {
    let Ok((player, rig)) = rig_query.single() else {
        return;
    };

    for event in consumed_reader.read() {
        if event.player == player {
            consumed_visual.item = Some(event.item.clone());
            consumed_visual.remaining_seconds = FOOD_USE_VISUAL_SECONDS;
        }
    }
    if consumed_visual.remaining_seconds > 0.0 {
        consumed_visual.remaining_seconds -= time.delta_secs();
        if consumed_visual.remaining_seconds <= 0.0 {
            consumed_visual.item = None;
        }
    }

    let selected_item = inventory
        .hotbar
        .get_stack(inventory.hotbar.active_index)
        .map(|stack| stack.item.clone())
        .unwrap_or_default();
    let item = consumed_visual.item.clone().unwrap_or(selected_item);

    let item_identifier = item.identifier();
    let is_air = item.is_air();
    if render_state.current_item.as_ref() == Some(item_identifier)
        && (is_air || render_state.held_entity.is_some())
    {
        return;
    }

    if let Some(old_entity) = render_state.held_entity.take() {
        despawn_hierarchy_if_exists(&mut commands, old_entity);
    }

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

    let item_key = item_identifier.to_string();
    let Some(spawned) = ItemRenderer::spawn_item_entity(
        &mut commands,
        &item,
        ItemDisplayContext::ThirdPersonRightHand,
        Some(rig.held_item),
        format!("RigHeldItem_{item_key}"),
        &mut render_context,
    ) else {
        render_state.current_item = None;
        render_state.held_entity = None;
        return;
    };

    for entity in spawned.entities() {
        commands.entity(entity).insert(RigHeldItemEntity);
    }

    render_state.current_item = Some(item_identifier.clone());
    render_state.held_entity = Some(spawned.root);
}

fn despawn_hierarchy_if_exists(commands: &mut Commands, entity: Entity) {
    commands.queue(move |world: &mut World| {
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.despawn_related::<Children>();
            entity_mut.despawn();
        }
    });
}
