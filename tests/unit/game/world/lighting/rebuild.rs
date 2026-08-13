use super::*;
use crate::game::world::chunk::ChunkData;
use std::sync::Arc;

// 测试 ID：0=空气，1=石头，2=暖白光源，3=红玻璃，4=红光源，5=绿光源。
fn test_info() -> GameLightInfo {
    GameLightInfo {
        props: vec![
            VoxelLightProp {
                filter: [1.0; 3],
                light: None,
            },
            VoxelLightProp::default(),
            VoxelLightProp {
                filter: [1.0; 3],
                light: Some(BlockLightDef {
                    emission: 15,
                    color: [1.0, 0.62, 0.28],
                    range: 14,
                    casts_shadow: true,
                }),
            },
            VoxelLightProp {
                filter: [1.0, 0.0, 0.0],
                light: None,
            },
            VoxelLightProp {
                filter: [1.0; 3],
                light: Some(BlockLightDef {
                    emission: 15,
                    color: [1.0, 0.0, 0.0],
                    range: 14,
                    casts_shadow: false,
                }),
            },
            VoxelLightProp {
                filter: [1.0; 3],
                light: Some(BlockLightDef {
                    emission: 15,
                    color: [0.0, 1.0, 0.0],
                    range: 14,
                    casts_shadow: false,
                }),
            },
        ],
        max_block_range: 14,
    }
}

fn state_with_air_chunk() -> WorldState {
    let mut state = WorldState::default();
    state.insert_chunk(IVec3::ZERO, Arc::new(ChunkData::new()));
    state
}

fn set_block(state: &mut WorldState, position: IVec3, id: u16) {
    let (chunk_pos, local) = split_world(position);
    let data = Arc::make_mut(state.chunk_mut(chunk_pos).unwrap());
    data.set_voxel(local.x as usize, local.y as usize, local.z as usize, id);
}

/// 等价旧的 `sky_dirty=true`：快照内所有水平列都标记为需要重灌天光。
fn all_sky_dirty_columns(snapshot: &LightingWorldSnapshot) -> HashSet<(i32, i32)> {
    snapshot
        .chunks()
        .map(|(position, _)| (position.x, position.z))
        .collect()
}

fn rebuild(world: &WorldState, info: &GameLightInfo) -> HashMap<IVec3, ChunkLight> {
    let mut lights = HashMap::new();
    let snapshot = LightingWorldSnapshot::from_world(world);
    let dirty = all_sky_dirty_columns(&snapshot);
    rebuild_loaded_lighting(&snapshot, info, &mut lights, &dirty);
    lights
}

fn light_at(lights: &HashMap<IVec3, ChunkLight>, position: IVec3) -> LightCell {
    let (chunk_pos, local) = split_world(position);
    lights[&chunk_pos].get(local.x as usize, local.y as usize, local.z as usize)
}

#[test]
fn air_transmits_sky_and_opaque_block_stops_it() {
    let mut world = state_with_air_chunk();
    set_block(&mut world, IVec3::new(8, 14, 8), 1);
    let lights = rebuild(&world, &test_info());

    assert_eq!(light_at(&lights, IVec3::new(8, 15, 8)).sky.r, 15);
    assert!(light_at(&lights, IVec3::new(8, 14, 8)).sky.is_dark());
    assert!(lights[&IVec3::ZERO].is_initialized());
}

#[test]
fn unloaded_space_above_terrain_is_not_treated_as_open_sky() {
    assert!(initial_vertical_sky(47, Some(64)).is_dark());
    assert_eq!(initial_vertical_sky(79, Some(64)).r, 15);
}

#[test]
fn block_light_falls_off_and_respects_range() {
    let mut world = state_with_air_chunk();
    let source_pos = IVec3::new(8, 8, 8);
    set_block(&mut world, source_pos, 2);
    let mut info = test_info();
    info.props[2].light.as_mut().unwrap().range = 2;
    let lights = rebuild(&world, &info);

    assert_eq!(light_at(&lights, source_pos).block.r, 15);
    assert_eq!(light_at(&lights, source_pos + IVec3::X * 2).block.r, 5);
    assert_eq!(light_at(&lights, source_pos + IVec3::X * 3).block.r, 0);
}

#[test]
fn warm_light_edge_does_not_gain_green_or_blue_channels() {
    let source = LightRgb::from_emission(15, [1.0, 0.62, 0.28]);
    let edge = block_level_at_distance(source, source, 14, 14);

    assert_eq!(edge.r, 1);
    assert_eq!(edge.g, 0);
    assert_eq!(edge.b, 0);
}

