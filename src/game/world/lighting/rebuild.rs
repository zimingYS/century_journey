//! 重建天空光与 RGB 方块光场（纯算法，可白盒测试）。
//!
//! 光场只依赖派发时的已加载体素快照和内容定义。区块拓扑、方块或内容发生变化时，
//! 任务线程整体重建已加载窗口，优先保证跨区块传播、移除光源和多路径滤色的确定性；
//! 固定步主线程只校验并提交仍然匹配权威世界的结果。

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::Arc;

use bevy::math::IVec3;

use crate::content::block::definition::BlockLightDef;
use crate::content::block::registry::BlockRegistry;
use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::pipeline::GenerationPipeline;
use crate::game::world::lighting::chunk_light::{ChunkLight, LightCell, LightRgb};
use crate::game::world::state::WorldState;
use crate::shared::voxel::CHUNK_SIZE;

/// 后台光照任务持有的不可变权威区块快照。
///
/// 快照只克隆 `Arc`，派发成本与区块数线性相关但不复制体素数组；任务结束前，
/// 世界侧的写入会通过 `Arc::make_mut` 产生新快照，便于主线程拒绝过期结果。
#[derive(Clone)]
pub struct LightingWorldSnapshot {
    chunks: HashMap<IVec3, Arc<ChunkData>>,
    terrain_pipeline: Option<GenerationPipeline>,
}

impl LightingWorldSnapshot {
    /// 从当前权威世界取得后台任务可独占读取的轻量快照。
    pub fn from_world(world: &WorldState) -> Self {
        let chunks = world
            .chunks()
            .map(|(position, data)| (position, Arc::clone(data)))
            .collect::<HashMap<_, _>>();
        Self {
            chunks,
            terrain_pipeline: None,
        }
    }

    /// 从当前世界和生成管线取得带自然地表判定的完整快照。
    pub fn from_world_with_terrain(
        world: &WorldState,
        terrain_pipeline: &GenerationPipeline,
    ) -> Self {
        let mut snapshot = Self::from_world(world);
        snapshot.terrain_pipeline = Some(terrain_pipeline.clone());
        snapshot
    }

    /// 从指定水平区块列取得局部传播快照；每列包含当前已加载的全部高度层。
    pub fn from_columns(world: &WorldState, columns: &HashSet<(i32, i32)>) -> Self {
        let chunks = world
            .chunks()
            .filter(|(position, _)| columns.contains(&(position.x, position.z)))
            .map(|(position, data)| (position, Arc::clone(data)))
            .collect::<HashMap<_, _>>();
        Self {
            chunks,
            terrain_pipeline: None,
        }
    }

    /// 从指定列和生成管线取得带自然地表判定的局部快照。
    pub fn from_columns_with_terrain(
        world: &WorldState,
        columns: &HashSet<(i32, i32)>,
        terrain_pipeline: &GenerationPipeline,
    ) -> Self {
        let mut snapshot = Self::from_columns(world, columns);
        snapshot.terrain_pipeline = Some(terrain_pipeline.clone());
        snapshot
    }

    /// 交还任务持有的区块快照，供已提交光场判断数据是否仍然匹配。
    pub fn into_chunks(self) -> HashMap<IVec3, Arc<ChunkData>> {
        self.chunks
    }

    /// 判断目标区块给定水平依赖圈内的列是否仍与派发任务时完全一致。
    ///
    /// 同时检查快照已有区块和任务期间新加入的区块，避免新增上层区块改变天空光后
    /// 仍提交旧结果。高度方向不裁剪，以覆盖整条直射天空光柱。
    pub fn neighborhood_is_current(&self, world: &WorldState, target: IVec3, halo: i32) -> bool {
        let in_neighborhood = |position: IVec3| {
            (position.x - target.x).abs() <= halo && (position.z - target.z).abs() <= halo
        };
        self.chunks
            .iter()
            .filter(|(position, _)| in_neighborhood(**position))
            .all(|(position, snapshot)| {
                world
                    .chunk(*position)
                    .is_some_and(|current| Arc::ptr_eq(snapshot, current))
            })
            && world
                .chunks()
                .filter(|(position, _)| in_neighborhood(*position))
                .all(|(position, current)| {
                    self.chunks
                        .get(&position)
                        .is_some_and(|snapshot| Arc::ptr_eq(snapshot, current))
                })
    }

