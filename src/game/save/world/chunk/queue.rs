//! 串行化区块写入意图，并向异步加载提供“最新已接受快照”读取屏障。

use crate::engine::task::{TaskManager, TaskResult};
use crate::game::save::config::SaveConfig;
use crate::game::save::world::chunk::model::SavedChunk;
use crate::game::save::world::chunk::region::RegionManager;
use bevy::math::IVec3;
use bevy::prelude;
use bevy::prelude::{Res, ResMut, Resource};
use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, mpsc};

/// 单次后台写盘批次最多包含的区块数，限制复制与压缩造成的帧尖峰。
const MAX_SAVE_PER_FRAME: usize = 4;

/// 尚未交给后台写盘的区块快照队列。
#[derive(Resource, Default, Debug)]
pub struct SaveQueue {
    pub queue: VecDeque<SavedChunk>,
}

impl SaveQueue {
    /// 同一区块只保留时间更新的快照；时间相同时以后来提交者为准。
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

/// 流式区块保存后台状态，并保留飞行中快照供加载屏障查询。
///
/// 只允许一个批次写盘，避免同一 Region 并发覆盖；飞行中最多四个快照，
/// 有界克隆换取了卸载后立即重载时不会读取旧磁盘状态。
#[derive(Resource)]
pub struct SaveWorker {
    sender: mpsc::Sender<SaveBatchCompletion>,
    receiver: Mutex<mpsc::Receiver<SaveBatchCompletion>>,
    in_flight_snapshots: HashMap<IVec3, SavedChunk>,
    in_flight_batches: usize,
}

impl Default for SaveWorker {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
            in_flight_snapshots: HashMap::new(),
            in_flight_batches: 0,
        }
    }
}

impl SaveWorker {
    /// 判断后台是否没有正在写入的区块批次。
    pub fn is_idle(&self) -> bool {
        self.in_flight_batches == 0
    }
}

/// 返回加载必须看到的最新内存快照，但不消费其保存意图。
///
/// 同坐标待写队列一定晚于已经派发的飞行中批次，因此即便两个墙钟时间相同，
/// 也必须优先选择队列，随后才选择飞行中快照，最后由调用方读取磁盘。
pub(in crate::game) fn latest_snapshot_for_load(
    queue: &SaveQueue,
    worker: &SaveWorker,
    position: IVec3,
) -> Option<SavedChunk> {
    queue
        .queue
        .iter()
        .find(|chunk| chunk.position == position)
        .cloned()
        .or_else(|| worker.in_flight_snapshots.get(&position).cloned())
}

/// 每帧收集完成结果并按固定批量派发新的后台写盘任务。
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
        if worker.in_flight_snapshots.contains_key(&chunk.position) {
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
        worker
            .in_flight_snapshots
            .insert(chunk.position, chunk.clone());
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

/// 同步写完队列中的所有区块；世界切换和保存退出必须先经过此屏障。
pub fn flush_save_queue(
    world_name: &str,
    save_queue: &mut SaveQueue,
    worker: &mut SaveWorker,
) -> prelude::Result<usize, super::region::SaveError> {
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
        match apply_completion(save_queue, worker, completion) {
            Ok(saved) => log::trace!("[存档系统] 后台已保存 {saved} 个区块"),
            Err(error) => log::error!("[存档系统] 后台保存区块失败: {error}"),
        }
    }
}

fn apply_completion(
    save_queue: &mut SaveQueue,
    worker: &mut SaveWorker,
    completion: SaveBatchCompletion,
) -> Result<usize, String> {
    worker.in_flight_batches = worker.in_flight_batches.saturating_sub(1);
    for chunk in &completion.chunks {
        worker.in_flight_snapshots.remove(&chunk.position);
    }

    if let Some(error) = completion.error {
        for chunk in completion.chunks {
            // 若写盘期间已经产生更新快照，旧失败批次不得覆盖或替换它。
            if !save_queue
                .queue
                .iter()
                .any(|queued| queued.position == chunk.position)
            {
                save_queue.queue.push_back(chunk);
            }
        }
        return Err(error);
    }
    Ok(completion.chunks.len())
}

fn wait_for_save_worker(
    save_queue: &mut SaveQueue,
    worker: &mut SaveWorker,
) -> prelude::Result<usize, super::region::SaveError> {
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
        match apply_completion(save_queue, worker, completion) {
            Ok(count) => saved += count,
            Err(error) => return Err(super::region::SaveError::Serialize(error)),
        }
    }
    Ok(saved)
}

#[cfg(test)]
#[path = "../../../../../tests/unit/game/save/world/chunk/queue.rs"]
mod tests;
