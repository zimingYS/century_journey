//! 菜单表现层拥有的临时资源。

use bevy::prelude::*;

use super::components::SettingsTab;

/// 缓存创建世界输入框中的草稿，不作为世界或存档的权威名称来源。
#[derive(Resource, Debug, Clone)]
pub(crate) struct WorldNameDraft(pub(super) String);

impl Default for WorldNameDraft {
    fn default() -> Self {
        Self("new_world".into())
    }
}

/// 键位设置页的本地界面状态；绑定权威保存在 `Keybinds` 资源。
#[derive(Resource, Debug, Clone)]
pub(crate) struct KeybindsUiState {
    /// 当前页签。
    pub(crate) tab: SettingsTab,
    /// 搜索关键词，匹配动作名或键名。
    pub(crate) search: String,
    /// 仅显示有冲突的条目。
    pub(crate) conflicts_only: bool,
    /// 仅显示未绑定的条目。
    pub(crate) unbound_only: bool,
    /// 列表需要重建的标记。
    pub(crate) list_dirty: bool,
}

impl Default for KeybindsUiState {
    fn default() -> Self {
        Self {
            tab: SettingsTab::General,
            search: String::new(),
            conflicts_only: false,
            unbound_only: false,
            list_dirty: true,
        }
    }
}

/// 注册菜单 UI 生命周期所需的本地资源。
pub(crate) fn init_menu_resources(app: &mut App) {
    app.init_resource::<WorldNameDraft>();
    app.init_resource::<KeybindsUiState>();
}
