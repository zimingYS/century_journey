use std::collections::HashSet;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use crate::core::constant::CHUNK_SIZE;
use crate::player::components::Player;
use crate::voxel::types::VoxelType;
use crate::world::chunk::{ChunkComponents, ChunkData, ChunkState};
use crate::world::generation::WorldGenerator;
use crate::world::storage::WorldStorage;

/// 区块加载与卸载调度
pub fn manage_chunks_system(
    mut commands: Commands,
    mut world_storage: ResMut<WorldStorage>,
    chunk_query: Query<(Entity, &ChunkComponents)>,
    player_query: Query<&Transform, With<Player>>,
){
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    // 获取玩家位置
    let player_pos = player_transform.translation;
    let size_f32 = CHUNK_SIZE as f32;
    // 玩家视野半径
    let render_distance = 8;

    let data_distance = render_distance + 1;

    // 将玩家的世界绝对坐标 (x, y, z) 换算为世界区块坐标 (IVec3)
    let player_chunk_x = (player_pos.x / size_f32).floor() as i32;
    let player_chunk_y = (player_pos.y / size_f32).floor() as i32;
    let player_chunk_z = (player_pos.z / size_f32).floor() as i32;
    let player_chunk_pos = IVec3::new(player_chunk_x, player_chunk_y, player_chunk_z);

    // 记录这一帧应该存在的所有区块坐标
    let mut expected_chunks = HashSet::new();

    // 加载范围内的区块
    for x in -data_distance..=data_distance {
        for y in -data_distance..=data_distance {
            for z in -data_distance..=data_distance {
                let chunk_pos = player_chunk_pos + IVec3::new(x, y, z);
                expected_chunks.insert(chunk_pos);

                // 检测坐标是否存在数据
                if !world_storage.loaded_chunks.contains_key(&chunk_pos)
                    && !world_storage.chunk_entities.contains_key(&chunk_pos)
                {
                    // 生成区块，标记初始状态
                    let entity = commands.spawn((
                        ChunkComponents { position: chunk_pos },
                        ChunkState::GeneratingData,
                        Transform::from_translation(Vec3::new(
                            (chunk_pos.x * CHUNK_SIZE as i32) as f32,
                            (chunk_pos.y * CHUNK_SIZE as i32) as f32,
                            (chunk_pos.z * CHUNK_SIZE as i32) as f32,
                        )),
                        Visibility::default(),
                    )).id();

                    world_storage.chunk_entities.insert(chunk_pos, entity);
                }
            }
        }
    }

    // 卸载超出范围的区块
    for (entity, chunk_components) in chunk_query.iter() {
        let pos = chunk_components.position;

        if !expected_chunks.contains(&pos) {
            commands.entity(entity).despawn();
            world_storage.chunk_entities.remove(&pos);
            // todo!("这边要加上保存到本地存档功能");
            world_storage.loaded_chunks.remove(&pos);
        }
    }
}

/// 生成区块内方块数据
pub fn generate_chunk_data_system(
    world_generator: Res<WorldGenerator>,
    mut world_storage: ResMut<WorldStorage>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
){
    for (chunk_components, mut state) in &mut chunk_query {
        if *state != ChunkState::GeneratingData {
            continue;
        }

        // 获取该区块在世界网格中的 IVec3 坐标
        let chunk_pos = chunk_components.position;
        // 调用生成器计算出方块数据
        let chunk_data = world_generator.generate_chunk_data(chunk_pos);
        // 将计算好的方块数据存入世界存储器中
        world_storage.loaded_chunks.insert(chunk_pos, chunk_data);
        // 激活下一个状态
        *state = ChunkState::DataReady;
    }
}

