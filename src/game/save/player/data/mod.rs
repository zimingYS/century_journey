//! 组织玩家当前模型、历史只读适配、物品编码和数据校验。

pub mod item_codec;
pub(super) mod legacy_bincode;
pub mod model;
pub mod validation;
