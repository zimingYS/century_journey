use super::*;

#[test]
fn water_top_is_lower_than_adjacent_solid_blocks() {
    let (mut positions, _) = get_merged_face_data(0, 0, 0, 1, 1, 0, 1, 0, 2, 0, 1, true);
    inset_water_surface(&mut positions, 0);
    assert!(
        positions
            .iter()
            .all(|position| (position[1] - (1.0 - WATER_SURFACE_INSET)).abs() < 0.0001)
    );
}

#[test]
fn water_side_keeps_its_bottom_and_lowers_only_the_top_edge() {
    let (mut positions, _) = get_merged_face_data(0, 0, 0, 1, 1, 2, 0, 2, 1, 0, 1, true);
    inset_water_surface(&mut positions, 2);
    let top_vertices = positions
        .iter()
        .filter(|position| position[1] > 0.0)
        .count();
    let bottom_vertices = positions
        .iter()
        .filter(|position| position[1] == 0.0)
        .count();
    assert_eq!(top_vertices, 2);
    assert_eq!(bottom_vertices, 2);
}
#[test]
fn water_voxel_builds_a_visible_water_mesh_channel() {
    use crate::content::block::registry::{BlockRegistry, init_block_registry_system};
    use crate::content::validation::compile_content;
    use crate::engine::asset::AssetResolver;
    use crate::shared::states::AppState;
    use bevy::state::app::StatesPlugin;

    let resolver =
        AssetResolver::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"));
    let compilation = compile_content(&resolver);
    assert!(compilation.is_valid());

    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .init_state::<AppState>()
        .insert_resource(compilation)
        .add_systems(Update, init_block_registry_system);
    app.update();

    let registry = app.world().resource::<BlockRegistry>();
    let water_id = registry
        .get_id_by_identifier("century_journey:water")
        .expect("内容注册表必须包含水方块");
    let block_info = BlockInfoSnapshot::from_registry(registry);

    let mut chunk = ChunkData::new();
    chunk.set_voxel(8, 8, 8, water_id);
    let result = build_greedy_mesh(MeshBuildInput {
        chunk_pos: IVec3::ZERO,
        current_data: Arc::new(chunk),
        neighbors: std::array::from_fn(|_| None),
        block_info,
    });

    assert!(!result.water.is_empty());
    assert!(result.opaque.is_empty());
    assert!(result.cutout.is_empty());
}
