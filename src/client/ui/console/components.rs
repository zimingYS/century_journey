//! 控制台的数据定义：标记组件、开合状态与提交消息。

use bevy::prelude::*;

/// 控制台覆盖层根节点。
#[derive(Component)]
pub struct ConsoleRoot;

/// 控制台输入框标记（挂 `EditableText` 的实体）。
#[derive(Component)]
pub struct ConsoleInput;

/// 历史回显消息容器。
#[derive(Component)]
pub struct ConsoleHistory;

/// 用户提交的一行文字（输入层采集，UI 层回显）。
#[derive(Message, Debug, Clone)]
pub struct ConsoleLineSubmitted {
    pub text: String,
}

/// 控制台开合状态与持久消息历史。
#[derive(Resource, Default)]
pub struct ConsoleState {
    pub open: bool,
    /// 持久消息历史：UI 淡出隐藏不影响这里，只有显式删除才移除。
    pub history: Vec<String>,
}

/// 单条聊天消息（表现投影；可见性与透明度由 ConsoleState 统一驱动）。
#[derive(Component)]
pub struct ConsoleMessage {
    /// 消息独立显示计时，到期后进入淡出。
    pub timer: Timer,
    /// 是否正在淡出。
    pub fading: bool,
    /// 淡出过渡计时。
    pub fade_timer: Timer,
}
