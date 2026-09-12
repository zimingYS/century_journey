//! cj-render —— 渲染层
//!
//! 规则：只消费渲染数据，不知道模拟算法（不依赖 sim）。

use bevy::prelude::*;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: 注册体素渲染管线
    }
}