    #[inline]
    fn chunk(&self, position: IVec3) -> Option<&Arc<ChunkData>> {
        self.chunks.get(&position)
    }

    /// 判断指定区块是否在快照中。供远景 LOD 判定"该位置真实区块是否加载"，
    /// 玩家跨区块或垂直飞行导致已加载区块卸载时由远景兜底，避免真空带。
    #[inline]
    pub fn contains_chunk(&self, position: IVec3) -> bool {
        self.chunks.contains_key(&position)
    }

    /// 遍历任务快照中的区块，供局部调度复用已有天空光并构造传播目标。
    pub(super) fn chunks(&self) -> impl Iterator<Item = (IVec3, &Arc<ChunkData>)> {
        self.chunks.iter().map(|(position, data)| (*position, data))
    }
}

/// 六方向单位偏移；顺序固定以保证传播测试和调试结果稳定。
const DIRECTIONS: [IVec3; 6] = [
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(1, 0, 0),
    IVec3::new(0, 0, -1),
    IVec3::new(0, 0, 1),
];

/// 单个方块的光传播属性。
#[derive(Debug, Clone)]
pub struct VoxelLightProp {
    /// RGB 透射系数（全零 = 阻断，全一 = 完全透过）。
    pub filter: [f32; 3],
    /// 数据驱动发光定义；`None` 表示不发光。
    pub light: Option<BlockLightDef>,
}

impl Default for VoxelLightProp {
    fn default() -> Self {
        Self {
            filter: [0.0; 3],
            light: None,
        }
    }
}

const DEFAULT_PROP: VoxelLightProp = VoxelLightProp {
    filter: [0.0; 3],
    light: None,
};

/// 传播使用的内容属性快照，避免固定步算法反复查询哈希注册表。
#[derive(Debug, Clone, Default)]
pub struct GameLightInfo {
    props: Vec<VoxelLightProp>,
    max_block_range: u8,
}

impl GameLightInfo {
    /// 从内容注册表复制并归一化传播所需属性。
    pub fn from_registry(registry: &BlockRegistry) -> Self {
        let max_id = registry
            .iter_properties()
            .map(|(&id, _)| id)
            .max()
            .unwrap_or(0);
        let mut props = vec![VoxelLightProp::default(); (max_id + 1) as usize];
        let mut max_block_range = 0;
        for (&id, property) in registry.iter_properties() {
            let light = property.light.or_else(|| {
                (property.light_emission > 0)
                    .then(|| BlockLightDef::from_legacy_emission(property.light_emission))
            });
            let filter = property
                .light_filter
                .unwrap_or([property.light_transmission; 3])
                .map(|channel| channel.clamp(0.0, 1.0));
            if let Some(light) = light.filter(|light| light.emission > 0) {
                max_block_range = max_block_range.max(light.range);
            }
            props[id as usize] = VoxelLightProp { filter, light };
        }
        Self {
            props,
            max_block_range,
        }
    }

    /// 返回方块传播属性；未知运行时 ID 按完全阻光处理。
    #[inline]
    pub fn prop(&self, id: u16) -> &VoxelLightProp {
        self.props.get(id as usize).unwrap_or(&DEFAULT_PROP)
    }

    /// 返回局部重建覆盖全部方块光影响所需的水平区块圈数。
    pub fn block_light_chunk_halo(&self) -> i32 {
        usize::from(self.max_block_range)
            .div_ceil(CHUNK_SIZE)
            .max(1) as i32
    }
}

/// 已加载世界中一个可供客户端表现的方块光源。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockLightSource {
    /// 光源方块的整数世界坐标。
    pub world_pos: IVec3,
    /// 经过内容兼容层解析后的完整光源定义。
    pub light: BlockLightDef,
}

/// 读取世界坐标处的方块编号；区块未加载视为空气，但传播写入仍受光数组边界限制。
#[inline]
pub fn voxel_at(world: &LightingWorldSnapshot, pos: IVec3) -> u16 {
    let (chunk_pos, local) = split_world(pos);
    world
        .chunk(chunk_pos)
        .map(|chunk| chunk.get_voxel(local.x as usize, local.y as usize, local.z as usize))
        .unwrap_or(0)
}

