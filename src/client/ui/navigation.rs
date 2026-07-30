//! 处理界面返回、关闭和焦点切换等屏幕导航规则。

use bevy::prelude::*;

use crate::client::input::InterfaceCommand;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::state::LocalInventory;
use crate::shared::states::InputContextState;

/// 可由界面导航栈管理的顶层屏幕类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiScreen {
    MainMenu,
    Loading,
    PauseMenu,
    Settings,
    Inventory,
    Container,
    Modal,
}

/// 按打开顺序保存当前界面层级的导航栈。
#[derive(Resource, Debug, Default, Clone)]
pub struct UiScreenStack {
    screens: Vec<UiScreen>,
}

impl UiScreenStack {
    /// 返回当前最上层屏幕。
    pub fn top(&self) -> Option<UiScreen> {
        self.screens.last().copied()
    }

    /// 判断指定屏幕是否已在导航栈中。
    pub fn contains(&self, screen: UiScreen) -> bool {
        self.screens.contains(&screen)
    }

    /// 把指定屏幕移到栈顶，避免同一屏幕重复入栈。
    pub fn open(&mut self, screen: UiScreen) {
        self.close(screen);
        self.screens.push(screen);
    }

    /// 关闭当前栈顶并打开指定屏幕。
    pub fn replace(&mut self, screen: UiScreen) {
        self.screens.pop();
        self.open(screen);
    }

    /// 关闭并返回当前栈顶屏幕。
    pub fn back(&mut self) -> Option<UiScreen> {
        self.screens.pop()
    }

    /// 移除指定屏幕，并返回栈是否发生变化。
    pub fn close(&mut self, screen: UiScreen) -> bool {
        let old_len = self.screens.len();
        self.screens.retain(|entry| *entry != screen);
        old_len != self.screens.len()
    }

    /// 清空全部屏幕层级。
    pub fn clear(&mut self) {
        self.screens.clear();
    }

    /// 按从底到顶的顺序遍历当前屏幕。
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = UiScreen> + '_ {
        self.screens.iter().copied()
    }
}

/// 请求修改客户端界面导航栈的消息。
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiNavigation {
    Open(UiScreen),
    Replace(UiScreen),
    Back,
    Close(UiScreen),
    Reset(UiScreen),
}

/// 限制屏幕只对指定游戏模式可见的受众条件。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiScreenAudience {
    #[default]
    Any,
    Creative,
    Survival,
}

/// 声明界面根节点对应的屏幕类别和受众条件。
#[derive(Component, Debug, Clone, Copy)]
pub struct UiScreenRoot {
    pub screen: UiScreen,
    pub audience: UiScreenAudience,
}

impl UiScreenRoot {
    /// 创建对所有游戏模式可见的屏幕根标记。
    pub const fn new(screen: UiScreen) -> Self {
        Self {
            screen,
            audience: UiScreenAudience::Any,
        }
    }

    /// 创建只对指定游戏模式可见的背包屏幕根标记。
    pub const fn inventory(audience: UiScreenAudience) -> Self {
        Self {
            screen: UiScreen::Inventory,
            audience,
        }
    }
}

/// 消费导航消息，并同步界面栈与输入上下文命令。
pub fn handle_ui_navigation_system(
    mut reader: MessageReader<UiNavigation>,
    mut stack: ResMut<UiScreenStack>,
    mut interface: MessageWriter<InterfaceCommand>,
) {
    for command in reader.read().copied() {
        match command {
            UiNavigation::Open(screen) => open_screen(screen, &mut stack, &mut interface),
            UiNavigation::Replace(screen) => {
                if let Some(closed) = stack.back() {
                    close_screen_and_parent(closed, &mut stack, &mut interface);
                }
                open_screen(screen, &mut stack, &mut interface);
            }
            UiNavigation::Back => {
                if let Some(closed) = stack.back() {
                    close_screen_and_parent(closed, &mut stack, &mut interface);
                } else {
                    open_screen(UiScreen::PauseMenu, &mut stack, &mut interface);
                }
            }
            UiNavigation::Close(screen) => {
                if stack.close(screen) {
                    close_screen_and_parent(screen, &mut stack, &mut interface);
                }
            }
            UiNavigation::Reset(screen) => {
                for closed in stack.iter().collect::<Vec<_>>() {
                    close_screen_and_parent(closed, &mut stack, &mut interface);
                }
                stack.clear();
                open_screen(screen, &mut stack, &mut interface);
            }
        }
    }
}

fn close_screen_and_parent(
    screen: UiScreen,
    _stack: &mut UiScreenStack,
    interface: &mut MessageWriter<InterfaceCommand>,
) {
    close_screen_state(screen, interface);
}

fn open_screen(
    screen: UiScreen,
    stack: &mut UiScreenStack,
    interface: &mut MessageWriter<InterfaceCommand>,
) {
    match screen {
        UiScreen::Inventory => {
            interface.write(InterfaceCommand::OpenInventory);
        }
        UiScreen::Container => {
            interface.write(InterfaceCommand::OpenInventory);
        }
        UiScreen::MainMenu | UiScreen::PauseMenu | UiScreen::Settings => {
            interface.write(InterfaceCommand::OpenMenu);
        }
        UiScreen::Loading => {}
        UiScreen::Modal => return stack.open(screen),
    };
    stack.open(screen);
}

fn close_screen_state(screen: UiScreen, interface: &mut MessageWriter<InterfaceCommand>) {
    match screen {
        UiScreen::Inventory => {
            interface.write(InterfaceCommand::CloseInventory);
        }
        UiScreen::Container => {
            interface.write(InterfaceCommand::CloseInventory);
        }
        UiScreen::MainMenu | UiScreen::PauseMenu | UiScreen::Settings => {
            interface.write(InterfaceCommand::CloseMenu);
        }
        UiScreen::Loading => {}
        UiScreen::Modal => {}
    }
}

/// 在旧输入状态仍存在期间把菜单和背包状态桥接到导航栈。
pub fn sync_legacy_screen_state_system(
    mut navigation: MessageReader<UiNavigation>,
    inventory: LocalInventory,
    context: Res<InputContextState>,
    mut stack: ResMut<UiScreenStack>,
) {
    if navigation.read().next().is_some() {
        return;
    }
    if inventory.opened {
        if !stack.contains(UiScreen::Inventory) && !stack.contains(UiScreen::Container) {
            stack.open(UiScreen::Inventory);
        }
    } else {
        stack.close(UiScreen::Container);
        stack.close(UiScreen::Inventory);
    }

    if context.menu_open() {
        if !stack.contains(UiScreen::PauseMenu) && !stack.contains(UiScreen::MainMenu) {
            stack.open(UiScreen::PauseMenu);
        }
    } else {
        stack.close(UiScreen::PauseMenu);
    }
}

/// 根据导航栈顶层级和游戏模式同步各屏幕根节点可见性。
pub fn sync_screen_visibility_system(
    stack: Res<UiScreenStack>,
    gamemode: Res<PlayerGameMode>,
    mut query: Query<(&UiScreenRoot, &mut Visibility)>,
) {
    if !stack.is_changed() && !gamemode.is_changed() {
        return;
    }
    for (root, mut visibility) in &mut query {
        let audience_matches = match root.audience {
            UiScreenAudience::Any => true,
            UiScreenAudience::Creative => gamemode.is_creative(),
            UiScreenAudience::Survival => gamemode.is_survival(),
        };
        *visibility = if stack.contains(root.screen) && audience_matches {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/client/ui/navigation.rs"]
mod tests;
