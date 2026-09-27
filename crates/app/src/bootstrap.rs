//! 启动装配：世界生成 → 快照 → 交给渲染层

use crate::settings::{INITIAL_LOAD_RADIUS_CHUNKS, WORLD_SEED};
use bevy::log::info;
use bevy::prelude::*;
use cj_core::pos::{BlockPos, SectionLocalPos, SectionPos};
use cj_core::spec::SECTION_COUNT_USIZE;
use cj_core::voxel::BlockId;
use cj_render::input::SectionSnapshot;
use cj_render::resources::{RENDER_CHUNK_RADIUS, SectionSnapshots};
use cj_runtime::world_data::WorldData;
use cj_world::seed::WorldSeed;
use std::time::Instant;

pub struct BootstrapPlugin;

impl Plugin for BootstrapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(initial_world_data()).add_systems(
            Startup,
            (generate_initial_world, build_section_snapshots).chain(),
        );
    }
}

fn initial_world_data() -> WorldData {
    WorldData::new(WorldSeed::from_seed(WORLD_SEED))
}

fn generate_initial_world(mut world_data: ResMut<WorldData>) {
    info!("开始生成世界：半径 {INITIAL_LOAD_RADIUS_CHUNKS} chunk");

    let start = Instant::now();
    world_data.generate_initial_area(INITIAL_LOAD_RADIUS_CHUNKS);
    let elapsed = start.elapsed();

    let chunk_count = world_data.chunks().len();
    let non_air: u64 = world_data
        .chunks()
        .iter()
        .map(|(_, chunk)| u64::from(chunk.non_air_count()))
        .sum();

    info!("世界生成完成：chunk = {chunk_count}，非空气格 = {non_air}，耗时 = {elapsed:?}");
}

fn block_at(world: &WorldData, pos: BlockPos) -> BlockId {
    match world.chunks().get(pos.chunk_pos()) {
        Some(chunk) => chunk.get_block(pos).block(),
        None => BlockId::AIR,
    }
}

fn build_section_snapshots(world: Res<WorldData>, mut commands: Commands) {
    let start = Instant::now();
    let mut snapshots = Vec::new();

    for (chunk_pos, chunk) in world.chunks().iter() {
        if chunk_pos.x.abs() > RENDER_CHUNK_RADIUS || chunk_pos.z.abs() > RENDER_CHUNK_RADIUS {
            continue;
        }

        for index in 0..SECTION_COUNT_USIZE {
            let section_pos = SectionPos::from_index(*chunk_pos, index);

            if chunk.section(section_pos.index()).is_none() {
                continue;
            }

            let origin = section_pos.block_at(SectionLocalPos { x: 0, y: 0, z: 0 });
            let mut snapshot = SectionSnapshot::new(section_pos);

            for oy in -1..17i32 {
                for oz in -1..17i32 {
                    for ox in -1..17i32 {
                        let pos = BlockPos::new(origin.x + ox, origin.y + oy, origin.z + oz);
                        snapshot.set(pos, block_at(&world, pos));
                    }
                }
            }

            snapshots.push(snapshot);
        }
    }

    info!(
        "快照构建完成：section = {}，耗时 = {:?}",
        snapshots.len(),
        start.elapsed()
    );

    commands.insert_resource(SectionSnapshots::new(snapshots));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_world_uses_current_generator_version() {
        assert!(
            initial_world_data().seed().matches_current(),
            "初始世界必须打上当前生成器版本；此断言若失败，说明构造时硬编码了版本号，应改用 WorldSeed::from_seed"
        );
    }
}
