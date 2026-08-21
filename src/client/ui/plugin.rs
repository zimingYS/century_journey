//! 组装 Client UI 的共享资源和各子领域插件。

use super::interaction::UiInteractionPlugin;
use super::navigation::{UiNavigation, UiScreenStack};
use super::screens::UiScreensPlugin;
use super::theme::category_theme::CategoryTheme;
use super::theme::scale::UiScaleSettings;
use super::theme::ui_theme::UiTheme;
use super::widgets::UiWidgetsPlugin;
use super::widgets::slot::{CategoryClickedEvent, SearchInputState};
use super::{hud::plugin::HudPlugin, resources};
use crate::client::ui::console::plugin::ConsolePlugin;
use bevy::prelude::*;

/// Client 层界面的聚合插件。
///
/// 本插件只初始化跨 UI 子模块共享的资源与消息，具体系统由各子插件注册。
pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        super::screenshot_check::configure_ui_screenshot_check(app);
        app.add_message::<CategoryClickedEvent>()
            .add_message::<UiNavigation>()
            .init_resource::<UiTheme>()
            .init_resource::<UiScreenStack>()
            .init_resource::<UiScaleSettings>()
            .init_resource::<CategoryTheme>()
            .init_resource::<resources::ui_font::UiFont>()
            .init_resource::<SearchInputState>()
            .add_plugins((
                HudPlugin,
                UiWidgetsPlugin,
                UiInteractionPlugin,
                UiScreensPlugin,
                ConsolePlugin,
            ));
    }
}
