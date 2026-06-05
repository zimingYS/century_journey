use crate::core::constant::world::CHUNK_SIZE;
use crate::player::components::Player;
use crate::voxel::properties::RenderMode;
use crate::voxel::registry::BlockRegistry;
use crate::world::chunk::{ChunkComponents, ChunkData, ChunkState};
use crate::world::generation::noise::{CachedBlockIds, GenerationBlockIds};
use crate::world::generation::structure::StructureGenerator;
use crate::world::generation::WorldGenerator;
use crate::world::save;
use crate::world::save::format::SavedChunk;
use crate::world::save::system::{SaveConfig, SaveQueue};
use crate::world::storage::WorldStorage;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::collections::HashSet;
use std::sync::{mpsc, Mutex};
use bevy::tasks::AsyncComputeTaskPool;

/// 地形生成异步任务的结果
pub struct TerrainGenResult {
    pub chunk_pos: IVec3,
    pub chunk_data: ChunkData,
}

/// 地形生成通道资源
#[derive(Resource)]
pub struct TerrainGenChannel {
    pub sender: mpsc::Sender<TerrainGenResult>,
    pub receiver: Mutex<mpsc::Receiver<TerrainGenResult>>,
}

impl Default for TerrainGenChannel {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
        }
    }
}


/// 单个渲染通道的顶点缓冲区
struct MeshBufferData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl MeshBufferData {
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
    fn build_mesh(mut self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, std::mem::take(&mut self.positions));
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, std::mem::take(&mut self.normals));
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, std::mem::take(&mut self.uvs));
        mesh.insert_indices(Indices::U32(std::mem::take(&mut self.indices)));
        mesh
    }
}
/// Mesh构建异步任务的结果
pub struct MeshBuildResult {
    pub chunk_pos: IVec3,
    pub opaque: MeshBufferData,
    pub cutout: MeshBufferData,
    pub water: MeshBufferData,
}

/// Mesh 构建通道资源
#[derive(Resource)]
pub struct MeshBuildChannel {
    pub sender: mpsc::Sender<MeshBuildResult>,
    pub receiver: Mutex<mpsc::Receiver<MeshBuildResult>>,
}

impl Default for MeshBuildChannel {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
        }
    }
}

/// 方块信息查找表
#[derive(Clone)]
struct BlockInfoSnapshot {
    is_solid: Vec<bool>,
    render_modes: Vec<RenderMode>,
    texture_layers: std::collections::HashMap<(u16, usize), u32>,
    water_id: u16,
    total_layers: u32,
}

impl BlockInfoSnapshot {
    /// 复制当前的方块注册表配置用于异步多线程共享
    fn from_registry(registry: &BlockRegistry) -> Self {
        let water_id = registry.get_id_by_identifier("century_journey:water").unwrap_or(0);
        let total_layers = registry.texture_layers.values().map(|&v| v + 1).max().unwrap_or(1);

        let max_id = registry.id_to_properties.keys().copied().max().unwrap_or(0);
        let mut is_solid = vec![false; (max_id + 1) as usize];
        let mut render_modes = vec![RenderMode::Opaque; (max_id + 1) as usize];

        for (&id, prop) in &registry.id_to_properties {
            is_solid[id as usize] = prop.is_solid;
            render_modes[id as usize] = prop.render_mode;
        }

        Self {
            is_solid,
            render_modes,
            texture_layers: registry.texture_layers.clone(),
            water_id,
            total_layers,
        }
    }
}

/// 单个区块的 Mesh 构建输入快照
struct MeshBuildInput {
    chunk_pos: IVec3,
    current_data: ChunkData,
    neighbors: [Option<ChunkData>; 6],
    block_info: BlockInfoSnapshot,
}

