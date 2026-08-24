//! 键位绑定资源的数据模型、默认表与查询逻辑。
//!
//! 本模块只定义绑定契约：动作有哪些、默认键是什么、某键当前是否按住、
//! 是否与其他动作冲突。磁盘 TOML 编解码见 `keybinds_toml`。

use bevy::input::mouse::MouseButton;
use bevy::prelude::*;

/// 一条可被玩家重新绑定的离散动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    Jump,
    Sprint,
    Squat,
    BreakOrAttack,
    PlaceOrUse,
    DropItem,
    Hotbar1,
    Hotbar2,
    Hotbar3,
    Hotbar4,
    Hotbar5,
    Hotbar6,
    Hotbar7,
    Hotbar8,
    Hotbar9,
    ToggleInventory,
    TogglePerspective,
    ToggleDebugOverlay,
    ToggleSkeletonDebug,
}

impl KeyAction {
    /// 动作在紧凑绑定数组中的固定元素数。
    pub const COUNT: usize = 23;

    /// 返回动作在紧凑绑定数组中的稳定索引。
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// 一个物理输入：键盘按键或鼠标按键。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingKey {
    Key(KeyCode),
    Mouse(MouseButton),
}

/// 单个动作的注册信息，构成键位注册表。
pub struct KeyActionSpec {
    /// 动作本体。
    pub action: KeyAction,
    /// TOML 配置中的稳定标识符。
    pub id: &'static str,
    /// 界面分组名称。
    pub category: &'static str,
    /// 界面显示名称。
    pub display: &'static str,
    /// 默认绑定；当前布局的固定起点。
    pub default: Option<BindingKey>,
}

