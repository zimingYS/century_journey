use crate::content::block::definition::RenderMode;
use crate::content::block::registry::BlockRegistry;
use crate::content::constant::world::CHUNK_SIZE;
use crate::game::world::storage::WorldStorage;
use bevy::prelude::*;

/// 判断指定世界坐标的方块是否为固体
fn is_voxel_solid(
    vx: i32,
    vy: i32,
    vz: i32,
    world_storage: &WorldStorage,
    registry: &BlockRegistry,
) -> bool {
    let chunk_pos = IVec3::new(
        vx.div_euclid(CHUNK_SIZE as i32),
        vy.div_euclid(CHUNK_SIZE as i32),
        vz.div_euclid(CHUNK_SIZE as i32),
    );
    let local = IVec3::new(
        vx.rem_euclid(CHUNK_SIZE as i32),
        vy.rem_euclid(CHUNK_SIZE as i32),
        vz.rem_euclid(CHUNK_SIZE as i32),
    );

    let Some(chunk_data) = world_storage.loaded_chunks.get(&chunk_pos) else {
        return true;
    };

    let voxel_id = chunk_data.get_voxel(local.x as usize, local.y as usize, local.z as usize);
    if voxel_id == 0 {
        return false;
    }

    // 水面不碰撞
    if let Some(identifier) = registry.get_identifier_by_id(voxel_id)
        && identifier.path() == "water"
    {
        return false;
    }

    let Some(prop) = registry.get(voxel_id) else {
        return true;
    };

    prop.is_solid && prop.render_mode != RenderMode::Cutout
}

/// 检测AABB是否与固体方块重叠
/// 未加载区块视为固体，阻止移动
pub fn check_collision_at(
    position: Vec3,
    half_extents: Vec3,
    world_storage: &WorldStorage,
    registry: &BlockRegistry,
) -> bool {
    let min_x = (position.x - half_extents.x).floor() as i32;
    let max_x = (position.x + half_extents.x).ceil() as i32;
    let min_y = (position.y - half_extents.y).floor() as i32;
    let max_y = (position.y + half_extents.y).ceil() as i32;
    let min_z = (position.z - half_extents.z).floor() as i32;
    let max_z = (position.z + half_extents.z).ceil() as i32;

    for vx in min_x..=max_x {
        for vy in min_y..=max_y {
            for vz in min_z..=max_z {
                if is_voxel_solid(vx, vy, vz, world_storage, registry) {
                    // 确认AABB确实与方块重叠
                    if position.x - half_extents.x < (vx + 1) as f32
                        && position.x + half_extents.x > vx as f32
                        && position.y - half_extents.y < (vy + 1) as f32
                        && position.y + half_extents.y > vy as f32
                        && position.z - half_extents.z < (vz + 1) as f32
                        && position.z + half_extents.z > vz as f32
                    {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// 检测玩家是否着地
pub fn is_grounded_at(
    position: Vec3,
    half_extents: Vec3,
    world_storage: &WorldStorage,
    registry: &BlockRegistry,
) -> bool {
    let feet_pos = position - Vec3::new(0.0, 0.05, 0.0);
    check_collision_at(feet_pos, half_extents, world_storage, registry)
}

/// 寻找安全位置
pub fn find_safe_position(
    start_pos: Vec3,
    half_extents: Vec3,
    world_storage: &WorldStorage,
    registry: &BlockRegistry,
) -> Option<Vec3> {
    // 优先使用最短水平位移脱离碰撞，避免沿树干或方块柱一路传送到顶部。
    let directions = [
        Vec3::X,
        Vec3::NEG_X,
        Vec3::Z,
        Vec3::NEG_Z,
        Vec3::new(1.0, 0.0, 1.0).normalize(),
        Vec3::new(-1.0, 0.0, 1.0).normalize(),
        Vec3::new(1.0, 0.0, -1.0).normalize(),
        Vec3::new(-1.0, 0.0, -1.0).normalize(),
    ];
    for step in 1..=16 {
        let distance = step as f32 * 0.1;
        for direction in directions {
            let test_pos = start_pos + direction * distance;
            if !check_collision_at(test_pos, half_extents, world_storage, registry) {
                return Some(test_pos);
            }
        }
    }

    // 水平空间完全被堵住时才有限度向上恢复，禁止扫描整根方块柱。
    for step in 1..=24 {
        let test_pos = start_pos + Vec3::Y * (step as f32 * 0.1);
        if !check_collision_at(test_pos, half_extents, world_storage, registry) {
            return Some(test_pos);
        }
    }

    None
}
