//! 菜单 UI 的 ECS 标记组件。
//!
//! 这些组件只标识表现实体，不保存权威游戏状态。

use bevy::prelude::*;

use crate::app::flow::SettingAction;
use crate::app::settings::KeyAction;

/// 标记主菜单中的世界列表容器。
#[derive(Component)]
pub(crate) struct WorldList;

/// 标记创建世界时使用的名称输入框。
#[derive(Component)]
pub(crate) struct WorldNameInput;

/// 标记一个可选择的世界条目，并保存其稳定世界标识。
#[derive(Component)]
pub(crate) struct WorldEntryButton {
    pub(super) id: String,
}

/// 标记进入所选世界的按钮。
#[derive(Component, Default)]
pub(crate) struct PlayButton;

/// 标记创建世界的按钮。
#[derive(Component, Default)]
pub(crate) struct CreateButton;

/// 标记请求删除所选世界的按钮。
#[derive(Component, Default)]
pub(crate) struct DeleteButton;

/// 标记主菜单中的设置按钮。
#[derive(Component, Default)]
pub(crate) struct MainSettingsButton;

/// 标记退出应用的按钮。
#[derive(Component, Default)]
pub(crate) struct QuitButton;

/// 标记从暂停状态恢复游戏的按钮。
///
/// 截图检查会直接查找该标记，因此在菜单模块入口保留稳定重导出。
#[derive(Component, Default)]
pub(crate) struct ResumeButton;

/// 标记暂停菜单中的设置按钮。
///
/// 截图检查会直接查找该标记，因此在菜单模块入口保留稳定重导出。
#[derive(Component, Default)]
pub(crate) struct PauseSettingsButton;

/// 标记保存当前世界并返回主菜单的按钮。
///
/// 截图检查会直接查找该标记，因此在菜单模块入口保留稳定重导出。
#[derive(Component, Default)]
pub(crate) struct SaveQuitButton;

/// 标记关闭设置页的按钮。
#[derive(Component, Default)]
pub(crate) struct SettingsBackButton;

/// 标记确认当前流程对话框的按钮。
#[derive(Component, Default)]
pub(crate) struct DialogConfirmButton;

/// 标记取消当前流程对话框的按钮。
#[derive(Component, Default)]
pub(crate) struct DialogCancelButton;

/// 标记设置页的根实体。
#[derive(Component)]
pub(crate) struct SettingsRoot;

/// 标记流程对话框的根实体。
#[derive(Component)]
pub(crate) struct DialogRoot;

/// 标记流程对话框的标题文本。
#[derive(Component)]
pub(crate) struct DialogTitle;

/// 标记流程对话框的正文文本。
#[derive(Component)]
pub(crate) struct DialogMessage;

/// 标记加载界面的标题文本。
#[derive(Component)]
pub(crate) struct LoadingTitle;

/// 标记加载界面的详情文本。
#[derive(Component)]
pub(crate) struct LoadingDetail;

/// 将设置按钮绑定到应用层定义的设置调整命令。
#[derive(Component, Clone, Copy)]
pub(crate) struct SettingButton(pub(super) SettingAction);

/// 标识设置页中需要从 `GameSettings` 同步的显示值。
#[derive(Component, Clone, Copy)]
pub(crate) enum SettingValue {
    RenderDistance,
    MasterVolume,
    MouseSensitivity,
    UiScale,
    Language,
    Fullscreen,
    Vsync,
}

/// 标记设置页顶部的页签按钮，并记录其目标页。
#[derive(Component, Clone, Copy)]
pub(crate) struct SettingsTabButton {
    pub(super) tab: SettingsTab,
}

/// 设置页内的通用设置内容容器。
#[derive(Component)]
pub(crate) struct SettingsGeneralPage;

/// 设置页内的键位列表内容容器。
#[derive(Component)]
pub(crate) struct SettingsKeybindsPage;

/// 标记键位搜索输入框。
#[derive(Component)]
pub(crate) struct KeybindSearchInput;

/// 标记键位列表的动态行容器。
#[derive(Component)]
pub(crate) struct KeybindList;

/// 标记一行键位条目，用于重建列表时整体清理。
#[derive(Component)]
pub(crate) struct KeybindRow;

/// 标记触发重新绑定的键位按钮，并保存其动作。
#[derive(Component, Clone, Copy)]
pub(crate) struct KeybindKeyButton {
    pub(super) action: KeyAction,
}

/// 键位过滤开关的种类。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum KeybindFilter {
    /// 仅显示有冲突的条目。
    Conflicts,
    /// 仅显示未绑定的条目。
    Unbound,
}

/// 标记键位过滤开关按钮。
#[derive(Component, Clone, Copy)]
pub(crate) struct KeybindFilterButton {
    pub(super) filter: KeybindFilter,
}

/// 标记恢复全部默认键位的按钮。
#[derive(Component, Default)]
pub(crate) struct KeybindResetButton;

/// 设置页的页签。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum SettingsTab {
    /// 通用设置页。
    #[default]
    General,
    /// 键位设置页。
    Keybinds,
}
