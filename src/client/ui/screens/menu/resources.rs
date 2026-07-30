//! 菜单表现层拥有的临时资源。

use bevy::prelude::*;

/// 缓存创建世界输入框中的草稿，不作为世界或存档的权威名称来源。
#[derive(Resource, Debug, Clone)]
pub(crate) struct WorldNameDraft(pub(super) String);

impl Default for WorldNameDraft {
    fn default() -> Self {
        Self("new_world".into())
    }
}

/// 注册菜单 UI 生命周期所需的本地资源。
pub(crate) fn init_menu_resources(app: &mut App) {
    app.init_resource::<WorldNameDraft>();
}
