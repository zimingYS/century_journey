//! 定义远景真实方块 LOD 后台任务返回主线程的结果通道。

use super::block_mesh::DistantTerrainBlockMeshData;
use super::planner::DistantTerrainTileKey;
use bevy::prelude::Resource;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex, mpsc};

/// 单个异步远景真实方块 LOD 构建结果。
pub(super) struct DistantTerrainBuildResult {
    /// 当前会话世代，用于拒绝上个世界延迟到达的结果。
    pub(super) session_generation: u64,
    /// 当前同键请求版本，用于拒绝同一瓦片的过期构建结果。
    pub(super) request_id: u64,
    /// 目标远景瓦片键。
    pub(super) key: DistantTerrainTileKey,
    /// 构建该网格时使用的覆盖位图；玩家移动后位图变化时据此判断是否原地更新。
    pub(super) coverage_mask: [u64; 4],
    /// 不持有 ECS 或 GPU 资产的纯网格数据。
    pub(super) mesh: DistantTerrainBlockMeshData,
}

/// Client 层远景真实方块 LOD 任务通道。
#[derive(Resource)]
pub(crate) struct DistantTerrainBuildChannel {
    pub(super) sender: mpsc::Sender<DistantTerrainBuildResult>,
    pub(super) receiver: Mutex<mpsc::Receiver<DistantTerrainBuildResult>>,
    pub(super) in_flight: Arc<AtomicUsize>,
}

impl Default for DistantTerrainBuildChannel {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
            in_flight: Arc::new(AtomicUsize::new(0)),
        }
    }
}
