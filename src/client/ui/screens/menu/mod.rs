//! 主菜单、暂停菜单、设置页与流程对话框的客户端表现入口。
//!
//! 本模块仅负责客户端 UI：构建实体、同步流程状态与把玩家交互转换为
//! `FlowCommand`。世界创建、存档和应用状态迁移仍由 `app::flow` 持有。

mod components;
mod interaction;
mod resources;
mod settings;
mod spawn;
mod style;
mod sync;

pub(crate) use components::{PauseSettingsButton, ResumeButton, SaveQuitButton};
pub(crate) use interaction::menu_button_system;
pub(crate) use resources::init_menu_resources;
pub(crate) use spawn::spawn_menu_screens_system;
pub(crate) use sync::{
    populate_world_list_system, sync_dialog_text_system, sync_flow_screen_stack_system,
    sync_loading_text_system, sync_menu_visibility_system, sync_setting_values_system,
    sync_world_name_draft_system,
};
