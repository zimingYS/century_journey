//! 应用流程在菜单、加载和运行时之间共享的状态与命令。

use bevy::prelude::*;

use crate::app::settings::{BindingKey, KeyAction};

/// 当前单机游戏会话的应用级状态。
#[derive(Resource, Debug, Default)]
pub struct GameSession {
    /// 是否刚完成一次世界装载，用于触发本轮内容消费者刷新。
    pub fresh_load: bool,
    /// 当前进入的世界标识；主菜单阶段为 `None`。
    pub active_world: Option<String>,
}

/// 世界选择列表中的只读摘要。
#[derive(Debug, Clone)]
pub struct WorldSummary {
    /// 世界存档标识。
    pub id: String,
    /// 世界生成种子。
    pub seed: u64,
    /// 存档目录最近修改的 Unix 秒数。
    pub modified_unix: u64,
}

/// 主菜单展示和选择的世界目录。
#[derive(Resource, Debug, Default)]
pub struct WorldCatalog {
    /// 按最近修改时间降序排列的世界摘要。
    pub worlds: Vec<WorldSummary>,
    /// 当前选中的世界标识。
    pub selected: Option<String>,
}

/// 即将进入加载流程的世界标识。
#[derive(Resource, Debug, Default)]
pub struct PendingWorld(pub Option<String>);

/// 加载界面当前展示的标题和细节。
#[derive(Resource, Debug, Clone)]
pub struct LoadingStatus {
    /// 加载阶段标题。
    pub title: String,
    /// 当前步骤说明。
    pub detail: String,
}

impl Default for LoadingStatus {
    fn default() -> Self {
        Self {
            title: "正在启动".into(),
            detail: "正在加载内容资源...".into(),
        }
    }
}

/// 应用流程可展示的模态对话框类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogKind {
    /// 确认删除指定世界。
    ConfirmDelete { world_id: String },
    /// 确认从备份恢复世界元数据。
    ConfirmRecoverWorld { world_id: String },
    /// 确认从备份恢复玩家存档。
    ConfirmRecoverPlayer { world_id: String },
    /// 确认从备份恢复应用设置。
    ConfirmRecoverSettings,
    /// 仅展示错误，不执行确认操作。
    Error,
}

impl DialogKind {
    /// 判断对话框是否需要显示确认按钮。
    pub fn requires_confirmation(&self) -> bool {
        !matches!(self, Self::Error)
    }
}

/// 当前模态对话框的显示状态。
#[derive(Resource, Debug, Default)]
pub struct DialogState {
    /// 当前对话框类型；`None` 表示没有打开对话框。
    pub kind: Option<DialogKind>,
    /// 面向玩家的对话框标题。
    pub title: String,
    /// 面向玩家的对话框正文。
    pub message: String,
}

impl DialogState {
    /// 用统一错误样式覆盖当前对话框内容。
    pub fn error(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.kind = Some(DialogKind::Error);
        self.title = title.into();
        self.message = message.into();
    }

    /// 关闭对话框并清空显示文本。
    pub fn clear(&mut self) {
        self.kind = None;
        self.title.clear();
        self.message.clear();
    }
}

/// 主菜单或暂停菜单当前显示的分页。
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuPage {
    /// 世界选择页。
    #[default]
    Worlds,
    /// 应用设置页。
    Settings,
}

/// 玩家在设置界面发起的单次调整。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingAction {
    /// 按给定步数调整渲染距离。
    RenderDistance(i32),
    /// 按给定增量调整主音量。
    MasterVolume(f32),
    /// 按给定增量调整鼠标灵敏度。
    MouseSensitivity(f32),
    /// 按给定增量调整 UI 缩放。
    UiScale(f32),
    /// 切换无边框全屏。
    ToggleFullscreen,
    /// 切换垂直同步。
    ToggleVsync,
}

/// 菜单和输入层提交给应用流程的命令。
#[derive(Message, Debug, Clone)]
pub enum FlowCommand {
    /// 重新扫描世界目录。
    RefreshWorlds,
    /// 选中指定世界。
    SelectWorld(String),
    /// 使用玩家输入的名称创建世界。
    CreateWorld(String),
    /// 进入当前选中的世界。
    PlaySelected,
    /// 请求确认删除当前世界。
    RequestDeleteSelected,
    /// 执行当前确认对话框对应的操作。
    ConfirmDialog,
    /// 取消并关闭当前对话框。
    CancelDialog,
    /// 打开设置页。
    OpenSettings,
    /// 返回世界列表页。
    CloseSettings,
    /// 从暂停状态恢复游戏。
    Resume,
    /// 保存当前会话并返回主菜单。
    SaveAndQuit,
    /// 退出应用进程。
    QuitApplication,
    /// 调整一项应用设置。
    AdjustSetting(SettingAction),
    /// 修改一个键位绑定；None 表示解除绑定。
    RebindKey(KeyAction, Option<BindingKey>),
    /// 恢复全部键位为默认布局。
    ResetKeybinds,
}

/// 延迟到 Update 流程执行的保存退出请求。
#[derive(Resource, Default)]
pub(super) struct SaveAndQuitRequest(pub(super) bool);