/// 重建当前已加载窗口的天空光、方块光与光源索引。
///
/// 返回值按世界坐标排序，客户端据此选择有限数量的 Bevy 实体光源；函数本身
/// 不创建表现实体，也不依赖相机或渲染帧。`sky_dirty_columns` 列出需要重灌
/// 直射天光的水平列；其余列保留调用者预先放入 `lights` 的天空光，只清除并
/// 重建方块光，适用于已确认没有改变天空通路的区域。
pub fn rebuild_loaded_lighting(
    world: &LightingWorldSnapshot,
    info: &GameLightInfo,
    lights: &mut HashMap<IVec3, ChunkLight>,
    sky_dirty_columns: &HashSet<(i32, i32)>,
) -> Vec<BlockLightSource> {
    rebuild_loaded_lighting_impl(world, info, lights, sky_dirty_columns, None, None)
}

/// 复用持久光源索引并为尚未建立索引的区块执行全量扫描的混合重建。
///
/// 已就绪区块的发光方块已由权威世界增量维护进 `indexed_sources`（方块事件与提交路径
/// 共同更新），流送和交互任务都无需重复遍历这些区块；只有 `scan_positions` 列出的
/// 新加载区块需要全量扫描。索引项仍会依据任务快照重新解析，过期条目不会传播。
pub(super) fn rebuild_loaded_lighting_from_source_index(
    world: &LightingWorldSnapshot,
    info: &GameLightInfo,
    lights: &mut HashMap<IVec3, ChunkLight>,
    sky_dirty_columns: &HashSet<(i32, i32)>,
    indexed_sources: &[BlockLightSource],
    scan_positions: &[IVec3],
) -> Vec<BlockLightSource> {
    rebuild_loaded_lighting_impl(
        world,
        info,
        lights,
        sky_dirty_columns,
        Some(indexed_sources),
        Some(scan_positions),
    )
}

fn rebuild_loaded_lighting_impl(
    world: &LightingWorldSnapshot,
    info: &GameLightInfo,
    lights: &mut HashMap<IVec3, ChunkLight>,
    sky_dirty_columns: &HashSet<(i32, i32)>,
    indexed_sources: Option<&[BlockLightSource]>,
    scan_positions: Option<&[IVec3]>,
) -> Vec<BlockLightSource> {
    let mut chunk_positions = world
        .chunks()
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    chunk_positions.sort_by_key(|position| (position.x, position.z, position.y));
    let loaded = chunk_positions.iter().copied().collect::<HashSet<_>>();
    lights.retain(|position, _| loaded.contains(position));
    for position in &chunk_positions {
        let light = lights.entry(*position).or_default();
        if sky_dirty_columns.contains(&(position.x, position.z)) {
            light.reset();
        } else {
            light.reset_block();
        }
    }

    if !sky_dirty_columns.is_empty() {
        initialize_vertical_sky(world, info, lights, &chunk_positions, sky_dirty_columns);
        spread_sky_light(world, info, lights, &chunk_positions, sky_dirty_columns);
    }

    let sources = match (indexed_sources, scan_positions) {
        (None, _) => collect_sources(world, info, &chunk_positions),
        (Some(indexed), None) => collect_indexed_sources(world, info, indexed),
        (Some(indexed), Some(scan)) => {
            let mut sources = collect_indexed_sources(world, info, indexed);
            sources.extend(collect_sources(world, info, scan));
            sources
                .sort_by_key(|source| (source.world_pos.x, source.world_pos.y, source.world_pos.z));
            sources.dedup_by_key(|source| source.world_pos);
            sources
        }
    };
    for source in &sources {
        propagate_block_source(world, info, lights, *source);
    }

    for light in lights.values_mut() {
        light.mark_initialized();
    }
    sources
}

/// 根据任务快照校验持久索引，并用当前内容定义刷新光源参数。
fn collect_indexed_sources(
    world: &LightingWorldSnapshot,
    info: &GameLightInfo,
    indexed_sources: &[BlockLightSource],
) -> Vec<BlockLightSource> {
    let mut sources = indexed_sources
        .iter()
        .filter_map(|source| {
            info.prop(voxel_at(world, source.world_pos))
                .light
                .filter(|light| light.emission > 0)
                .map(|light| BlockLightSource {
                    world_pos: source.world_pos,
                    light,
                })
        })
        .collect::<Vec<_>>();
    sources.sort_by_key(|source| (source.world_pos.x, source.world_pos.y, source.world_pos.z));
    sources.dedup_by_key(|source| source.world_pos);
    sources
}

