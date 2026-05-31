use crate::player::components::Player;
use crate::voxel::properties::RenderMode;
use crate::voxel::registry::BlockRegistry;
use crate::world::chunk::{ChunkComponents, ChunkData, ChunkState};
use crate::world::generation::noise::GenerationBlockIds;
use crate::world::generation::WorldGenerator;
use crate::world::storage::WorldStorage;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::collections::HashSet;
use crate::core::constant::world::CHUNK_SIZE;

/// 单个渲染通道的顶点缓冲区
struct MeshBuffer {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

impl MeshBuffer {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// 向缓冲区追加一个面的 4 个顶点
    fn append_face(
        &mut self,
        face_vertices: &[[f32; 3]; 4],
        normal: Vec3,
        uvs: &[[f32; 2]; 4],
    ) {
        let start_idx = self.positions.len() as u32;
        self.positions.extend_from_slice(face_vertices);
        for _ in 0..4 {
            self.normals.push([normal.x, normal.y, normal.z]);
        }
        self.uvs.extend_from_slice(uvs);
        self.indices.extend_from_slice(&[
            start_idx, start_idx + 1, start_idx + 2,
            start_idx, start_idx + 2, start_idx + 3,
        ]);
    }

    /// 从缓冲区生成 Bevy Mesh
    fn build_mesh(&self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs.clone());
        mesh.insert_indices(Indices::U32(self.indices.clone()));
        mesh
    }
}


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

    let mut spawned = 0u32;
    let max_spawn_per_frame = 16;

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
                    if spawned >= max_spawn_per_frame { continue; }
                    spawned += 1;
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
    registry: Option<Res<BlockRegistry>>,
    world_generator: Res<WorldGenerator>,
    mut world_storage: ResMut<WorldStorage>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
){
    let Some(reg) = registry else { return; };

    // 计算出噪声内循环所需的动态ID
    let block_ids = GenerationBlockIds::from_registry(&reg);

    // 每帧对多渲染的区块数
    let mut generated = 0u32;
    let max_per_frame = 16;

    for (chunk_components, mut state) in &mut chunk_query {
        if *state != ChunkState::GeneratingData {
            continue;
        }

        if generated >= max_per_frame { break; }

        // 获取该区块在世界网格中的 IVec3 坐标
        let chunk_pos = chunk_components.position;
        // 调用生成器计算出方块数据
        let chunk_data = world_generator.generate_chunk_data(chunk_pos, block_ids);
        // 将计算好的方块数据存入世界存储器中
        world_storage.loaded_chunks.insert(chunk_pos, chunk_data);
        // 激活下一个状态
        *state = ChunkState::DataReady;
        generated += 1;
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
    mut chunk_query: Query<(Entity, &ChunkComponents, &mut ChunkState)>,
    registry: Option<Res<BlockRegistry>>,
    world_storage: Res<WorldStorage>,
) {
    let Some(reg) = registry else { return; };

    let water_id = reg.get_id_by_identifier("century_journey:water").unwrap_or(0);
    let total_layers = reg.texture_layers.values().map(|&v| v + 1).max().unwrap_or(1);

    let mut built = 0u32;
    let max_per_frame = 8;

    for (chunk_entity, chunk_components, mut state) in &mut chunk_query {
        if !world_storage.chunk_entities.contains_key(&chunk_components.position) {
            continue;
        }

        if *state != ChunkState::DataReady { continue; }

        if built >= max_per_frame { break; }

        let current_chunk_pos = chunk_components.position;

        // 跨区块检查邻居区块是否已生成数据
        let neighbors_ready = DIRECTIONS.iter()
            .all(|(dir, _)| world_storage.loaded_chunks.contains_key(&(current_chunk_pos + *dir)));
        if !neighbors_ready { continue; }

        let Some(current_chunk_data) = world_storage.loaded_chunks.get(&current_chunk_pos)
        else { continue; };

        *state = ChunkState::GeneratingMesh;

        // 三通道缓冲区
        let mut opaque_buf = MeshBuffer::new();
        let mut cutout_buf = MeshBuffer::new();
        let mut water_buf = MeshBuffer::new();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let voxel_id = current_chunk_data.get_voxel(x, y, z);
                    if voxel_id == 0 { continue; }

                    let current_is_water = voxel_id == water_id;
                    let local_pos = IVec3::new(x as i32, y as i32, z as i32);
                    let Some(prop) = reg.get(voxel_id) else { continue; };

                    for face_idx in 0..6 {
                        let (dir, normal) = DIRECTIONS[face_idx];
                        let neighbor_local_pos = local_pos + dir;

                        // 面剔除判断（统一逻辑，不再重复）
                        let is_visible = match get_neighbor_voxel_id(
                            neighbor_local_pos,
                            current_chunk_data,
                            current_chunk_pos,
                            &world_storage,
                            dir,
                        ) {
                            Some(nbr_id) => is_face_visible(current_is_water, nbr_id, &*reg),
                            None => !current_is_water, // 未加载的邻居区块边界
                        };

                        if !is_visible { continue; }

                        let face_vertices = get_face_vertices(x as f32, y as f32, z as f32, face_idx);
                        let layer_id = reg.texture_layers.get(&(voxel_id, face_idx)).copied().unwrap_or(0);
                        let uvs = compute_face_uvs(layer_id, total_layers);

                        if current_is_water {
                            water_buf.append_face(&face_vertices, normal, &uvs);
                        } else if prop.render_mode == RenderMode::Cutout {
                            cutout_buf.append_face(&face_vertices, normal, &uvs);
                        } else {
                            opaque_buf.append_face(&face_vertices, normal, &uvs);
                        }
                    }
                }
            }
        }

        // 清理旧的子实体
        commands.entity(chunk_entity)
            .remove::<Mesh3d>()
            .remove::<MeshMaterial3d<StandardMaterial>>();
        commands.entity(chunk_entity).despawn_related::<Children>();

        // 不透明方块 — 直接挂在区块实体上
        if !opaque_buf.is_empty() {
            commands.entity(chunk_entity).insert((
                Mesh3d(meshes.add(opaque_buf.build_mesh())),
                MeshMaterial3d(reg.opaque_material.clone()),
            ));
        }

        // 半透明方块 — 子实体
        if !cutout_buf.is_empty() {
            let child = commands.spawn((
                Mesh3d(meshes.add(cutout_buf.build_mesh())),
                MeshMaterial3d(reg.cutout_material.clone()),
                Transform::IDENTITY,
                Visibility::default(),
            )).id();
            commands.entity(chunk_entity).add_child(child);
        }

        // 水面 — 子实体
        if !water_buf.is_empty() {
            let child = commands.spawn((
                Mesh3d(meshes.add(water_buf.build_mesh())),
                MeshMaterial3d(reg.transparent_material.clone()),
                Transform::IDENTITY,
                Visibility::default(),
            )).id();
            commands.entity(chunk_entity).add_child(child);
        }

        *state = ChunkState::Rendered;
        built += 1;
    }
}

