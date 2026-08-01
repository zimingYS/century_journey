//! 组织区块存档模型、区域文件访问、加载与写入队列。

mod codec;
pub(in crate::game::save) mod load;
pub mod model;
pub(in crate::game) mod queue;
pub mod region;
