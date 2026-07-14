use crate::content::block::registry::BlockRegistry;
use crate::content::constant::world::*;
use crate::engine::task::{TaskManager, TaskResult};
use crate::game::player::components::Player;
use crate::game::world::generation::WorldGenerator;
use crate::game::world::save::format::{LevelData, SavedChunk};
use crate::game::world::save::level;
use crate::game::world::save::region::RegionManager;
use crate::game::world::storage::WorldStorage;
use crate::game::world::time::TimeOfDay;
use bevy::prelude::*;
use bincode::Options;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, mpsc};

/// 缓存的 block_id 重映射表
#[derive(Resource, Clone, Default)]
pub struct CachedBlockIdRemap(pub HashMap<u16, u16>);

/// 保存配置
#[derive(Resource, Clone, Debug)]
pub struct SaveConfig {
    /// 存档名称
    pub world_name: String,
    /// 是否在区块卸载时自动保存
    pub save_on_unload: bool,
    /// 自动全量保存间隔（秒），0 = 禁用
    pub auto_save_interval: f64,
}

impl Default for SaveConfig {
    fn default() -> Self {
        Self {
            world_name: DEFAULT_WORLD_NAME.to_string(),
            save_on_unload: true,
            auto_save_interval: AUTO_SAVE_INTERVAL_SECS,
        }
    }
}

/// 保存队列
#[derive(Resource, Default, Debug)]
pub struct SaveQueue {
    pub queue: VecDeque<SavedChunk>,
}

impl SaveQueue {
    /// 同一区块只保留最新快照，避免玩家在边界往返时重复排队。
    pub fn enqueue(&mut self, chunk: SavedChunk) {
        if let Some(existing) = self
            .queue
            .iter_mut()
            .find(|queued| queued.position == chunk.position)
        {
            if chunk.modified_time >= existing.modified_time {
                *existing = chunk;
            }
        } else {
            self.queue.push_back(chunk);
        }
    }
}

struct SaveBatchCompletion {
    chunks: Vec<SavedChunk>,
    error: Option<String>,
}

/// 流式区块保存后台任务状态。只允许一个批次写盘，避免同一 Region 并发覆盖。
#[derive(Resource)]
pub struct SaveWorker {
    sender: mpsc::Sender<SaveBatchCompletion>,
    receiver: Mutex<mpsc::Receiver<SaveBatchCompletion>>,
    in_flight_positions: HashSet<IVec3>,
    in_flight_batches: usize,
}

impl Default for SaveWorker {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
            in_flight_positions: HashSet::new(),
            in_flight_batches: 0,
        }
    }
}

impl SaveWorker {
    pub fn is_idle(&self) -> bool {
        self.in_flight_batches == 0
    }
}

/// 加载队列
#[derive(Resource, Default, Debug)]
pub struct LoadQueue {
    pub queue: VecDeque<SavedChunk>,
}

/// 自动保存计时器
#[derive(Resource, Default, Debug)]
pub struct AutoSaveTimer {
    pub timer: Option<Timer>,
    pub elapsed: f64,
}

