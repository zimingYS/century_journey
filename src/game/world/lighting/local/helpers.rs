//! 局部光照调度与目标选择的纯辅助函数。

use std::collections::HashSet;

use bevy::prelude::*;

use crate::content::block::definition::BlockLightDef;
use crate::content::block::event::BlockChangedEvent;
use crate::game::world::chunk::ChunkState;
use crate::game::world::lighting::chunk_light::ChunkLight;
use crate::game::world::lighting::local::constants::{
    LOCAL_INTERACTION_COLUMN_BATCH_LIMIT, LOCAL_LIGHTING_MAX_IN_FLIGHT,
    LOCAL_LIGHTING_MIN_IN_FLIGHT, LOCAL_TARGET_COLUMN_BATCH_SIZE,
};
use crate::game::world::lighting::local_queue::LocalLightingQueue;
use crate::game::world::lighting::rebuild::{BlockLightSource, GameLightInfo};
use crate::game::world::lighting::resources::WorldLighting;
use crate::game::world::state::{ChunkRuntime, WorldState};
use crate::game::world::streaming::PlayerChunkCache;
use crate::shared::voxel::CHUNK_SIZE;

/// 把方块编辑位置及光环内所有已加载区块加入编辑优先队列。
pub(super) fn enqueue_block_change_targets(
    world: &WorldState,
    world_pos: IVec3,
    halo: i32,
    sky_dirty: bool,
    queue: &mut LocalLightingQueue,
) {
    let center = world_to_chunk(world_pos);
    let mut positions = world
        .chunks()
        .map(|(position, _)| position)
        .filter(|position| {
            (position.x - center.x).abs() <= halo && (position.z - center.z).abs() <= halo
        })
        .collect::<Vec<_>>();
    positions.sort_by_key(|position| {
        let delta = *position - center;
        (delta.x.abs() + delta.y.abs() + delta.z.abs(), delta.y.abs())
    });
    for position in positions.into_iter().rev() {
        queue.prioritize_edit(position, sky_dirty);
    }
}

/// 只有方块的天空光透射发生变化时才需要重新灌入整列天光。
///
/// 发光属性变化只影响方块光；同透射材质间替换也不会改变天空通路。
pub(super) fn edit_affects_sky(
    info: &GameLightInfo,
    lighting: &WorldLighting,
    world: &WorldState,
    change: &BlockChangedEvent,
) -> bool {
    const NEIGHBORS: [IVec3; 7] = [
        IVec3::ZERO,
        IVec3::X,
        IVec3::NEG_X,
        IVec3::Y,
        IVec3::NEG_Y,
        IVec3::Z,
        IVec3::NEG_Z,
    ];
    let old_filter = info.prop(change.old_block_id).filter;
    let new_filter = info.prop(change.new_block_id).filter;
    if old_filter == new_filter {
        return false;
    }
    if NEIGHBORS.into_iter().any(|offset| {
        lighting
            .light_cell_at_world(change.world_pos + offset)
            .is_some_and(|cell| !cell.sky.is_dark())
    }) {
        return true;
    }

    // 打开原本封闭的天井时，改动格及其邻域仍可能全黑；向上检查同列的已加载
    // 区块，只有确实能接到现有天空光柱时才升级为天光重建。未加载空间不会被当作露天。
    let opens_transmission = new_filter
        .iter()
        .zip(old_filter)
        .any(|(new, old)| *new > old);
    if !opens_transmission {
        return false;
    }
    let max_chunk_y = world
        .chunks()
        .filter(|(position, _)| {
            position.x == change.world_pos.x.div_euclid(CHUNK_SIZE as i32)
                && position.z == change.world_pos.z.div_euclid(CHUNK_SIZE as i32)
        })
        .map(|(position, _)| position.y)
        .max();
    let Some(max_chunk_y) = max_chunk_y else {
        return false;
    };
    let max_y = (max_chunk_y + 1) * CHUNK_SIZE as i32 - 1;
    let mut position = change.world_pos + IVec3::Y;
    while position.y <= max_y {
        if lighting
            .light_cell_at_world(position)
            .is_some_and(|cell| !cell.sky.is_dark())
        {
            return true;
        }
        position.y += 1;
    }
    false
}

