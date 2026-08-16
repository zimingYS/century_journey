use super::*;
use bevy::mesh::VertexAttributeValues;

#[test]
fn world_mesh_uploads_block_light_in_second_uv_channel() {
    let encoded = block_light_to_uv(LightRgb { r: 15, g: 9, b: 4 });
    let mut buffer = MeshBufferData::new();
    buffer.append_face(
        &[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        Vec3::Z,
        &[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        [1.0; 4],
        encoded,
    );

    let mesh = buffer.build_mesh();
    let Some(VertexAttributeValues::Float32x2(values)) = mesh.attribute(Mesh::ATTRIBUTE_UV_1)
    else {
        panic!("世界区块网格必须上传第二组 UV 方块光数据");
    };
    assert_eq!(values.as_slice(), &[encoded; 4]);
}

#[test]
fn face_key_round_trips_combined_and_block_light() {
    let combined = LightRgb { r: 15, g: 9, b: 4 };
    let block = LightRgb { r: 12, g: 7, b: 2 };
    let tint = LightRgb { r: 10, g: 5, b: 3 };
    let key = encode_face_key(1234, 1, combined, block, tint);

    assert_eq!(decode_face_key(key), (1234, 1, tint, combined, block));
}

#[test]
fn face_key_distinguishes_sky_light_from_equal_block_light() {
    let combined = LightRgb {
        r: 10,
        g: 10,
        b: 10,
    };
    let tint = LightRgb {
        r: 10,
        g: 10,
        b: 10,
    };
    let sky_only = encode_face_key(3, 0, combined, LightRgb::default(), tint);
    let block_only = encode_face_key(3, 0, combined, combined, tint);

    assert_ne!(sky_only, block_only);
}

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
    use crate::content::biome::BiomeRegistry;
    use crate::content::block::registry::{BlockRegistry, init_block_registry_system};
    use crate::content::validation::compile_content;
    use crate::engine::asset::AssetResolver;
    use crate::game::world::generation::block_ids::GenerationBlockIds;
    use crate::game::world::generation::pipeline::{GenerationPipeline, TerrainSurfaceSampler};
    use crate::game::world::time::Season;
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
    // 未就绪的采样器即可——水方块未配置 tint，compute_face_tint 不会读到气候。
    let tint_sampler = TerrainSurfaceSampler::pending(
        GenerationPipeline::new(0, BiomeRegistry::default()),
        GenerationBlockIds::default(),
    );

    let mut chunk = ChunkData::new();
    chunk.set_voxel(8, 8, 8, water_id);
    let result = build_greedy_mesh(MeshBuildInput {
        chunk_pos: IVec3::ZERO,
        request_entity: Entity::PLACEHOLDER,
        request_id: 1,
        current_data: Arc::new(chunk),
        neighbors: std::array::from_fn(|_| None),
        block_info,
        light: None,
        neighbor_lights: std::array::from_fn(|_| None),
        season: Season::Spring,
        tint_sampler,
    });

    assert!(!result.water.is_empty());
    assert!(result.opaque.is_empty());
    assert!(result.cutout.is_empty());
}

#[test]
fn transparent_block_routes_to_its_own_blend_buffer() {
    use crate::content::biome::BiomeRegistry;
    use crate::content::block::registry::{BlockRegistry, init_block_registry_system};
    use crate::content::validation::compile_content;
    use crate::engine::asset::AssetResolver;
    use crate::game::world::generation::block_ids::GenerationBlockIds;
    use crate::game::world::generation::pipeline::{GenerationPipeline, TerrainSurfaceSampler};
    use crate::game::world::time::Season;
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
    let glass_id = registry
        .get_id_by_identifier("century_journey:glass")
        .expect("内容注册表必须包含玻璃方块");
    let block_info = BlockInfoSnapshot::from_registry(registry);
    let tint_sampler = TerrainSurfaceSampler::pending(
        GenerationPipeline::new(0, BiomeRegistry::default()),
        GenerationBlockIds::default(),
    );

    let mut chunk = ChunkData::new();
    chunk.set_voxel(8, 8, 8, glass_id);
    let result = build_greedy_mesh(MeshBuildInput {
        chunk_pos: IVec3::ZERO,
        request_entity: Entity::PLACEHOLDER,
        request_id: 1,
        current_data: Arc::new(chunk),
        neighbors: std::array::from_fn(|_| None),
        block_info,
        light: None,
        neighbor_lights: std::array::from_fn(|_| None),
        season: Season::Spring,
        tint_sampler,
    });

    // 半透明方块必须进入独立 blend 通道，而不是不透明通道（否则黑色不透光）。
    assert!(!result.transparent.is_empty());
    assert!(result.opaque.is_empty());
    assert!(result.cutout.is_empty());
}