/// 区块卸载时自动保存系统
pub fn auto_save_on_unload_system(
    time: Res<Time>,
    mut auto_save_timer: ResMut<AutoSaveTimer>,
    save_config: Res<SaveConfig>,
    mut world_storage: ResMut<WorldStorage>,
    mut save_queue: ResMut<SaveQueue>,
    block_registry: Res<BlockRegistry>,
    world_generator: Res<WorldGenerator>,
    time_of_day: Res<TimeOfDay>,
    player_query: Query<&Transform, With<Player>>,
) {
    // 禁用自动保存则跳过
    if save_config.auto_save_interval <= 0.0 {
        return;
    }

    // 初始化计时器
    if auto_save_timer.timer.is_none() {
        auto_save_timer.timer = Some(Timer::from_seconds(
            save_config.auto_save_interval as f32,
            TimerMode::Repeating,
        ));
    }

    // 推进计时器
    let Some(ref mut timer) = auto_save_timer.timer else {
        return;
    };
    timer.tick(time.delta());

    if !timer.just_finished() {
        return;
    }

    // 获取玩家位置作为出生点
    let spawn_pos = player_query
        .single()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);

    // 元数据很小，直接原子保存；真正修改过的区块交给后台保存队列。
    if let Err(error) = level::save_level(
        &save_config.world_name,
        world_generator.seed as u64,
        spawn_pos,
        time_of_day.current_time,
        &block_registry,
    ) {
        log::error!("[自动保存] 世界元数据保存失败: {error}");
        return;
    }

    let modified: Vec<_> = world_storage.chunk_modified_times.keys().copied().collect();
    for position in modified {
        let Some(data) = world_storage
            .loaded_chunks
            .get(&position)
            .map(|data| data.as_ref().clone())
        else {
            world_storage.chunk_modified_times.remove(&position);
            continue;
        };
        let modified_time = world_storage
            .chunk_modified_times
            .remove(&position)
            .unwrap_or_default();
        save_queue.enqueue(SavedChunk {
            position,
            data,
            modified_time,
        });
    }
    log::trace!(
        "[自动保存] 元数据已保存，{} 个修改区块已进入后台队列",
        save_queue.queue.len()
    );
}

/// 每帧处理保存队列，批量写入磁盘
pub fn process_save_queue_system(
    mut save_queue: ResMut<SaveQueue>,
    save_config: Res<SaveConfig>,
    task: Res<TaskManager>,
    mut worker: ResMut<SaveWorker>,
) {
    collect_save_completions(&mut save_queue, &mut worker);
    if worker.in_flight_batches > 0 {
        return;
    }

    let mut batch = Vec::with_capacity(MAX_SAVE_PER_FRAME);
    let queued_count = save_queue.queue.len();
    for _ in 0..queued_count {
        let Some(chunk) = save_queue.queue.pop_front() else {
            break;
        };
        if worker.in_flight_positions.contains(&chunk.position) {
            save_queue.queue.push_back(chunk);
            continue;
        }
        batch.push(chunk);
        if batch.len() >= MAX_SAVE_PER_FRAME {
            break;
        }
    }
    if batch.is_empty() {
        return;
    }

    for chunk in &batch {
        worker.in_flight_positions.insert(chunk.position);
    }
    worker.in_flight_batches += 1;

    let world_name = save_config.world_name.clone();
    let sender = worker.sender.clone();
    task.spawn_io(move || {
        let error = RegionManager::write_chunks_batch(&world_name, &batch)
            .err()
            .map(|error| error.to_string());
        let failed = error.clone();
        let _ = sender.send(SaveBatchCompletion {
            chunks: batch,
            error,
        });
        match failed {
            Some(error) => TaskResult::Failed(error),
            None => TaskResult::Success,
        }
    });
}

/// 同步写完队列中的所有区块。保存并退出必须在离开世界前调用此函数。
pub fn flush_save_queue(
    world_name: &str,
    save_queue: &mut SaveQueue,
    worker: &mut SaveWorker,
) -> Result<usize, super::region::SaveError> {
    let mut saved = wait_for_save_worker(save_queue, worker)?;
    let batch: Vec<SavedChunk> = save_queue.queue.drain(..).collect();
    if batch.is_empty() {
        return Ok(saved);
    }

    if let Err(error) = RegionManager::write_chunks_batch(world_name, &batch) {
        for chunk in batch.into_iter().rev() {
            save_queue.queue.push_front(chunk);
        }
        return Err(error);
    }

    saved += batch.len();
    Ok(saved)
}

