//! 加载由内容定义引用的自定义物品模型资产。

use bevy::prelude::*;

/// 自定义物品模型加载入口；当前仅保留稳定扩展边界。
pub struct CustomItemMeshBuilder;

impl CustomItemMeshBuilder {
    /// 尝试从内容路径构建自定义网格；未支持的格式返回 `None`。
    pub fn build_mesh(_path: &str) -> Option<Mesh> {
        None
    }
}