/// 计算目标区块及其光环覆盖的全部水平列集合。
pub(super) fn dependency_columns(targets: &[IVec3], halo: i32) -> HashSet<(i32, i32)> {
    let mut columns = HashSet::new();
    for target in targets {
        for x in -halo..=halo {
            for z in -halo..=halo {
                columns.insert((target.x + x, target.z + z));
            }
        }
    }
    columns
}

/// 返回本轮任务允许取出的水平列上限。
pub(super) fn local_column_batch_size(interaction: bool, dependency_halo: i32) -> usize {
    if !interaction {
        return LOCAL_TARGET_COLUMN_BATCH_SIZE;
    }
    let diameter = dependency_halo.max(0) as usize * 2 + 1;
    diameter
        .saturating_mul(diameter)
        .clamp(1, LOCAL_INTERACTION_COLUMN_BATCH_LIMIT)
}

/// 普通局部任务受固定并发上限约束；交互可临时多占一个槽，但不能无界超发。
///
/// 基础并发随工作线程数在 [`LOCAL_LIGHTING_MIN_IN_FLIGHT`] 与
/// [`LOCAL_LIGHTING_MAX_IN_FLIGHT`] 之间伸缩：区块流送阶段网格任务仍被光照闸门阻塞，
/// 线程空闲时可并行处理更多不相交列集，从而缩短区块进入 `LightingReady` 的时间。
pub(super) fn local_lighting_slot_available(
    in_flight: usize,
    interaction: bool,
    worker_count: usize,
) -> bool {
    let base_limit = worker_count
        .saturating_sub(2)
        .clamp(LOCAL_LIGHTING_MIN_IN_FLIGHT, LOCAL_LIGHTING_MAX_IN_FLIGHT);
    let limit = base_limit + usize::from(interaction);
    in_flight < limit
}

/// 判断目标区块及光环邻域是否都已生成完毕。
pub(super) fn neighborhood_generation_ready(
    world: &WorldState,
    target: IVec3,
    runtime: &ChunkRuntime,
    states: &Query<&ChunkState>,
    player_cache: &PlayerChunkCache,
    halo: i32,
) -> bool {
    player_cache
        .ordered_chunks()
        .iter()
        .copied()
        .filter(|position| {
            (position.x - target.x).abs() <= halo && (position.z - target.z).abs() <= halo
        })
        .all(|position| {
            world.contains_chunk(position)
                && runtime
                    .chunk_entity(position)
                    .and_then(|entity| states.get(entity).ok())
                    .is_some_and(|state| {
                        matches!(*state, ChunkState::LoadFailed) || state.has_completed_structure()
                    })
        })
}

/// 判断单个区块是否已完成结构生成。
pub(super) fn chunk_generation_ready(
    position: IVec3,
    runtime: &ChunkRuntime,
    states: &Query<&ChunkState>,
) -> bool {
    runtime
        .chunk_entity(position)
        .and_then(|entity| states.get(entity).ok())
        .is_some_and(|state| state.has_completed_structure())
}

/// 维护点光源索引，返回索引是否发生变化。
pub(super) fn update_source_entry(
    sources: &mut Vec<BlockLightSource>,
    world_pos: IVec3,
    light: Option<BlockLightDef>,
) -> bool {
    let previous = sources
        .iter()
        .find(|source| source.world_pos == world_pos)
        .copied();
    let next = light.map(|light| BlockLightSource { world_pos, light });
    if previous == next {
        return false;
    }
    sources.retain(|source| source.world_pos != world_pos);
    if let Some(source) = next {
        sources.push(source);
        sources.sort_by_key(|source| (source.world_pos.x, source.world_pos.y, source.world_pos.z));
    }
    true
}

/// 比较两个光数组的初始化状态与指纹是否一致。
pub(super) fn same_light(left: &ChunkLight, right: &ChunkLight) -> bool {
    left.is_initialized() == right.is_initialized() && left.fingerprint() == right.fingerprint()
}

/// 把整数世界坐标换算为区块坐标。
pub(super) fn world_to_chunk(position: IVec3) -> IVec3 {
    IVec3::new(
        position.x.div_euclid(CHUNK_SIZE as i32),
        position.y.div_euclid(CHUNK_SIZE as i32),
        position.z.div_euclid(CHUNK_SIZE as i32),
    )
}