#[test]
fn distance_limit_preserves_a_filtered_light_hue() {
    let filtered = LightRgb { r: 0, g: 12, b: 6 };
    let limited = limit_light_peak(filtered, 4);

    assert_eq!(limited, LightRgb { r: 0, g: 4, b: 2 });
}

#[test]
fn low_emission_can_reach_its_independent_declared_range() {
    let mut world = state_with_air_chunk();
    world.insert_chunk(IVec3::X, Arc::new(ChunkData::new()));
    let source_pos = IVec3::new(2, 8, 8);
    set_block(&mut world, source_pos, 2);
    let mut info = test_info();
    let light = info.props[2].light.as_mut().unwrap();
    light.emission = 4;
    light.range = 10;
    let lights = rebuild(&world, &info);

    assert!(light_at(&lights, source_pos + IVec3::X * 10).block.r > 0);
    assert_eq!(light_at(&lights, source_pos + IVec3::X * 11).block.r, 0);
}

#[test]
fn content_range_selects_the_required_chunk_halo() {
    let mut info = test_info();
    assert_eq!(info.block_light_chunk_halo(), 1);

    info.max_block_range = 32;
    assert_eq!(info.block_light_chunk_halo(), 2);
}

#[test]
fn red_and_green_sources_mix_independent_channels() {
    let mut world = state_with_air_chunk();
    set_block(&mut world, IVec3::new(4, 8, 8), 4);
    set_block(&mut world, IVec3::new(12, 8, 8), 5);
    let lights = rebuild(&world, &test_info());
    let middle = light_at(&lights, IVec3::new(8, 8, 8)).block;

    assert!(middle.r > 0);
    assert!(middle.g > 0);
    assert_eq!(middle.b, 0);
}

#[test]
fn block_light_can_route_around_an_opaque_obstacle() {
    let mut world = state_with_air_chunk();
    let source = IVec3::new(4, 8, 8);
    let target = IVec3::new(8, 8, 8);
    set_block(&mut world, source, 4);
    set_block(&mut world, IVec3::new(6, 8, 8), 1);
    let lights = rebuild(&world, &test_info());

    assert!(light_at(&lights, target).block.r > 0);
    assert!(light_at(&lights, IVec3::new(6, 8, 8)).block.is_dark());
}

#[test]
fn red_glass_filters_block_light() {
    let mut world = state_with_air_chunk();
    let source_pos = IVec3::new(8, 8, 8);
    set_block(&mut world, source_pos, 2);
    set_block(&mut world, source_pos + IVec3::Y, 3);
    for y in 7..=11 {
        for direction in [IVec3::X, IVec3::NEG_X, IVec3::Z, IVec3::NEG_Z] {
            set_block(&mut world, IVec3::new(8, y, 8) + direction, 1);
        }
    }
    let mut info = test_info();
    info.props[2].light.as_mut().unwrap().range = 4;
    let lights = rebuild(&world, &info);
    let filtered = light_at(&lights, source_pos + IVec3::Y * 2).block;

    assert!(filtered.r > 0);
    assert_eq!(filtered.g, 0);
    assert_eq!(filtered.b, 0);
}

#[test]
fn red_glass_filters_vertical_sky_light() {
    let mut world = state_with_air_chunk();
    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if x != 8 || z != 8 {
                    set_block(&mut world, IVec3::new(x as i32, y as i32, z as i32), 1);
                }
            }
        }
    }
    set_block(&mut world, IVec3::new(8, 14, 8), 3);
    let lights = rebuild(&world, &test_info());
    let filtered = light_at(&lights, IVec3::new(8, 13, 8)).sky;

    assert!(filtered.r > 0);
    assert_eq!(filtered.g, 0);
    assert_eq!(filtered.b, 0);
}

#[test]
fn block_only_rebuild_preserves_sky_and_refreshes_block_light() {
    let mut world = state_with_air_chunk();
    let source_pos = IVec3::new(8, 8, 8);
    set_block(&mut world, source_pos, 2);
    let snapshot = LightingWorldSnapshot::from_world(&world);
    let mut lights = HashMap::new();
    let dirty = all_sky_dirty_columns(&snapshot);
    rebuild_loaded_lighting(&snapshot, &test_info(), &mut lights, &dirty);
    let sky_before = light_at(&lights, source_pos + IVec3::Y).sky;
    assert!(!light_at(&lights, source_pos + IVec3::X).block.is_dark());

    set_block(&mut world, source_pos, 0);
    let snapshot = LightingWorldSnapshot::from_world(&world);
    rebuild_loaded_lighting(&snapshot, &test_info(), &mut lights, &HashSet::new());

    assert_eq!(light_at(&lights, source_pos + IVec3::Y).sky, sky_before);
    assert!(light_at(&lights, source_pos + IVec3::X).block.is_dark());
}

