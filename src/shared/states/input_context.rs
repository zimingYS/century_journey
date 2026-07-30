//! 定义 App 与 Client 共享的输入上下文契约。

use bevy::prelude::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
/// App 与 Client 约定的互斥输入上下文。
pub enum InputContext {
    #[default]
    Gameplay,
    Inventory,
    Menu,
    TextInput,
}

impl InputContext {
    /// 返回上下文冲突解析使用的优先级。
    pub const fn priority(self) -> u8 {
        match self {
            Self::Gameplay => 0,
            Self::Inventory => 1,
            Self::Menu => 2,
            Self::TextInput => 3,
        }
    }

    /// 从候选集合解析优先级最高的输入上下文。
    pub fn resolve(candidates: impl IntoIterator<Item = Self>) -> Self {
        candidates
            .into_iter()
            .max_by_key(|context| context.priority())
            .unwrap_or_default()
    }

    /// 判断该上下文是否允许采集玩家玩法输入。
    pub const fn allows_gameplay(self) -> bool {
        matches!(self, Self::Gameplay)
    }
}

#[derive(Resource, Debug, Default, Clone)]
/// 保存当前输入上下文和暂停菜单意图的共享状态。
pub struct InputContextState {
    active: InputContext,
    menu_open: bool,
}

impl InputContextState {
    /// 返回当前生效的输入上下文。
    pub const fn active(&self) -> InputContext {
        self.active
    }

    /// 返回暂停菜单是否被请求打开。
    pub const fn menu_open(&self) -> bool {
        self.menu_open
    }

    /// 更新当前生效的输入上下文。
    pub fn set_active(&mut self, active: InputContext) {
        self.active = active;
    }

    /// 更新暂停菜单打开意图。
    pub fn set_menu_open(&mut self, open: bool) {
        self.menu_open = open;
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/shared/states/input_context.rs"]
mod tests;