/// 全部可重绑定动作。新增动作 = 追加一行 + 枚举加变体。
pub const KEY_ACTIONS: &[KeyActionSpec] = &[
    KeyActionSpec {
        action: KeyAction::MoveForward,
        id: "move_forward",
        category: "移动",
        display: "前进",
        default: Some(BindingKey::Key(KeyCode::KeyW)),
    },
    KeyActionSpec {
        action: KeyAction::MoveBackward,
        id: "move_backward",
        category: "移动",
        display: "后退",
        default: Some(BindingKey::Key(KeyCode::KeyS)),
    },
    KeyActionSpec {
        action: KeyAction::MoveLeft,
        id: "move_left",
        category: "移动",
        display: "向左移动",
        default: Some(BindingKey::Key(KeyCode::KeyA)),
    },
    KeyActionSpec {
        action: KeyAction::MoveRight,
        id: "move_right",
        category: "移动",
        display: "向右移动",
        default: Some(BindingKey::Key(KeyCode::KeyD)),
    },
    KeyActionSpec {
        action: KeyAction::Jump,
        id: "jump",
        category: "移动",
        display: "跳跃",
        default: Some(BindingKey::Key(KeyCode::Space)),
    },
    KeyActionSpec {
        action: KeyAction::Sprint,
        id: "sprint",
        category: "移动",
        display: "疾跑",
        default: Some(BindingKey::Key(KeyCode::ShiftLeft)),
    },
    KeyActionSpec {
        action: KeyAction::Squat,
        id: "squat",
        category: "移动",
        display: "下蹲",
        default: Some(BindingKey::Key(KeyCode::ControlLeft)),
    },
    KeyActionSpec {
        action: KeyAction::BreakOrAttack,
        id: "break_or_attack",
        category: "战斗",
        display: "破坏方块 / 攻击",
        default: Some(BindingKey::Mouse(MouseButton::Left)),
    },
    KeyActionSpec {
        action: KeyAction::PlaceOrUse,
        id: "place_or_use",
        category: "战斗",
        display: "放置方块 / 使用",
        default: Some(BindingKey::Mouse(MouseButton::Right)),
    },
    KeyActionSpec {
        action: KeyAction::DropItem,
        id: "drop_item",
        category: "物品",
        display: "丢弃物品",
        default: Some(BindingKey::Key(KeyCode::KeyQ)),
    },
    KeyActionSpec {
        action: KeyAction::Hotbar1,
        id: "hotbar_1",
        category: "物品",
        display: "快捷栏 1",
        default: Some(BindingKey::Key(KeyCode::Digit1)),
    },
    KeyActionSpec {
        action: KeyAction::Hotbar2,
        id: "hotbar_2",
        category: "物品",
        display: "快捷栏 2",
        default: Some(BindingKey::Key(KeyCode::Digit2)),
    },
    KeyActionSpec {
        action: KeyAction::Hotbar3,
        id: "hotbar_3",
        category: "物品",
        display: "快捷栏 3",
        default: Some(BindingKey::Key(KeyCode::Digit3)),
    },
    KeyActionSpec {
        action: KeyAction::Hotbar4,
        id: "hotbar_4",
        category: "物品",
        display: "快捷栏 4",
        default: Some(BindingKey::Key(KeyCode::Digit4)),
    },
    KeyActionSpec {
        action: KeyAction::Hotbar5,
        id: "hotbar_5",
        category: "物品",
        display: "快捷栏 5",
        default: Some(BindingKey::Key(KeyCode::Digit5)),
    },
    KeyActionSpec {
        action: KeyAction::Hotbar6,
        id: "hotbar_6",
        category: "物品",
        display: "快捷栏 6",
        default: Some(BindingKey::Key(KeyCode::Digit6)),
    },
    KeyActionSpec {
        action: KeyAction::Hotbar7,
        id: "hotbar_7",
        category: "物品",
        display: "快捷栏 7",
        default: Some(BindingKey::Key(KeyCode::Digit7)),
    },
    KeyActionSpec {
        action: KeyAction::Hotbar8,
        id: "hotbar_8",
        category: "物品",
        display: "快捷栏 8",
        default: Some(BindingKey::Key(KeyCode::Digit8)),
    },
    KeyActionSpec {
        action: KeyAction::Hotbar9,
        id: "hotbar_9",
        category: "物品",
        display: "快捷栏 9",
        default: Some(BindingKey::Key(KeyCode::Digit9)),
    },
    KeyActionSpec {
        action: KeyAction::ToggleInventory,
        id: "toggle_inventory",
        category: "界面",
        display: "打开 / 关闭背包",
        default: Some(BindingKey::Key(KeyCode::KeyE)),
    },
    KeyActionSpec {
        action: KeyAction::TogglePerspective,
        id: "toggle_perspective",
        category: "视角",
        display: "切换观察视角",
        default: Some(BindingKey::Key(KeyCode::F5)),
    },
    KeyActionSpec {
        action: KeyAction::ToggleDebugOverlay,
        id: "toggle_debug_overlay",
        category: "调试",
        display: "调试浮层",
        default: Some(BindingKey::Key(KeyCode::F3)),
    },
    KeyActionSpec {
        action: KeyAction::ToggleSkeletonDebug,
        id: "toggle_skeleton_debug",
        category: "调试",
        display: "玩家骨架调试",
        default: Some(BindingKey::Key(KeyCode::F7)),
    },
];

/// 当前生效的键位绑定表，按 `KeyAction::index` 紧凑存储。
///
/// 固定不参与重映射的输入：鼠标移动视角、滚轮切换快捷栏、
/// Ctrl+F5 存档与 F9 元数据检查等调试组合、聊天框的 Enter/Tab/上下键。
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Keybinds {
    bindings: Vec<Option<BindingKey>>,
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            bindings: KEY_ACTIONS.iter().map(|spec| spec.default).collect(),
        }
    }
}

impl Keybinds {
    /// 返回动作当前绑定的物理输入；未绑定时返回 None。
    pub fn binding(&self, action: KeyAction) -> Option<BindingKey> {
        self.bindings[action.index()]
    }

    /// 设置动作绑定并做规范化：左右修饰键统一为左侧代表键，避免丢失另一侧。
    pub fn set_binding(&mut self, action: KeyAction, binding: Option<BindingKey>) {
        self.bindings[action.index()] = binding.map(normalize_binding);
    }

    /// 恢复全部默认键位。
    pub fn reset_all(&mut self) {
        *self = Self::default();
    }

