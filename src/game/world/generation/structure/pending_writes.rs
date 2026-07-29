use bevy::prelude::IVec3;
use std::collections::HashMap;

/// 尚未加载的目标区块需要延后应用的方块写入。
///
/// 该队列属于世界权威状态；结构生成任务只能构造写入结果，主线程负责合并和消费。
#[derive(Default, Debug)]
pub struct PendingVoxelWrites {
    pub writes: HashMap<IVec3, Vec<PendingVoxel>>,
}

/// 单次延迟方块写入的区块局部坐标与方块 ID
#[derive(Debug)]
pub struct PendingVoxel {
    pub local_x: usize,
    pub local_y: usize,
    pub local_z: usize,
    pub block_id: u16,
}
