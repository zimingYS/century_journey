//! 保存只影响客户端界面的短生命周期状态。

use bevy::prelude::*;

/// 记录创造物品栏搜索框是否正在接收文本输入。
///
/// 该状态只参与客户端输入上下文解析，不属于权威物品栏数据，也不进入存档。
#[derive(Resource, Default)]
pub struct SearchInputState {
    /// 搜索框当前是否拥有有效输入焦点。
    pub active: bool,
}