/// 判断某个面是否需要渲染（邻居是否透明）
fn is_face_visible(
    current_is_water: bool,
    neighbor_voxel_id: u16,
    registry: &BlockRegistry,
) -> bool {
    if neighbor_voxel_id == 0 {
        // 空气不渲染
        return true;
    }
    if current_is_water {
        // 水旁边不渲染水面
        return false;
    }
    let nbr_is_solid = registry.get(neighbor_voxel_id).map(|p| p.is_solid).unwrap_or(true);
    !nbr_is_solid || neighbor_voxel_id == registry.get_id_by_identifier("century_journey:water").unwrap_or(0)
}

/// 获取邻居方块的ID
fn get_neighbor_voxel_id(
    neighbor_local_pos: IVec3,
    current_chunk_data: &ChunkData,
    current_chunk_pos: IVec3,
    world_storage: &WorldStorage,
    dir: IVec3,
) -> Option<u16> {
    // 先尝试在本区块内查找
    if let Some(nbr_id) = current_chunk_data.get_voxel_safe(
        neighbor_local_pos.x,
        neighbor_local_pos.y,
        neighbor_local_pos.z,
    ) {
        return Some(nbr_id);
    }
    // 跨区块查找
    let neighbor_chunk_pos = current_chunk_pos + dir;
    let nbr_local_x = neighbor_local_pos.x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let nbr_local_y = neighbor_local_pos.y.rem_euclid(CHUNK_SIZE as i32) as usize;
    let nbr_local_z = neighbor_local_pos.z.rem_euclid(CHUNK_SIZE as i32) as usize;

    world_storage
        .loaded_chunks
        .get(&neighbor_chunk_pos)
        .map(|data| data.get_voxel(nbr_local_x, nbr_local_y, nbr_local_z))
}


/// 计算某个面在图集中的 UV 坐标
fn compute_face_uvs(layer_id: u32, total_layers: u32) -> [[f32; 2]; 4] {
    let u0 = layer_id as f32 / total_layers as f32;
    let u1 = (layer_id + 1) as f32 / total_layers as f32;
    [
        [u0, 0.0],
        [u1, 0.0],
        [u1, 1.0],
        [u0, 1.0],
    ]
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
        2 => [v[7], v[3], v[0], v[4]], // 左 Left
        3 => [v[2], v[6], v[5], v[1]], // 右 Right
        4 => [v[6], v[7], v[4], v[5]], // 前 Front
        5 => [v[3], v[2], v[1], v[0]], // 后 Back
        _ => unreachable!(),
    }
}
