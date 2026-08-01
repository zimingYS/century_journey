//! 定义跨渲染帧和固定步传递的玩家动作及其生命周期快照。

use crate::game::player::control::command::PlayerCommand;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
/// 客户端可以提交、由固定步规则解释的离散玩家动作。
pub enum PlayerAction {
    /// 向面朝方向移动。
    MoveForward,
    /// 向面朝反方向移动。
    MoveBackward,
    /// 向左侧移动。
    MoveLeft,
    /// 向右侧移动。
    MoveRight,
    /// 请求加速奔跑。
    Sprint,
    /// 请求下蹲
    Squat,
    /// 请求跳跃。
    Jump,
    /// 持续破坏目标方块。
    BreakBlock,
    /// 发起攻击。
    Attack,
    /// 放置当前持有方块。
    PlaceBlock,
    /// 使用当前物品或目标方块。
    Use,
    /// 丢弃当前物品。
    DropItem,
    /// 选择上一个快捷栏槽位。
    HotbarPrevious,
    /// 选择下一个快捷栏槽位。
    HotbarNext,
    /// 选择快捷栏第一槽。
    Hotbar1,
    /// 选择快捷栏第二槽。
    Hotbar2,
    /// 选择快捷栏第三槽。
    Hotbar3,
    /// 选择快捷栏第四槽。
    Hotbar4,
    /// 选择快捷栏第五槽。
    Hotbar5,
    /// 选择快捷栏第六槽。
    Hotbar6,
    /// 选择快捷栏第七槽。
    Hotbar7,
    /// 选择快捷栏第八槽。
    Hotbar8,
    /// 选择快捷栏第九槽。
    Hotbar9,
    /// 请求切换客户端观察视角。
    TogglePerspective,
}

impl PlayerAction {
    /// 动作枚举在紧凑状态数组中占用的固定元素数。
    pub const COUNT: usize = 24;

    /// 返回动作在紧凑状态数组中的稳定索引。
    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 单个动作相对上一模拟刻的生命周期阶段。
pub enum PlayerActionPhase {
    /// 本刻刚开始生效。
    Pressed,
    /// 从上一刻持续生效。
    Held,
    /// 本刻正常释放。
    Released,
    /// 因输入上下文失效而取消。
    Cancelled,
}

#[derive(Resource, Debug, Clone)]
/// 固定步当前可读取的全部玩家动作状态。
pub struct PlayerActionState {
    active: [bool; PlayerAction::COUNT],
    pressed: [bool; PlayerAction::COUNT],
    released: [bool; PlayerAction::COUNT],
    cancelled: [bool; PlayerAction::COUNT],
}

impl Default for PlayerActionState {
    fn default() -> Self {
        Self {
            active: [false; PlayerAction::COUNT],
            pressed: [false; PlayerAction::COUNT],
            released: [false; PlayerAction::COUNT],
            cancelled: [false; PlayerAction::COUNT],
        }
    }
}

impl PlayerActionState {
    /// 用新动作集合更新状态，并派生按下、释放和取消边沿。
    pub fn update(&mut self, enabled: bool, actions: impl IntoIterator<Item = PlayerAction>) {
        let previous = self.active;
        let mut next = [false; PlayerAction::COUNT];
        if enabled {
            for action in actions {
                next[action.index()] = true;
            }
        }

        for index in 0..PlayerAction::COUNT {
            self.pressed[index] = enabled && next[index] && !previous[index];
            self.released[index] = enabled && !next[index] && previous[index];
            self.cancelled[index] = !enabled && previous[index];
        }
        self.active = next;
    }

    /// 判断动作当前是否持续生效。
    pub fn pressed(&self, action: PlayerAction) -> bool {
        self.active[action.index()]
    }

    /// 判断动作是否在本刻刚按下。
    pub fn just_pressed(&self, action: PlayerAction) -> bool {
        self.pressed[action.index()]
    }

    /// 判断动作是否在本刻正常释放。
    pub fn just_released(&self, action: PlayerAction) -> bool {
        self.released[action.index()]
    }

    /// 判断动作是否因输入上下文关闭而取消。
    pub fn cancelled(&self, action: PlayerAction) -> bool {
        self.cancelled[action.index()]
    }

    /// 返回动作在本刻最具体的生命周期阶段。
    pub fn phase(&self, action: PlayerAction) -> Option<PlayerActionPhase> {
        if self.just_pressed(action) {
            Some(PlayerActionPhase::Pressed)
        } else if self.pressed(action) {
            Some(PlayerActionPhase::Held)
        } else if self.just_released(action) {
            Some(PlayerActionPhase::Released)
        } else if self.cancelled(action) {
            Some(PlayerActionPhase::Cancelled)
        } else {
            None
        }
    }

    /// 清除全部动作和边沿状态。
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// 复制当前持续动作位图，用于构造跨调度命令。
    pub(crate) fn active_snapshot(&self) -> [bool; PlayerAction::COUNT] {
        self.active
    }

    /// 复制本刻按下边沿位图。
    pub(crate) fn pressed_snapshot(&self) -> [bool; PlayerAction::COUNT] {
        self.pressed
    }

    /// 复制本刻释放边沿位图。
    pub(crate) fn released_snapshot(&self) -> [bool; PlayerAction::COUNT] {
        self.released
    }

    /// 复制本刻取消边沿位图。
    pub(crate) fn cancelled_snapshot(&self) -> [bool; PlayerAction::COUNT] {
        self.cancelled
    }

    /// 使用固定步选中的命令完整替换权威动作状态。
    pub(crate) fn apply_command(&mut self, command: &PlayerCommand) {
        self.active = command.active;
        self.pressed = command.pressed;
        self.released = command.released;
        self.cancelled = command.cancelled;
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/player/control/action.rs"]
mod tests;
