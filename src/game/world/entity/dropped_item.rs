use crate::client::renderer::item::{
    ItemDisplayContext, ItemModelCache, ItemRenderContext, ItemRenderer,
};
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::content::block::registry::BlockRegistry;
use crate::content::constant::world::CHUNK_SIZE;
use crate::content::item::model::ItemModelRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::world::storage::WorldStorage;
use bevy::prelude::*;

/// 掉落物逻辑数据。
#[derive(Component, Debug, Clone)]
pub struct DroppedItem {
    /// 掉落物中保存的物品堆。
    pub stack: ItemStack,
    /// 已存在时间，超过生命周期后会销毁。
    pub age: f32,
    /// 拾取延迟，刚生成时避免立刻被玩家吸回去。
    pub pickup_delay: f32,
}

impl DroppedItem {
    /// 创建一个新的掉落物组件。
    pub fn new(stack: ItemStack) -> Self {
        Self {
            stack,
            age: 0.0,
            pickup_delay: 0.5,
        }
    }

    /// 判断当前掉落物是否允许被拾取。
    pub fn can_pickup(&self) -> bool {
        self.pickup_delay <= 0.0
    }
}

/// 掉落物竖直方向速度。
#[derive(Component, Default)]
pub struct DroppedItemVelocity {
    /// 当前 Y 轴速度。
    pub y: f32,
}

/// 标记一个实体需要生成掉落物视觉模型。
#[derive(Component)]
pub struct DroppedItemVisual;

/// 标记掉落物视觉模型已经通过 ItemRenderer 创建。
#[derive(Component)]
pub struct DroppedItemVisualReady;

/// 标记掉落物已经落地。
#[derive(Component, Default)]
pub struct DroppedItemGrounded;

/// 判断掉落物下方是否有可站立方块。
fn solid_below(pos: Vec3, storage: &WorldStorage, reg: &BlockRegistry) -> bool {
    let bx = pos.x.floor() as i32;
    let by = (pos.y - 0.3).floor() as i32;
    let bz = pos.z.floor() as i32;
    let cp = IVec3::new(
        bx.div_euclid(CHUNK_SIZE as i32),
        by.div_euclid(CHUNK_SIZE as i32),
        bz.div_euclid(CHUNK_SIZE as i32),
    );
    let lp = IVec3::new(
        bx.rem_euclid(CHUNK_SIZE as i32),
        by.rem_euclid(CHUNK_SIZE as i32),
        bz.rem_euclid(CHUNK_SIZE as i32),
    );
    storage.loaded_chunks.get(&cp).is_none_or(|c| {
        let id = c.get_voxel(lp.x as usize, lp.y as usize, lp.z as usize);
        id != 0 && reg.get(id).is_some_and(|p| p.is_solid)
    })
}

/// 在世界中生成一个掉落物实体。
pub fn spawn_dropped_item(commands: &mut Commands, position: Vec3, stack: ItemStack) -> Entity {
    commands
        .spawn((
            DroppedItem::new(stack),
            DroppedItemVisual,
            DroppedItemVelocity::default(),
            Name::new("DroppedItem".to_string()),
            Transform::from_translation(position + Vec3::new(0.5, 1.0, 0.5)),
            Visibility::default(),
        ))
        .id()
}

/// 为新掉落物生成视觉模型。
///
/// 这里不再判断方块/贴图/挤出模型，只把物品和 Ground 场景交给统一 ItemRenderer。
pub fn dropped_item_visual_system(
    mut commands: Commands,
    query: Query<
        (Entity, &DroppedItem),
        (With<DroppedItemVisual>, Without<DroppedItemVisualReady>),
    >,
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

/// 掉落物重力系统。
pub fn dropped_item_gravity_system(
    time: Res<Time>,
    reg: Option<Res<BlockRegistry>>,
    storage: Res<WorldStorage>,
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut DroppedItemVelocity), Without<DroppedItemGrounded>>,
) {
    let Some(reg) = reg.as_ref() else { return };
    for (e, t, mut v) in &mut query {
        v.y -= 15.0 * time.delta_secs();
        let ny = t.translation.y + v.y * time.delta_secs();
        if solid_below(
            Vec3::new(t.translation.x, ny, t.translation.z),
            &storage,
            reg,
        ) {
            let ground_y = (ny - 0.3).floor() + 1.0 + 0.3;
            commands.entity(e).insert((
                Transform::from_translation(Vec3::new(t.translation.x, ground_y, t.translation.z)),
                DroppedItemGrounded,
            ));
        } else {
            commands
                .entity(e)
                .insert(Transform::from_translation(Vec3::new(
                    t.translation.x,
                    ny,
                    t.translation.z,
                )));
        }
    }
}

/// 合并附近同类掉落物。
pub fn dropped_item_merge_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &DroppedItem), With<DroppedItemGrounded>>,
) {
    let items: Vec<_> = query
        .iter()
        .map(|(e, t, d)| (e, t.translation, d.stack.clone()))
        .collect();
    let mut skip = std::collections::HashSet::new();
    for i in 0..items.len() {
        if skip.contains(&items[i].0) {
            continue;
        }
        for j in (i + 1)..items.len() {
            if skip.contains(&items[j].0) {
                continue;
            }
            if items[i].1.distance(items[j].1) > 1.5 {
                continue;
            }
            if items[i].2.item != items[j].2.item {
                continue;
            }
            let total_count = items[i].2.count + items[j].2.count;
            let max_count = crate::game::inventory::item::stack::ItemStack::MAX_STACK_SIZE;
            if total_count <= max_count {
                commands
                    .entity(items[i].0)
                    .insert(DroppedItem::new(ItemStack::new(
                        items[i].2.item.clone(),
                        total_count,
                    )));
                commands
                    .entity(items[j].0)
                    .despawn_related::<Children>()
                    .despawn();
                skip.insert(items[i].0);
                skip.insert(items[j].0);
            }
        }
    }
}

/// 更新掉落物生命周期和拾取延迟。
pub fn dropped_item_tick_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut DroppedItem)>,
) {
    for (e, mut item) in &mut query {
        item.age += time.delta_secs();
        item.pickup_delay = (item.pickup_delay - time.delta_secs()).max(0.0);
        if item.age > 300.0 {
            commands.entity(e).despawn_related::<Children>().despawn();
        }
    }
}

/// 让已落地掉落物缓慢自旋。
pub fn dropped_item_spin_system(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DroppedItemGrounded>>,
) {
    for mut t in &mut query {
        t.rotate_y(time.delta_secs() * 2.0);
    }
}

/// 掉落物系统插件。
pub struct DroppedItemPlugin;
impl Plugin for DroppedItemPlugin {
    /// 注册掉落物视觉、物理、合并、生命周期和旋转系统。
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                dropped_item_visual_system,
                dropped_item_gravity_system,
                dropped_item_merge_system,
                dropped_item_tick_system,
                dropped_item_spin_system,
            ),
        );
    }
}
