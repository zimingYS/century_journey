use crate::content::constant::world::MAX_SAVE_PER_FRAME;
use crate::engine::task::{TaskManager, TaskResult};
use crate::game::save::config::SaveConfig;
use crate::game::save::world::chunk::model::SavedChunk;
use crate::game::save::world::chunk::region::RegionManager;
use bevy::math::IVec3;
use bevy::prelude;
use bevy::prelude::{Res, ResMut, Resource};
use std::collections::{HashSet, VecDeque};
use std::sync::{Mutex, mpsc};

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

#[cfg(test)]
#[path = "../../../../../tests/unit/game/save/world/chunk/queue.rs"]
mod tests;

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