/// 定义6个方向的相对偏移量，以及对应的三维法线
const DIRECTIONS: [(IVec3, Vec3); 6] = [
    (IVec3::new(0, 1, 0),  Vec3::new(0.0, 1.0, 0.0)),   // 上 (Top)
    (IVec3::new(0, -1, 0), Vec3::new(0.0, -1.0, 0.0)),  // 下 (Bottom)
    (IVec3::new(-1, 0, 0), Vec3::new(-1.0, 0.0, 0.0)),  // 左 (Left)
    (IVec3::new(1, 0, 0),  Vec3::new(1.0, 0.0, 0.0)),   // 右 (Right)
    (IVec3::new(0, 0, 1),  Vec3::new(0.0, 0.0, 1.0)),   // 前 (Front)
    (IVec3::new(0, 0, -1), Vec3::new(0.0, 0.0, -1.0)),  // 后 (Back)
];

/// 构建网格系统
pub fn build_chunk_mesh_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_query:Query<(Entity, &ChunkComponents, &mut ChunkState)>,
    world_storage: Res<WorldStorage>,
){
    // 实体材质
    let opaque_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });

    // 水体材质
    let water_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 0.5, 0.8, 0.5),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.2,
        metallic: 0.1,
        ..default()
    });

    for (chunk_entity, chunk_components, mut state) in &mut chunk_query {
        if *state != ChunkState::DataReady { continue; }

        let current_chunk_pos = chunk_components.position;

        // // 跨区块检查旁边区块是否已生成区块数据
        // let neighbors_ready = DIRECTIONS.iter()
        //     .all(|(dir, _)| world_storage.loaded_chunks.contains_key(&(current_chunk_pos + *dir)));

        let neighbors_ready = DIRECTIONS.iter()
            .all(|(dir, _)| world_storage.loaded_chunks.contains_key(&(current_chunk_pos + *dir)));

        if !neighbors_ready { continue; }

        // 获取当前区块数据
        let Some(current_chunk_data) = world_storage.loaded_chunks.get(&current_chunk_pos)
            else { continue; };

        // 改变区块状态
        *state = ChunkState::GeneratingMesh;

        // 存储顶点容器
        // 实体方块容器
        let mut opaque_positions: Vec<[f32; 3]> = Vec::new();
        let mut opaque_normals: Vec<[f32; 3]> = Vec::new();
        let mut opaque_colors: Vec<[f32; 4]> = Vec::new();
        let mut opaque_indices: Vec<u32> = Vec::new();

        // 透明水体容器
        let mut water_positions: Vec<[f32; 3]> = Vec::new();
        let mut water_normals: Vec<[f32; 3]> = Vec::new();
        let mut water_colors: Vec<[f32; 4]> = Vec::new();
        let mut water_indices: Vec<u32> = Vec::new();

        for x in 0..16{
            for y in 0..16{
                for z in 0..16{
                    let voxel_id = current_chunk_data.voxels[ChunkData::xyz_to_index(x, y, z)];

                    // 是空气则不渲染
                    if voxel_id == VoxelType::Air as u8 { continue; }

                    let current_is_water = voxel_id == VoxelType::Water as u8;

                    let local_pos = IVec3::new(x as i32, y as i32, z as i32);

                    for face_idx in 0..6{
                        let (dir, normal) = DIRECTIONS[face_idx];
                        let neighbor_local_pos = local_pos + dir;

                        // 跨区块判断隔壁方块是否为空气
                        let is_neighbor_transparent = {
                            // 若隔壁方块在本区块内
                            if let Some(nbr_id) = current_chunk_data.get_voxel_safe(neighbor_local_pos.x, neighbor_local_pos.y, neighbor_local_pos.z) {
                                let nbr_is_air = nbr_id == VoxelType::Air as u8;
                                let nbr_is_water = nbr_id == VoxelType::Water as u8;

                                if current_is_water {
                                    nbr_is_air
                                } else {
                                    nbr_is_air || nbr_is_water
                                }
                            } else {
                                // 计算隔壁区块的绝对坐标
                                let neighbor_chunk_pos = current_chunk_pos + dir;
                                // 换算出邻居在它自己区块内部的局部坐标
                                let nbr_local_x = neighbor_local_pos.x.rem_euclid(16) as usize;
                                let nbr_local_y = neighbor_local_pos.y.rem_euclid(16) as usize;
                                let nbr_local_z = neighbor_local_pos.z.rem_euclid(16) as usize;

                                if let Some(neighbor_chunk_data) = world_storage.loaded_chunks.get(&neighbor_chunk_pos) {
                                    let nbr_id = neighbor_chunk_data.voxels[ChunkData::xyz_to_index(nbr_local_x, nbr_local_y, nbr_local_z)];
                                    let nbr_is_air = nbr_id == VoxelType::Air as u8;
                                    let nbr_is_water = nbr_id == VoxelType::Water as u8;

                                    if current_is_water { nbr_is_air } else { nbr_is_air || nbr_is_water }
                                } else {
                                    if current_is_water { false } else { true }
                                }
                            }
                        };

                        if is_neighbor_transparent {
                            let voxel_type = VoxelType::from_u8(voxel_id);
                            let color_array = voxel_type.get_voxel_color().to_linear().to_f32_array();
                            let face_vertices = get_face_vertices(x as f32, y as f32, z as f32, face_idx);

                            if current_is_water {
                                // 填充到水面容器
                                let start_idx = water_positions.len() as u32;
                                water_positions.extend_from_slice(&face_vertices);
                                for _ in 0..4 {
                                    water_normals.push([normal.x, normal.y, normal.z]);
                                    water_colors.push(color_array);
                                }
                                water_indices.extend_from_slice(&[
                                    start_idx + 0, start_idx + 1, start_idx + 2,
                                    start_idx + 0, start_idx + 2, start_idx + 3,
                                ]);
                            } else {
                                // 填充到实体方块容器
                                let start_idx = opaque_positions.len() as u32;
                                opaque_positions.extend_from_slice(&face_vertices);
                                for _ in 0..4 {
                                    opaque_normals.push([normal.x, normal.y, normal.z]);
                                    opaque_colors.push(color_array);
                                }
                                opaque_indices.extend_from_slice(&[
                                    start_idx + 0, start_idx + 1, start_idx + 2,
                                    start_idx + 0, start_idx + 2, start_idx + 3,
                                ]);
                            }
                        }
                    }
                }
            }
        }

        if !opaque_positions.is_empty() {
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, opaque_positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, opaque_normals);
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, opaque_colors);
            mesh.insert_indices(Indices::U32(opaque_indices));

            let mesh_handle = meshes.add(mesh);
            commands.entity(chunk_entity).insert((
                Mesh3d(mesh_handle),
                MeshMaterial3d(opaque_material.clone()),
            ));
        }

        // 重新构建网格前清除老的水面实体
        commands.entity(chunk_entity).despawn_related::<Children>();

        if !water_positions.is_empty() {
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, water_positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, water_normals);
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, water_colors);
            mesh.insert_indices(Indices::U32(water_indices));

            let water_mesh_handle = meshes.add(mesh);

            commands.entity(chunk_entity).with_children(|parent| {
                parent.spawn((
                    Mesh3d(water_mesh_handle),
                    MeshMaterial3d(water_material.clone()),
                    Transform::IDENTITY,
                    Visibility::default(),
                ));
            });
        }

        // 标记渲染就绪
        *state = ChunkState::Rendered;
    }
}

/// 计算方块局部坐标和面索引
fn get_face_vertices(x: f32, y: f32, z: f32, face_idx: usize) -> [[f32; 3]; 4] {
    let v = [
        [x,     y,     z],     // 0
        [x+1.0, y,     z],     // 1
        [x+1.0, y+1.0, z],     // 2
        [x,     y+1.0, z],     // 3
        [x,     y,     z+1.0], // 4
        [x+1.0, y,     z+1.0], // 5
        [x+1.0, y+1.0, z+1.0], // 6
        [x,     y+1.0, z+1.0], // 7
    ];

    match face_idx {
        0 => [v[2], v[3], v[7], v[6]], // 上 Top
        1 => [v[0], v[1], v[5], v[4]], // 下 Bottom
        2 => [v[0], v[4], v[7], v[3]], // 左 Left
        3 => [v[1], v[2], v[6], v[5]], // 右 Right
        4 => [v[4], v[5], v[6], v[7]], // 前 Front
        5 => [v[0], v[3], v[2], v[1]], // 后 Back
        _ => unreachable!(),
    }
}