    /// 判断动作绑定的输入本帧是否按住。
    pub fn is_held(
        &self,
        action: KeyAction,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        match self.binding(action) {
            Some(BindingKey::Key(code)) => held_with_modifier(code, keyboard),
            Some(BindingKey::Mouse(button)) => mouse.pressed(button),
            None => false,
        }
    }

    /// 判断动作绑定的输入本帧是否刚按下。
    pub fn is_just_pressed(
        &self,
        action: KeyAction,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        match self.binding(action) {
            Some(BindingKey::Key(code)) => {
                keyboard.just_pressed(code)
                    || modifier_group(code).is_some_and(|pair| {
                        keyboard.just_pressed(pair.0) || keyboard.just_pressed(pair.1)
                    })
            }
            Some(BindingKey::Mouse(button)) => mouse.just_pressed(button),
            None => false,
        }
    }

    /// 返回与指定动作绑定到同一输入的其他动作显示名，用于冲突提示。
    pub fn conflict_partners(&self, action: KeyAction) -> Vec<&'static str> {
        let Some(binding) = self.binding(action) else {
            return Vec::new();
        };
        KEY_ACTIONS
            .iter()
            .filter(|spec| spec.action != action && self.binding(spec.action) == Some(binding))
            .map(|spec| spec.display)
            .collect()
    }

    /// 判断动作当前是否与其他动作冲突。
    pub fn has_conflict(&self, action: KeyAction) -> bool {
        !self.conflict_partners(action).is_empty()
    }

    /// 判断动作是否通过键位界面的过滤条件。
    ///
    /// 搜索词匹配动作显示名或键名（忽略大小写）；两个开关可分别只看
    /// 冲突或未绑定的条目。
    pub fn matches_filter(
        &self,
        action: KeyAction,
        search: &str,
        conflicts_only: bool,
        unbound_only: bool,
    ) -> bool {
        if conflicts_only && !self.has_conflict(action) {
            return false;
        }
        if unbound_only && self.binding(action).is_some() {
            return false;
        }
        let query = search.trim();
        if query.is_empty() {
            return true;
        }
        let query = query.to_lowercase();
        let spec = spec_of(action);
        if spec.display.to_lowercase().contains(&query) {
            return true;
        }
        match self.binding(action) {
            Some(binding) => super::keybinds_toml::binding_key_name(binding)
                .to_lowercase()
                .contains(&query),
            None => false,
        }
    }
}

/// 返回动作的注册表信息；动作枚举与注册表一一对应。
pub fn spec_of(action: KeyAction) -> &'static KeyActionSpec {
    KEY_ACTIONS
        .iter()
        .find(|spec| spec.action == action)
        .expect("键位注册表必须覆盖全部动作")
}

/// 规范化绑定：左右 Shift/Ctrl/Alt 统一记为左侧代表键，查询时按组匹配。
fn normalize_binding(binding: BindingKey) -> BindingKey {
    match binding {
        BindingKey::Key(code) => match modifier_group(code) {
            Some(pair) => BindingKey::Key(pair.0),
            None => BindingKey::Key(code),
        },
        mouse => mouse,
    }
}

/// 返回修饰键所属的左右一对；非修饰键返回 None。
fn modifier_group(code: KeyCode) -> Option<(KeyCode, KeyCode)> {
    match code {
        KeyCode::ShiftLeft | KeyCode::ShiftRight => Some((KeyCode::ShiftLeft, KeyCode::ShiftRight)),
        KeyCode::ControlLeft | KeyCode::ControlRight => {
            Some((KeyCode::ControlLeft, KeyCode::ControlRight))
        }
        KeyCode::AltLeft | KeyCode::AltRight => Some((KeyCode::AltLeft, KeyCode::AltRight)),
        _ => None,
    }
}

/// 按住检查；修饰键左右任意一侧按住都视为按住。
fn held_with_modifier(code: KeyCode, keyboard: &ButtonInput<KeyCode>) -> bool {
    if keyboard.pressed(code) {
        return true;
    }
    modifier_group(code).is_some_and(|pair| keyboard.pressed(pair.0) || keyboard.pressed(pair.1))
}

#[cfg(test)]
#[path = "../../../tests/unit/app/settings/keybinds.rs"]
mod tests;