fn collect_save_completions(save_queue: &mut SaveQueue, worker: &mut SaveWorker) {
    let completions: Vec<_> = {
        let Ok(receiver) = worker.receiver.lock() else {
            return;
        };
        receiver.try_iter().collect()
    };
    for completion in completions {
        worker.in_flight_batches = worker.in_flight_batches.saturating_sub(1);
        for chunk in &completion.chunks {
            worker.in_flight_positions.remove(&chunk.position);
        }
        if let Some(error) = completion.error {
            log::error!("[存档系统] 后台保存区块失败: {error}");
            for chunk in completion.chunks {
                save_queue.enqueue(chunk);
            }
        } else {
            log::trace!("[存档系统] 后台已保存 {} 个区块", completion.chunks.len());
        }
    }
}

fn wait_for_save_worker(
    save_queue: &mut SaveQueue,
    worker: &mut SaveWorker,
) -> Result<usize, super::region::SaveError> {
    let mut saved = 0;
    while worker.in_flight_batches > 0 {
        let completion = worker
            .receiver
            .lock()
            .map_err(|_| super::region::SaveError::Serialize("保存任务通道已损坏".into()))?
            .recv()
            .map_err(|error| {
                super::region::SaveError::Serialize(format!("保存任务意外终止: {error}"))
            })?;
        worker.in_flight_batches = worker.in_flight_batches.saturating_sub(1);
        for chunk in &completion.chunks {
            worker.in_flight_positions.remove(&chunk.position);
        }
        if let Some(error) = completion.error {
            for chunk in completion.chunks {
                save_queue.enqueue(chunk);
            }
            return Err(super::region::SaveError::Serialize(error));
        }
        saved += completion.chunks.len();
    }
    Ok(saved)
}

/// 从存档文件加载区块，加载到世界数据WorldStorage
pub fn process_load_queue_system(
    mut load_queue: ResMut<LoadQueue>,
    mut world_storage: ResMut<WorldStorage>,
    save_config: Res<SaveConfig>,
    block_registry: Res<BlockRegistry>,
) {
    const MAX_LOAD_PER_FRAME: usize = 4;

    // 需要重映射的区块
    let level_data = level::load_level(&save_config.world_name).ok();
    let saved_id_map = level_data
        .as_ref()
        .map(|l| l.block_id_map.clone())
        .unwrap_or_default();

    let mut loaded = 0;
    while loaded < MAX_LOAD_PER_FRAME {
        let Some(saved) = load_queue.queue.pop_front() else {
            break;
        };

        let mut chunk_data = saved.data;

        // 如果存档中有 ID 映射，进行重映射
        if !saved_id_map.is_empty() {
            level::remap_chunk_block_ids(&mut chunk_data, &saved_id_map, &block_registry);
        }

        world_storage
            .loaded_chunks
            .insert(saved.position, Arc::from(chunk_data));

        loaded += 1;
    }

    if loaded > 0 {
        log::trace!("[存档系统] 已加载 {} 个区块", loaded);
    }
}

/// 从存档文件读取单个区块
pub fn try_load_chunk_from_disk(world_name: &str, chunk_pos: IVec3) -> Option<SavedChunk> {
    match RegionManager::read_chunk(world_name, chunk_pos) {
        Ok(Some(saved)) => Some(saved),
        Ok(None) => None,
        Err(e) => {
            log::error!("[存档系统] 加载区块 {:?} 失败: {e}", chunk_pos);
            None
        }
    }
}

/// 加载整个世界
pub fn load_world_metadata(world_name: &str) -> Result<LevelData, super::region::SaveError> {
    level::load_level(world_name)
}

/// 保存整个世界
pub fn save_entire_world(
    world_name: &str,
    world_storage: &WorldStorage,
    block_registry: &BlockRegistry,
    seed: u64,
    spawn_pos: Vec3,
    time_of_day: f32,
) -> Result<(), super::region::SaveError> {
    // 保存世界数据到 level.dat
    level::save_level(world_name, seed, spawn_pos, time_of_day, block_registry)?;

    // 获取当前时间戳
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    // 批量保存所有区块
    let chunks: Vec<SavedChunk> = world_storage
        .loaded_chunks
        .iter()
        .map(|(pos, data)| SavedChunk {
            position: *pos,
            data: data.as_ref().clone(),
            modified_time: world_storage
                .chunk_modified_times
                .get(pos)
                .copied()
                .unwrap_or(now),
        })
        .collect();

    RegionManager::write_chunks_batch(world_name, &chunks)?;

    log::info!("[存档系统] 世界已保存: {} 个区块", chunks.len());
    Ok(())
}

