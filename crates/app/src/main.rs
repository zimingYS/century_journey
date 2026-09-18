//! CenturyJourney —— 程序入口

use bevy::log::info;
use bevy::prelude::*;
use cj_core::pos::{BlockPos, SectionLocalPos, SectionPos};
use cj_core::spec::{SECTION_COUNT_USIZE, WORLD_BOTTOM, WORLD_TOP};
use cj_core::voxel::BlockId;
use cj_render::input::SectionSnapshot;
use cj_render::resources::{RENDER_CHUNK_RADIUS, SectionSnapshots};
use cj_runtime::world_data::{WorldData, generate_initial_world};
use std::time::Instant;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(cj_runtime::RuntimePlugin)
        .add_plugins(cj_render::RenderPlugin)
        .add_plugins(cj_game::GamePlugin)
        .add_systems(
            Startup,
            build_section_snapshots.after(generate_initial_world),
        )
        .run();
}

fn block_at(world: &WorldData, pos: BlockPos) -> BlockId {
    if pos.y < WORLD_BOTTOM || pos.y >= WORLD_TOP {
        return BlockId::AIR;
    }

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