/// 按水平区块列从高到低灌入不随垂直距离衰减的直射天空光。
fn initialize_vertical_sky(
    world: &LightingWorldSnapshot,
    info: &GameLightInfo,
    lights: &mut HashMap<IVec3, ChunkLight>,
    chunk_positions: &[IVec3],
    sky_dirty_columns: &HashSet<(i32, i32)>,
) {
    let mut columns = BTreeMap::<(i32, i32), Vec<i32>>::new();
    for position in chunk_positions {
        if !sky_dirty_columns.contains(&(position.x, position.z)) {
            continue;
        }
        columns
            .entry((position.x, position.z))
            .or_default()
            .push(position.y);
    }
    for ((chunk_x, chunk_z), mut ys) in columns {
        ys.sort_unstable_by(|left, right| right.cmp(left));
        let top_world_y = ys[0] * CHUNK_SIZE as i32 + CHUNK_SIZE as i32 - 1;
        let terrain = world
            .terrain_pipeline
            .as_ref()
            .map(|pipeline| pipeline.sample_context(IVec3::new(chunk_x, 0, chunk_z)));
        for local_x in 0..CHUNK_SIZE {
            for local_z in 0..CHUNK_SIZE {
                let terrain_surface_y = terrain
                    .as_ref()
                    .map(|context| context.get_column(local_x, local_z).base_height);
                let mut sky = initial_vertical_sky(top_world_y, terrain_surface_y);
                for chunk_y in &ys {
                    let chunk_pos = IVec3::new(chunk_x, *chunk_y, chunk_z);
                    let Some(data) = world.chunk(chunk_pos) else {
                        continue;
                    };
                    let light_chunk = lights
                        .get_mut(&chunk_pos)
                        .expect("已加载区块必须保留光数组");
                    for local_y in (0..CHUNK_SIZE).rev() {
                        let id = data.get_voxel(local_x, local_y, local_z);
                        sky = sky.filtered(info.prop(id).filter);
                        let mut cell = light_chunk.get(local_x, local_y, local_z);
                        cell.sky = sky;
                        light_chunk.set(local_x, local_y, local_z, cell);
                    }
                }
            }
        }
    }
}

/// 只有已加载列顶达到自然地表时才允许天空光进入；未知高度保留测试和工具兼容语义。
fn initial_vertical_sky(top_world_y: i32, terrain_surface_y: Option<i32>) -> LightRgb {
    if terrain_surface_y.is_none_or(|surface_y| top_world_y >= surface_y) {
        LightRgb {
            r: 15,
            g: 15,
            b: 15,
        }
    } else {
        LightRgb::default()
    }
}

/// 从直射天空列向洞口和侧向空间衰减扩散，使洞穴入口具有连续明暗过渡。
fn spread_sky_light(
    world: &LightingWorldSnapshot,
    info: &GameLightInfo,
    lights: &mut HashMap<IVec3, ChunkLight>,
    chunk_positions: &[IVec3],
    sky_dirty_columns: &HashSet<(i32, i32)>,
) {
    // 种子列覆盖脏列本身及其水平 8 邻居：新加载列需要从相邻已就绪列
    // 接收天光扩散（洞口过渡），而脏列自身直射被挡时也必须能从邻居取光。
    let mut seed_columns = HashSet::with_capacity(sky_dirty_columns.len().saturating_mul(9));
    for &(chunk_x, chunk_z) in sky_dirty_columns {
        for x in chunk_x - 1..=chunk_x + 1 {
            for z in chunk_z - 1..=chunk_z + 1 {
                seed_columns.insert((x, z));
            }
        }
    }

    let mut queue = VecDeque::new();
    for chunk_pos in chunk_positions {
        if !seed_columns.contains(&(chunk_pos.x, chunk_pos.z)) {
            continue;
        }

        let base = *chunk_pos * CHUNK_SIZE as i32;
        let light = lights.get(chunk_pos).expect("已加载区块必须先创建光数组");
        for local_y in 0..CHUNK_SIZE {
            for local_z in 0..CHUNK_SIZE {
                for local_x in 0..CHUNK_SIZE {
                    let sky = light.get(local_x, local_y, local_z).sky;
                    let position =
                        base + IVec3::new(local_x as i32, local_y as i32, local_z as i32);
                    if !sky.is_dark() && can_spread_sky(world, info, lights, position, sky) {
                        queue.push_back((position, sky));
                    }
                }
            }
        }
    }

    while let Some((position, level)) = queue.pop_front() {
        for direction in DIRECTIONS {
            let next_position = position + direction;
            // 预分割一次，热路径内合并"体素读取 + 光数组定位 + 写入"，
            // 避免每个方向重复做区块坐标分割与哈希定位。
            let (chunk_pos, local) = split_world(next_position);
            let voxel_id = world.chunk(chunk_pos).map_or(0, |data| {
                data.get_voxel(local.x as usize, local.y as usize, local.z as usize)
            });
            let next_level = level.attenuated(info.prop(voxel_id).filter);
            if next_level.is_dark() {
                continue;
            }
            let Some(light_chunk) = lights.get_mut(&chunk_pos) else {
                continue;
            };
            let mut cell = light_chunk.get(local.x as usize, local.y as usize, local.z as usize);
            if cell.sky.max_assign(next_level) {
                light_chunk.set(local.x as usize, local.y as usize, local.z as usize, cell);
                queue.push_back((next_position, next_level));
            }
        }
    }
}