/// 从存档创建初始 WorldStorage
pub fn load_entire_world(
    world_name: &str,
    block_registry: &BlockRegistry,
) -> Result<(WorldStorage, LevelData), super::region::SaveError> {
    let level = level::load_level(world_name)?;
    let mut storage = WorldStorage::default();

    // 遍历所有 region 文件
    let regions_dir = RegionManager::save_root(world_name).join(REGION_DIR_NAME);
    if regions_dir.exists() {
        for entry in std::fs::read_dir(&regions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "bin") {
                // 解析文件名获取 region 坐标
                let stem = path.file_stem().unwrap().to_string_lossy();
                let parts: Vec<&str> = stem.split('.').collect();
                if parts.len() == 4
                    && parts[0] == REGION_FILE_PREFIX
                    && let (Ok(rx), Ok(ry), Ok(rz)) = (
                        parts[1].parse::<i32>(),
                        parts[2].parse::<i32>(),
                        parts[3].parse::<i32>(),
                    )
                {
                    let region_pos = IVec3::new(rx, ry, rz);

                    // 读取该 region 中所有区块
                    let region_path = RegionManager::region_path(world_name, region_pos);
                    if let Ok(region) = RegionManager::read_region_path(&region_path) {
                        for compressed in &region.chunks {
                            if let Ok(decompressed) = RegionManager::decompress(compressed)
                                && let Ok(mut saved) = bincode::DefaultOptions::new()
                                    .with_varint_encoding()
                                    .deserialize::<SavedChunk>(&decompressed)
                            {
                                level::remap_chunk_block_ids(
                                    &mut saved.data,
                                    &level.block_id_map,
                                    block_registry,
                                );
                                storage
                                    .loaded_chunks
                                    .insert(saved.position, Arc::from(saved.data));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok((storage, level))
}

/// 构建重映射表
pub fn cache_level_data_on_enter(
    save_config: Res<SaveConfig>,
    block_registry: Res<BlockRegistry>,
    mut cached_remap: ResMut<CachedBlockIdRemap>,
) {
    match level::load_level(&save_config.world_name) {
        Ok(level_data) => {
            cached_remap.0 = block_registry.build_id_remap_table(&level_data.block_id_map);
            log::info!(
                "[存档系统] level.dat 已缓存，block_id_map 含 {} 条记录",
                level_data.block_id_map.len()
            );
        }
        Err(_) => {
            // 新存档没有 level.dat，正常
            log::info!("[存档系统] 未找到 level.dat，将使用纯生成模式");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::world::chunk::ChunkData;

    fn saved_chunk(position: IVec3, modified_time: f64, first_voxel: u16) -> SavedChunk {
        let mut data = ChunkData::default();
        data.voxels[0] = first_voxel;
        SavedChunk {
            position,
            data,
            modified_time,
        }
    }

    #[test]
    fn save_queue_coalesces_snapshots_and_keeps_the_newest() {
        let position = IVec3::new(1, 2, 3);
        let mut queue = SaveQueue::default();
        queue.enqueue(saved_chunk(position, 10.0, 10));
        queue.enqueue(saved_chunk(position, 20.0, 20));
        queue.enqueue(saved_chunk(position, 15.0, 15));

        assert_eq!(queue.queue.len(), 1);
        let saved = queue.queue.front().unwrap();
        assert_eq!(saved.modified_time, 20.0);
        assert_eq!(saved.data.voxels[0], 20);
    }
}
