//! 定义数据驱动方块放置约束。

use crate::shared::tag::identifier::TagId;
use serde::{Deserialize, Serialize};

/// 方块放置时需要满足的规则
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlockPlacementConfig {
    /// 放置位置下方方块必须属于的标签
    #[serde(default)]
    pub required_support_tag: Option<TagId>,
}
