use bevy::prelude::*;
use crate::content::constant::world::CHUNK_SIZE;
use crate::content::block::registry::BlockRegistry;
use crate::game::world::entity::dropped_item::DroppedItem;
use crate::game::world::storage::WorldStorage;

const GRAVITY: f32 = -15.0;
const GROUND_CHECK: f32 = 0.3;
const MERGE_RANGE: f32 = 1.5;
const MAX_ITEM_AGE: f32 = 300.0;

/// 掉落物速度 (Y轴)
#[derive(Component, Default)]
pub struct DroppedItemVelocity {
    pub y: f32,
}

/// 掉落物着地标记
#[derive(Component, Default)]
pub struct DroppedItemGrounded;

/// 掉落物重力 + 着地检测
pub fn dropped_item_gravity_system(
    time: Res<Time>,
    registry: Option<Res<BlockRegistry>>,
    world_storage: Res<WorldStorage>,
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut DroppedItemVelocity), Without<DroppedItemGrounded>>,
) {
    let Some(reg) = registry.as_ref() else { return };
    let dt = time.delta_secs();

    for (entity, transform, mut velocity) in &mut query {
        velocity.y += GRAVITY * dt;
        let new_y = transform.translation.y + velocity.y * dt;

        // 检查地面碰撞
        let foot_y = new_y - GROUND_CHECK;
        let block_x = transform.translation.x.floor() as i32;
        let block_z = transform.translation.z.floor() as i32;
        let block_y = foot_y.floor() as i32;

        let chunk_pos = IVec3::new(
            block_x.div_euclid(CHUNK_SIZE as i32),
            block_y.div_euclid(CHUNK_SIZE as i32),
            block_z.div_euclid(CHUNK_SIZE as i32),
        );

        let local = IVec3::new(
            block_x.rem_euclid(CHUNK_SIZE as i32),
            block_y.rem_euclid(CHUNK_SIZE as i32),
            block_z.rem_euclid(CHUNK_SIZE as i32),
        );

        let is_solid = world_storage.loaded_chunks.get(&chunk_pos)
            .map(|c| {
                let id = c.get_voxel(local.x as usize, local.y as usize, local.z as usize);
                id != 0 && reg.get(id).map_or(false, |p| p.is_solid)
            })
            .unwrap_or(false);

        if is_solid {
            // 着地: 固定在方块表面
            commands.entity(entity).insert((
                Transform::from_translation(Vec3::new(
                    transform.translation.x,
                    block_y as f32 + 1.0 + GROUND_CHECK,
                    transform.translation.z,
                )).with_scale(Vec3::splat(0.25)),
                DroppedItemGrounded,
            ));
        } else {
            commands.entity(entity).insert(
                Transform::from_translation(Vec3::new(
                    transform.translation.x, new_y, transform.translation.z,
                )).with_scale(Vec3::splat(0.25)),
            );
        }
    }
}

/// 同种掉落物合并 (范围内)
pub fn dropped_item_merge_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &DroppedItem), With<DroppedItemGrounded>>,
) {
    let items: Vec<(Entity, Vec3, DroppedItem)> = query.iter()
        .map(|(e, t, d)| (e, t.translation, d.clone()))
        .collect();

    let mut merged = std::collections::HashSet::new();
    for i in 0..items.len() {
        if merged.contains(&items[i].0) { continue; }
        for j in (i + 1)..items.len() {
            if merged.contains(&items[j].0) { continue; }
            let dist = items[i].1.distance(items[j].1);
            if dist > MERGE_RANGE { continue; }
            if items[i].2.stack.item != items[j].2.stack.item { continue; }

            let total = items[i].2.stack.count + items[j].2.stack.count;
            let max = crate::game::inventory::item::stack::ItemStack::MAX_STACK_SIZE;
            if total <= max {
                commands.entity(items[i].0).insert(DroppedItem::new(
                    crate::game::inventory::item::stack::ItemStack::new(items[i].2.stack.item.clone(), total)
                ));
                commands.entity(items[j].0).despawn();
                merged.insert(items[i].0);
                merged.insert(items[j].0);
            }
        }
    }
}

/// 超时销毁 (5 分钟) — 由 dropped_item_tick_system 在 dropped_item.rs 中处理