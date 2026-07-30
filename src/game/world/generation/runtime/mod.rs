//! 管理地形与结构生成任务通道及主线程结果接收系统。

mod channel;
mod structure_tasks;
mod terrain_tasks;

pub(crate) use channel::{StructureGenChannel, TerrainGenChannel};
pub(super) use channel::{StructureGenResult, TerrainGenResult};
pub(crate) use structure_tasks::{generate_structures_system, receive_structure_results};
pub(crate) use terrain_tasks::{receive_terrain_results, spawn_terrain_gen_tasks};