/// 只把确实能改善相邻格的直射天空光加入队列，避免为整片露天空气分配队列节点。
fn can_spread_sky(
    world: &LightingWorldSnapshot,
    info: &GameLightInfo,
    lights: &HashMap<IVec3, ChunkLight>,
    position: IVec3,
    level: LightRgb,
) -> bool {
    // 预分割当前格：绝大多数邻居位于同一区块，可直接按局部坐标读取；
    // 只有跨区块边界的邻居才需要重新分割与哈希定位。
    let (chunk_pos, local) = split_world(position);
    let Some(chunk) = lights.get(&chunk_pos) else {
        return false;
    };
    for direction in DIRECTIONS {
        let next_position = position + direction;
        let neighbor_local = local + direction;
        let in_chunk = (0..CHUNK_SIZE as i32).contains(&neighbor_local.x)
            && (0..CHUNK_SIZE as i32).contains(&neighbor_local.y)
            && (0..CHUNK_SIZE as i32).contains(&neighbor_local.z);
        let neighbor_sky = if in_chunk {
            chunk
                .get(
                    neighbor_local.x as usize,
                    neighbor_local.y as usize,
                    neighbor_local.z as usize,
                )
                .sky
        } else {
            let Some(cell) = light_cell_at(lights, next_position) else {
                continue;
            };
            cell.sky
        };
        // 快速负过滤：邻居天光已逐通道不低于当前级，衰减后只会更低、不可能被改善，
        // 直接跳过体素与滤色查询，避免为整片露天内部格做冗余衰减计算。
        if neighbor_sky.r >= level.r && neighbor_sky.g >= level.g && neighbor_sky.b >= level.b {
            continue;
        }
        let next_level = level.attenuated(info.prop(voxel_at(world, next_position)).filter);
        if next_level.r > neighbor_sky.r
            || next_level.g > neighbor_sky.g
            || next_level.b > neighbor_sky.b
        {
            return true;
        }
    }
    false
}

fn light_cell_at(lights: &HashMap<IVec3, ChunkLight>, position: IVec3) -> Option<LightCell> {
    let (chunk_pos, local) = split_world(position);
    lights
        .get(&chunk_pos)
        .map(|chunk| chunk.get(local.x as usize, local.y as usize, local.z as usize))
}

fn collect_sources(
    world: &LightingWorldSnapshot,
    info: &GameLightInfo,
    chunk_positions: &[IVec3],
) -> Vec<BlockLightSource> {
    let mut sources = Vec::new();
    for chunk_pos in chunk_positions {
        let Some(data) = world.chunk(*chunk_pos) else {
            continue;
        };
        let base = *chunk_pos * CHUNK_SIZE as i32;
        for local_y in 0..CHUNK_SIZE {
            for local_z in 0..CHUNK_SIZE {
                for local_x in 0..CHUNK_SIZE {
                    let id = data.get_voxel(local_x, local_y, local_z);
                    if let Some(light) = info.prop(id).light
                        && light.emission > 0
                    {
                        sources.push(BlockLightSource {
                            world_pos: base
                                + IVec3::new(local_x as i32, local_y as i32, local_z as i32),
                            light,
                        });
                    }
                }
            }
        }
    }
    sources.sort_by_key(|source| (source.world_pos.x, source.world_pos.y, source.world_pos.z));
    sources
}