#[test]
fn indexed_sources_match_scanned_sources_for_local_rebuild() {
    let mut world = state_with_air_chunk();
    let red_position = IVec3::new(4, 8, 8);
    let green_position = IVec3::new(12, 8, 8);
    set_block(&mut world, red_position, 4);
    set_block(&mut world, green_position, 5);
    let info = test_info();
    let snapshot = LightingWorldSnapshot::from_world(&world);
    let dirty = all_sky_dirty_columns(&snapshot);
    let mut scanned_lights = HashMap::new();
    let scanned_sources = rebuild_loaded_lighting(&snapshot, &info, &mut scanned_lights, &dirty);

    let mut indexed_lights = HashMap::new();
    let indexed_sources = rebuild_loaded_lighting_from_source_index(
        &snapshot,
        &info,
        &mut indexed_lights,
        &dirty,
        &scanned_sources,
    );

    assert_eq!(indexed_sources, scanned_sources);
    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let position = IVec3::new(x as i32, y as i32, z as i32);
                assert_eq!(
                    light_at(&indexed_lights, position),
                    light_at(&scanned_lights, position),
                );
            }
        }
    }
}

#[test]
fn clean_column_keeps_previous_sky_when_neighbor_column_is_dirty() {
    let mut world = state_with_air_chunk();
    world.insert_chunk(IVec3::X, Arc::new(ChunkData::new()));
    // X 列顶部整层石头：直射天光全挡，y=14 只能靠 ZERO 列水平扩散。
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            set_block(&mut world, IVec3::new(16 + x as i32, 15, z as i32), 1);
        }
    }
    let snapshot = LightingWorldSnapshot::from_world(&world);
    let mut lights = HashMap::new();
    let dirty_all = all_sky_dirty_columns(&snapshot);
    rebuild_loaded_lighting(&snapshot, &test_info(), &mut lights, &dirty_all);
    // X 列边界格从 ZERO 满光水平扩散获得约 14 级天光。
    let x_sky_before = light_at(&lights, IVec3::new(16, 14, 8)).sky;
    assert!(x_sky_before.r >= 12);

    // 现在 ZERO 列顶部也加石头（天空通路变化），只重灌 ZERO 列。
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            set_block(&mut world, IVec3::new(x as i32, 15, z as i32), 1);
        }
    }
    let snapshot = LightingWorldSnapshot::from_world(&world);
    let dirty_zero = [(0, 0)].into_iter().collect::<HashSet<_>>();
    rebuild_loaded_lighting(&snapshot, &test_info(), &mut lights, &dirty_zero);

    // X 列是 clean 列：天光必须原样保留；若被误 reset，会按新 ZERO 的暗种子
    // 重灌成 0，从而暴露回归。
    assert_eq!(light_at(&lights, IVec3::new(16, 14, 8)).sky, x_sky_before);
}

#[test]
fn source_crosses_initialized_chunk_boundary() {
    let mut world = state_with_air_chunk();
    world.insert_chunk(IVec3::X, Arc::new(ChunkData::new()));
    set_block(&mut world, IVec3::new(15, 8, 8), 4);
    let lights = rebuild(&world, &test_info());

    assert!(light_at(&lights, IVec3::new(16, 8, 8)).block.r > 0);
}

#[test]
fn local_snapshot_rejects_changed_or_new_neighborhood_chunks() {
    let target = IVec3::ZERO;
    let neighbor = IVec3::X;
    let mut world = WorldState::default();
    world.insert_chunk(target, Arc::new(ChunkData::new()));
    world.insert_chunk(neighbor, Arc::new(ChunkData::new()));
    let columns = [(0, 0), (1, 0)].into_iter().collect();
    let snapshot = LightingWorldSnapshot::from_columns(&world, &columns);
    assert!(snapshot.neighborhood_is_current(&world, target, 1));

    Arc::make_mut(world.chunk_mut(neighbor).unwrap()).set_voxel(0, 0, 0, 1);
    assert!(!snapshot.neighborhood_is_current(&world, target, 1));

    let snapshot = LightingWorldSnapshot::from_columns(&world, &columns);
    world.insert_chunk(IVec3::new(0, 1, 0), Arc::new(ChunkData::new()));
    assert!(!snapshot.neighborhood_is_current(&world, target, 1));
}
