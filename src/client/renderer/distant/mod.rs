//! 组织基于确定性基础地表的客户端远景 LOD 表现。
//!
//! 本子域只构建远距离的低分辨率真实方块柱，不请求权威区块、不参与玩法或存档，
//! 并通过会话世代和请求编号拒绝异步旧结果。近景仍完全由真实体素网格负责。

mod block_mesh;
mod channel;
mod config;
mod lifecycle;
mod planner;

pub(crate) use channel::DistantTerrainBuildChannel;
pub(crate) use config::DistantTerrainConfig;
pub(crate) use lifecycle::{
    DistantTerrainRuntime, clear_distant_terrain_system, initialize_distant_terrain_system,
    receive_distant_terrain_results_system, spawn_distant_terrain_tasks_system,
    sync_distant_terrain_camera_range_system, sync_distant_terrain_plan_system,
    tick_distant_terrain_expiry_system,
};
