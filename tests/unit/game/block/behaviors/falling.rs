use super::*;
use crate::game::world::block_ops::{get_voxel_at_world, set_voxel_at_world};
use crate::game::world::chunk::ChunkData;
use crate::game::world::state::{ChunkRuntime, WorldState};
use crate::game::world::voxel_change::apply::apply_changes;
use crate::game::world::voxel_change::provenance::VoxelProvenance;
use crate::shared::voxel_change::VoxelChangeBuffer;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::{Commands, IVec3, World};
use std::sync::Arc;

/// 测试用方块 ID（不依赖真实注册表，行为只比较 ID 语义）
const SAND: u16 = 7;
const STONE: u16 = 3;

fn loaded_world() -> WorldState {
    let mut state = WorldState::default();
    state.insert_chunk(IVec3::ZERO, Arc::new(ChunkData::new()));
    state
}

#[test]
fn below_becoming_air_triggers_fall() {
    let mut world = loaded_world();
    set_voxel_at_world(IVec3::new(1, 2, 1), SAND, &mut world).unwrap();
    set_voxel_at_world(IVec3::new(1, 1, 1), STONE, &mut world).unwrap();
    set_voxel_at_world(IVec3::new(1, 1, 1), 0, &mut world).unwrap();
    let mut runtime = ChunkRuntime::default();
    let mut buffer = VoxelChangeBuffer::default();
    let dummy_world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &dummy_world);

    let behavior = FallingBlockBehavior;
    // 下方支撑被挖 → 下方变空气
    behavior.on_neighbor_update(
        IVec3::new(1, 2, 1),
        SAND,
        IVec3::new(1, 1, 1),
        0,
        &world,
        &mut buffer,
        &mut commands,
    );

    assert_eq!(buffer.0.len(), 2, "应推 2 条命令：原位清空 + 下方写入");
    apply_changes(
        &mut world,
        &mut runtime,
        &mut VoxelProvenance::default(),
        &mut buffer,
    );
    assert_eq!(get_voxel_at_world(IVec3::new(1, 2, 1), &world), 0);
    assert_eq!(get_voxel_at_world(IVec3::new(1, 1, 1), &world), SAND);
}

#[test]
fn solid_below_does_not_fall() {
    let mut world = loaded_world();
    set_voxel_at_world(IVec3::new(1, 2, 1), SAND, &mut world).unwrap();
    let mut buffer = VoxelChangeBuffer::default();
    let dummy_world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &dummy_world);

    let behavior = FallingBlockBehavior;
    // 下方邻居变成了石头（非空）
    behavior.on_neighbor_update(
        IVec3::new(1, 2, 1),
        SAND,
        IVec3::new(1, 1, 1),
        STONE,
        &world,
        &mut buffer,
        &mut commands,
    );

    assert!(buffer.0.is_empty(), "下方非空不应下落");
}

#[test]
fn non_below_neighbor_is_ignored() {
    let mut world = loaded_world();
    set_voxel_at_world(IVec3::new(1, 2, 1), SAND, &mut world).unwrap();
    let mut buffer = VoxelChangeBuffer::default();
    let dummy_world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &dummy_world);

    let behavior = FallingBlockBehavior;
    // 上方邻居变化（上方变空气），不是下方
    behavior.on_neighbor_update(
        IVec3::new(1, 2, 1),
        SAND,
        IVec3::new(1, 3, 1),
        0,
        &world,
        &mut buffer,
        &mut commands,
    );

    assert!(buffer.0.is_empty(), "非下方邻居变化不应触发");
}

#[test]
fn stale_self_position_is_skipped() {
    let mut world = loaded_world();
    // 世界里该位置已不是 SAND（事件滞后：方块已被替换）
    set_voxel_at_world(IVec3::new(1, 2, 1), STONE, &mut world).unwrap();
    let mut buffer = VoxelChangeBuffer::default();
    let dummy_world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &dummy_world);

    let behavior = FallingBlockBehavior;
    behavior.on_neighbor_update(
        IVec3::new(1, 2, 1),
        SAND, // 事件认为这里是 SAND，但实际已是 STONE
        IVec3::new(1, 1, 1),
        0,
        &world,
        &mut buffer,
        &mut commands,
    );

    assert!(buffer.0.is_empty(), "自身位置已变应跳过");
}

#[test]
fn placed_in_air_falls_on_change() {
    let mut world = loaded_world();
    // 悬空放置：下方为空
    set_voxel_at_world(IVec3::new(1, 3, 1), SAND, &mut world).unwrap();
    let mut runtime = ChunkRuntime::default();
    let mut buffer = VoxelChangeBuffer::default();
    let dummy_world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &dummy_world);

    let behavior = FallingBlockBehavior;
    behavior.on_change(
        IVec3::new(1, 3, 1),
        SAND,
        &world,
        &mut buffer,
        &mut commands,
    );

    assert_eq!(buffer.0.len(), 2);
    apply_changes(
        &mut world,
        &mut runtime,
        &mut VoxelProvenance::default(),
        &mut buffer,
    );
    assert_eq!(get_voxel_at_world(IVec3::new(1, 3, 1), &world), 0);
    assert_eq!(get_voxel_at_world(IVec3::new(1, 2, 1), &world), SAND);
}

#[test]
fn sand_column_settles_after_repeated_steps() {
    let mut world = loaded_world();
    let mut runtime = ChunkRuntime::default();
    // 沙柱 (2,3,4)，下方 (1) 是石头支撑
    set_voxel_at_world(IVec3::new(1, 1, 1), STONE, &mut world).unwrap();
    for y in 2..=4 {
        set_voxel_at_world(IVec3::new(1, y, 1), SAND, &mut world).unwrap();
    }
    // 挖掉支撑
    set_voxel_at_world(IVec3::new(1, 1, 1), 0, &mut world).unwrap();

    let behavior = FallingBlockBehavior;
    let dummy_world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &dummy_world);
    // 模拟 3 帧事件驱动：每帧从低到高扫描，下方为空的沙触发下落
    for _ in 0..3 {
        let mut buffer = VoxelChangeBuffer::default();
        for y in 2..=5 {
            let pos = IVec3::new(1, y, 1);
            let below = pos - IVec3::Y;
            if get_voxel_at_world(pos, &world) == SAND && get_voxel_at_world(below, &world) == 0 {
                behavior.on_neighbor_update(
                    pos,
                    SAND,
                    below,
                    0,
                    &world,
                    &mut buffer,
                    &mut commands,
                );
            }
        }
        apply_changes(
            &mut world,
            &mut runtime,
            &mut VoxelProvenance::default(),
            &mut buffer,
        );
    }

    // 沙柱整体下移到 (1,2,3)
    for y in 1..=3 {
        assert_eq!(get_voxel_at_world(IVec3::new(1, y, 1), &world), SAND);
    }
    assert_eq!(get_voxel_at_world(IVec3::new(1, 4, 1), &world), 0);
}