/// 单独传播一个光源，局部访问表防止短程强光压住远程较弱光的继续扩散。
fn propagate_block_source(
    world: &LightingWorldSnapshot,
    info: &GameLightInfo,
    lights: &mut HashMap<IVec3, ChunkLight>,
    source: BlockLightSource,
) {
    let source_level = LightRgb::from_emission(source.light.emission, source.light.color);
    if source_level.is_dark() {
        return;
    }

    let mut queue = VecDeque::from([(source.world_pos, source_level, 0u8)]);
    max_assign_block(lights, source.world_pos, source_level);

    while let Some((position, level, distance)) = queue.pop_front() {
        if distance >= source.light.range {
            continue;
        }
        for direction in DIRECTIONS {
            let next_position = position + direction;
            // 预分割一次，热路径内合并"存在性检查 + 体素读取 + 光写入"，
            // 避免每个方向重复做区块坐标分割与哈希定位。
            let (chunk_pos, local) = split_world(next_position);
            let Some(light_chunk) = lights.get_mut(&chunk_pos) else {
                continue;
            };
            let voxel_id = world.chunk(chunk_pos).map_or(0, |data| {
                data.get_voxel(local.x as usize, local.y as usize, local.z as usize)
            });
            let filtered = level.filtered(info.prop(voxel_id).filter);
            let next_level =
                block_level_at_distance(filtered, source_level, distance + 1, source.light.range);
            if next_level.is_dark() {
                continue;
            }
            let mut cell = light_chunk.get(local.x as usize, local.y as usize, local.z as usize);
            if cell.block.max_assign(next_level) {
                light_chunk.set(local.x as usize, local.y as usize, local.z as usize, cell);
                queue.push_back((next_position, next_level, distance + 1));
            }
        }
    }
}

fn block_level_at_distance(
    filtered: LightRgb,
    source: LightRgb,
    distance: u8,
    range: u8,
) -> LightRgb {
    let denominator = u16::from(range.max(1)) + 1;
    let remaining = denominator.saturating_sub(u16::from(distance));
    let source_peak = source.r.max(source.g).max(source.b);
    let peak_cap = (u16::from(source_peak) * remaining).div_ceil(denominator) as u8;
    limit_light_peak(filtered, peak_cap)
}

/// 按当前滤色后的色相等比限制峰值，避免低光级逐通道向上取整变成白、青色边缘。
fn limit_light_peak(light: LightRgb, peak_cap: u8) -> LightRgb {
    let peak = light.r.max(light.g).max(light.b);
    if peak == 0 || peak <= peak_cap {
        return light;
    }
    let scale = |value: u8| u16::from(value) * u16::from(peak_cap) / u16::from(peak);
    LightRgb {
        r: scale(light.r) as u8,
        g: scale(light.g) as u8,
        b: scale(light.b) as u8,
    }
}

#[inline]
fn max_assign_block(
    lights: &mut HashMap<IVec3, ChunkLight>,
    position: IVec3,
    value: LightRgb,
) -> bool {
    update_cell(lights, position, |cell| cell.block.max_assign(value))
}

fn update_cell(
    lights: &mut HashMap<IVec3, ChunkLight>,
    position: IVec3,
    update: impl FnOnce(&mut LightCell) -> bool,
) -> bool {
    let (chunk_pos, local) = split_world(position);
    let Some(chunk) = lights.get_mut(&chunk_pos) else {
        return false;
    };
    let mut cell = chunk.get(local.x as usize, local.y as usize, local.z as usize);
    if !update(&mut cell) {
        return false;
    }
    chunk.set(local.x as usize, local.y as usize, local.z as usize, cell);
    true
}

/// 把世界坐标拆成区块坐标与区块内局部坐标。
#[inline]
fn split_world(pos: IVec3) -> (IVec3, IVec3) {
    (
        IVec3::new(
            pos.x.div_euclid(CHUNK_SIZE as i32),
            pos.y.div_euclid(CHUNK_SIZE as i32),
            pos.z.div_euclid(CHUNK_SIZE as i32),
        ),
        IVec3::new(
            pos.x.rem_euclid(CHUNK_SIZE as i32),
            pos.y.rem_euclid(CHUNK_SIZE as i32),
            pos.z.rem_euclid(CHUNK_SIZE as i32),
        ),
    )
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/lighting/rebuild.rs"]
mod tests;