// 玩家区块缓存
#[derive(Resource, Default)]
pub struct PlayerChunkCache {
    pub last_chunk_pos: Option<IVec3>,
    pub expected_chunks: HashSet<IVec3>,
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


/// 区块加载与卸载调度
pub fn manage_chunks_system(
    mut commands: Commands,
    mut save_queue: ResMut<SaveQueue>,
    mut world_storage: ResMut<WorldStorage>,
    mut player_cache: ResMut<PlayerChunkCache>,
    chunk_query: Query<(Entity, &ChunkComponents)>,
    player_query: Query<&Transform, With<Player>>,
    save_config: Res<SaveConfig>,
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

    //只有当玩家跨越区块边界时才重建 expected_chunks
    let needs_rebuild = player_cache.last_chunk_pos != Some(player_chunk_pos);

    if needs_rebuild {
        player_cache.last_chunk_pos = Some(player_chunk_pos);

        let render_distance = 8;
        let data_distance = render_distance + 1;

        let mut expected_chunks = HashSet::new();
        for x in -data_distance..=data_distance {
            for y in -data_distance..=data_distance {
                for z in -data_distance..=data_distance {
                    expected_chunks.insert(player_chunk_pos + IVec3::new(x, y, z));
                }
            }
        }
        player_cache.expected_chunks = expected_chunks;
    }

    // 加载范围内的区块
    let mut spawned = 0u32;
    let max_spawn_per_frame = 16;

    // 加载范围内的区块
    for &chunk_pos in &player_cache.expected_chunks {
        // 检测坐标是否存在数据
        if !world_storage.chunk_entities.contains_key(&chunk_pos) {
            // 若每帧生成的区块大于预定值则到下一帧再生成
            if spawned >= max_spawn_per_frame { break; }
            spawned += 1;

            // 生成区块，标记初始状态
            let entity = commands.spawn((
                ChunkComponents { position: chunk_pos },
                ChunkState::Empty,
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

    // 卸载超出范围的区块
    for (entity, chunk_components) in chunk_query.iter() {
        let pos = chunk_components.position;

        if !player_cache.expected_chunks.contains(&pos) {
            // 保存区块数据到存档（在移除数据之前）
            if save_config.save_on_unload {
                if let Some(chunk_data) = world_storage.loaded_chunks.get(&pos) {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs_f64();
                    save_queue.queue.push_back(SavedChunk {
                        position: pos,
                        data: chunk_data.clone(),
                        modified_time: world_storage
                            .chunk_modified_times
                            .get(&pos)
                            .copied()
                            .unwrap_or(now),
                    });
                }
            }

            // 移除实体映射和方块数据
            world_storage.chunk_entities.remove(&pos);
            world_storage.loaded_chunks.remove(&pos);
            world_storage.chunk_modified_times.remove(&pos);

            // 销毁实体（用 queue_silenced 静默处理已销毁实体，避免竞态报错）
            commands.entity(entity)
                .queue_silenced(|mut entity: EntityWorldMut| {
                    entity.despawn();
                });
        }
    }
}

/// 生成区块内方块数据
pub fn spawn_terrain_gen_tasks(
    block_registry: Res<BlockRegistry>,
    channel: Res<TerrainGenChannel>,
    world_generator: Res<WorldGenerator>,
    cached_ids: Res<CachedBlockIds>,
    save_config: Res<SaveConfig>,
    mut world_storage: ResMut<WorldStorage>,
    mut chunk_query: Query<(Entity, &ChunkComponents, &mut ChunkState)>,
){
    // 每帧对多渲染的区块数
    let mut spawned = 0u32;
    let max_per_frame = 16;

    for (entity, chunk_components,mut chunk_state) in &mut chunk_query {
        // 状态检查
        if *chunk_state != ChunkState::Empty { continue; }
        if spawned >= max_per_frame { break; }


        // 获取该区块在世界网格中的坐标
        let chunk_pos = chunk_components.position;

        // 若已存在数据则跳过生成
        if world_storage.loaded_chunks.contains_key(&chunk_pos) {
            *chunk_state = ChunkState::TerrainReady;
            spawned += 1;
            continue;
        }

        // 优先从存档加载区块数据
        if let Some(mut saved) = save::system::try_load_chunk_from_disk(
            &save_config.world_name,
            chunk_pos,
        )  {
            // 从level.dat加载ID映射表进行重映射
            if let Ok(level_data) = save::level::load_level(&save_config.world_name) {
                save::level::remap_chunk_block_ids(
                    &mut saved.data,
                    &level_data.block_id_map,
                    &block_registry,
                );
            }

            world_storage.loaded_chunks.insert(chunk_pos, saved.data);
            *chunk_state = ChunkState::TerrainReady;
            spawned += 1;
            continue;
        }

        // 将地形生成派发到异步线程池
        let sender = channel.sender.clone();
        let block_ids = cached_ids.0;
        let seed = world_generator.seed;
        let climate_config = world_generator.pipeline.climate_sampler.config.clone();
        let current_season = world_generator.pipeline.climate_sampler.current_season;
        let biome_registry = world_generator.pipeline.biome_registry.clone();

        AsyncComputeTaskPool::get().spawn(async move {
            // 在异步线程中重建 pipeline
            let pipeline = crate::world::generation::pipeline::GenerationPipeline::rebuild_from_seed(
                seed, climate_config, current_season, biome_registry,
            );
            let chunk_data = pipeline.generate_chunk(chunk_pos, block_ids);
            let _ = sender.send(TerrainGenResult { chunk_pos, chunk_data });
        }).detach();

        *chunk_state = ChunkState::GeneratingTerrain;
        spawned += 1;
    }
}
// 地形生成接收结果
pub fn receive_terrain_results(
    mut world_storage: ResMut<WorldStorage>,
    channel: Res<TerrainGenChannel>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
) {
    // 批量接收所有已完成的结果
    while let Ok(result) = channel.receiver.lock().unwrap().try_recv() {
        let chunk_pos = result.chunk_pos;
        let mut chunk_data = result.chunk_data;

        // 应用延迟写入
        apply_pending_writes(chunk_pos, &mut chunk_data, &mut world_storage);

        world_storage.loaded_chunks.insert(chunk_pos, chunk_data);

        // 更新对应区块实体的状态
        for (chunk_components, mut chunk_state) in &mut chunk_query {
            if chunk_components.position == chunk_pos && *chunk_state == ChunkState::GeneratingTerrain {
                *chunk_state = ChunkState::TerrainReady;
            }
        }
    }
}

// 结构生成
pub fn generate_structures_system(
    mut world_storage: ResMut<WorldStorage>,
    mut chunk_query: Query<(&ChunkComponents, &mut ChunkState)>,
    block_registry: Res<BlockRegistry>,
    seed: Res<WorldGenerator>,
    cached_ids: Res<CachedBlockIds>,
) {
    // 基于时间预算
    let budget = std::time::Instant::now();
    let max_budget_ms = 4.0;

    for (chunk_components, mut state) in &mut chunk_query {
        if *state != ChunkState::TerrainReady && *state != ChunkState::GeneratingStructure {
            continue;
        }

        if budget.elapsed().as_secs_f64() * 1000.0 > max_budget_ms {
            break;
        }

        *state = ChunkState::GeneratingStructure;


        let chunk_pos = chunk_components.position;
        let ctx = crate::world::generation::terrain::TerrainGenerator::sample_context(
            &seed.pipeline.noise_sampler,
            &seed.pipeline.climate_sampler,
            &seed.pipeline.biome_registry,
            chunk_pos,
        );

        StructureGenerator::generate_structures_world_aware(
            chunk_pos,
            &ctx,
            &cached_ids.0,
            &seed.pipeline.biome_registry,
            seed.seed,
            &mut world_storage,
        );

        *state = ChunkState::StructureReady;
    }
}

/// 构建网格系统
pub fn spawn_mesh_build_tasks(
    channel: Res<MeshBuildChannel>,
    registry: Option<Res<BlockRegistry>>,
    world_storage: Res<WorldStorage>,
    mut chunk_query: Query<(Entity, &ChunkComponents, &mut ChunkState)>,
) {
    let Some(reg) = registry else { return; };

    // 扫描有无待处理区块
    let ready_count = chunk_query.iter()
        .filter(|(_, c, s)| **s == ChunkState::StructureReady
            && world_storage.chunk_entities.contains_key(&c.position))
        .count();
    if ready_count == 0 { return; }

    let block_info = BlockInfoSnapshot::from_registry(&reg);

    let mut spawned = 0u32;
    let max_per_frame = 8;

    for (chunk_entity, chunk_components, mut state) in &mut chunk_query {
        if !world_storage.chunk_entities.contains_key(&chunk_components.position) {
            continue;
        }
        if *state != ChunkState::StructureReady { continue; }
        if spawned >= max_per_frame { break; }

        let current_chunk_pos = chunk_components.position;

        // 检查邻居是否就绪
        let neighbors_ready = DIRECTIONS.iter()
            .all(|(dir, _)| world_storage.loaded_chunks.contains_key(&(current_chunk_pos + *dir)));
        if !neighbors_ready { continue; }

        let Some(current_chunk_data) = world_storage.loaded_chunks.get(&current_chunk_pos) else {
            continue;
        };

        // 快照当前区块和邻居数据（异步任务不能持有 WorldStorage 引用）
        let current_data = current_chunk_data.clone();
        let neighbors: [Option<ChunkData>; 6] = DIRECTIONS.map(|(dir, _)| {
            world_storage.loaded_chunks.get(&(current_chunk_pos + dir)).cloned()
        });

        let sender = channel.sender.clone();
        let input = MeshBuildInput {
            chunk_pos: current_chunk_pos,
            current_data,
            neighbors,
            block_info: block_info.clone(),
        };

        AsyncComputeTaskPool::get().spawn(async move {
            let result = build_mesh_offthread(input);
            let _ = sender.send(result);
        }).detach();

        *state = ChunkState::GeneratingMesh;
        spawned += 1;
    }
}

// 构建Mesh顶点数据
fn build_mesh_offthread(input: MeshBuildInput) -> MeshBuildResult {
    let MeshBuildInput { chunk_pos, current_data, neighbors, block_info } = input;

    let mut opaque_buf = MeshBufferData::new();
    let mut cutout_buf = MeshBufferData::new();
    let mut water_buf = MeshBufferData::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let voxel_id = current_data.get_voxel(x, y, z);
                if voxel_id == 0 { continue; }

                let current_is_water = voxel_id == block_info.water_id;
                let local_pos = IVec3::new(x as i32, y as i32, z as i32);

                // 从快照获取渲染模式
                let render_mode = block_info.render_modes
                    .get(voxel_id as usize)
                    .copied()
                    .unwrap_or(RenderMode::Opaque);

                for face_idx in 0..6 {
                    let (dir, normal) = DIRECTIONS[face_idx];
                    let neighbor_local_pos = local_pos + dir;

                    let is_visible = match get_neighbor_voxel_id_snapshot(
                        neighbor_local_pos,
                        &current_data,
                        chunk_pos,
                        &neighbors,
                        dir,
                    ) {
                        Some(nbr_id) => is_face_visible_snapshot(current_is_water, nbr_id, &block_info),
                        None => !current_is_water,
                    };

                    if !is_visible { continue; }

                    let face_vertices = get_face_vertices(x as f32, y as f32, z as f32, face_idx);
                    let layer_id = block_info.texture_layers.get(&(voxel_id, face_idx)).copied().unwrap_or(0);
                    let uvs = compute_face_uvs(layer_id, block_info.total_layers);

                    if current_is_water {
                        water_buf.append_face(&face_vertices, normal, &uvs);
                    } else if render_mode == RenderMode::Cutout {
                        cutout_buf.append_face(&face_vertices, normal, &uvs);
                    } else {
                        opaque_buf.append_face(&face_vertices, normal, &uvs);
                    }
                }
            }
        }
    }

    MeshBuildResult {
        chunk_pos,
        opaque: opaque_buf,
        cutout: cutout_buf,
        water: water_buf,
    }
}

/// 邻居查询
fn get_neighbor_voxel_id_snapshot(
    neighbor_local_pos: IVec3,
    current_chunk_data: &ChunkData,
    current_chunk_pos: IVec3,
    neighbors: &[Option<ChunkData>; 6],
    dir: IVec3,
) -> Option<u16> {
    if let Some(nbr_id) = current_chunk_data.get_voxel_safe(
        neighbor_local_pos.x,
        neighbor_local_pos.y,
        neighbor_local_pos.z,
    ) {
        return Some(nbr_id);
    }
    // 跨区块查找
    let face_idx = DIRECTIONS.iter().position(|(d, _)| *d == dir)?;
    let neighbor_chunk_data = neighbors[face_idx].as_ref()?;
    let nbr_local_x = neighbor_local_pos.x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let nbr_local_y = neighbor_local_pos.y.rem_euclid(CHUNK_SIZE as i32) as usize;
    let nbr_local_z = neighbor_local_pos.z.rem_euclid(CHUNK_SIZE as i32) as usize;
    Some(neighbor_chunk_data.get_voxel(nbr_local_x, nbr_local_y, nbr_local_z))
}

/// 判断某个面是否需要渲染（邻居是否透明）
fn is_face_visible_snapshot(
    current_is_water: bool,
    neighbor_voxel_id: u16,
    block_info: &BlockInfoSnapshot,
) -> bool {
    if neighbor_voxel_id == 0 {
        return true;
    }
    if current_is_water {
        return false;
    }
    let nbr_is_solid = block_info.is_solid
        .get(neighbor_voxel_id as usize)
        .copied()
        .unwrap_or(true);
    !nbr_is_solid || neighbor_voxel_id == block_info.water_id
}

/// 接收Mesh构建
pub fn receive_mesh_results(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    channel: Res<MeshBuildChannel>,
    registry: Option<Res<BlockRegistry>>,
    mut chunk_query: Query<(Entity, &ChunkComponents, &mut ChunkState)>,
) {
    let Some(reg) = registry else { return; };
    let opaque_mat = reg.opaque_material.clone();
    let cutout_mat = reg.cutout_material.clone();
    let transparent_mat = reg.transparent_material.clone();

    while let Ok(result) = channel.receiver.lock().unwrap().try_recv() {
        let Some((chunk_entity, _, mut state)) = chunk_query
            .iter_mut()
            .find(|(_, c, _)| c.position == result.chunk_pos)
        else {
            continue;
        };

        if *state != ChunkState::GeneratingMesh { continue; }

        commands.entity(chunk_entity)
            .queue_silenced(|mut entity: EntityWorldMut| {
                entity.remove::<Mesh3d>()
                    .remove::<MeshMaterial3d<StandardMaterial>>();
            });
        commands.entity(chunk_entity)
            .queue_silenced(|mut entity: EntityWorldMut| {
                entity.despawn_related::<Children>();
            });

        if !result.opaque.is_empty() {
            let opaque_mesh = meshes.add(result.opaque.build_mesh());
            let mat = opaque_mat.clone();
            commands.entity(chunk_entity)
                .queue_silenced(move |mut entity: EntityWorldMut| {
                    entity.insert((
                        Mesh3d(opaque_mesh),
                        MeshMaterial3d(mat),
                    ));
                });
        }

        if !result.cutout.is_empty() {
            let cutout_mesh = meshes.add(result.cutout.build_mesh());
            let mat = cutout_mat.clone();
            let child = commands.spawn((
                Mesh3d(cutout_mesh),
                MeshMaterial3d(mat),
                Transform::IDENTITY,
                Visibility::default(),
            )).id();
            commands.entity(chunk_entity)
                .queue_silenced(move |mut entity: EntityWorldMut| {
                    entity.add_child(child);
                });
        }

        if !result.water.is_empty() {
            let water_mesh = meshes.add(result.water.build_mesh());
            let mat = transparent_mat.clone();
            let child = commands.spawn((
                Mesh3d(water_mesh),
                MeshMaterial3d(mat),
                Transform::IDENTITY,
                Visibility::default(),
            )).id();
            commands.entity(chunk_entity)
                .queue_silenced(move |mut entity: EntityWorldMut| {
                    entity.add_child(child);
                });
        }

        *state = ChunkState::Rendered;
    }
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

// 延迟写入
fn apply_pending_writes(
    chunk_pos: IVec3,
    chunk: &mut ChunkData,
    storage: &mut WorldStorage,
) {
    if let Some(writes) = storage.pending_writes.writes.remove(&chunk_pos) {
        for write in writes {
            if chunk.get_voxel(write.local_x, write.local_y, write.local_z) == 0 {
                chunk.set_voxel(write.local_x, write.local_y, write.local_z, write.block_id);
            }
        }
    }
}