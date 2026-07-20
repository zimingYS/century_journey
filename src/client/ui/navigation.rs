use bevy::prelude::*;

use crate::client::input::InterfaceCommand;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::state::LocalInventory;
use crate::shared::states::InputContextState;

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

#[derive(Resource, Debug, Default, Clone)]
pub struct UiScreenStack {
    screens: Vec<UiScreen>,
}

impl UiScreenStack {
    pub fn top(&self) -> Option<UiScreen> {
        self.screens.last().copied()
    }

    pub fn contains(&self, screen: UiScreen) -> bool {
        self.screens.contains(&screen)
    }

    pub fn open(&mut self, screen: UiScreen) {
        self.close(screen);
        self.screens.push(screen);
    }

    pub fn replace(&mut self, screen: UiScreen) {
        self.screens.pop();
        self.open(screen);
    }

    pub fn back(&mut self) -> Option<UiScreen> {
        self.screens.pop()
    }

    pub fn close(&mut self, screen: UiScreen) -> bool {
        let old_len = self.screens.len();
        self.screens.retain(|entry| *entry != screen);
        old_len != self.screens.len()
    }

    pub fn clear(&mut self) {
        self.screens.clear();
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = UiScreen> + '_ {
        self.screens.iter().copied()
    }
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiNavigation {
    Open(UiScreen),
    Replace(UiScreen),
    Back,
    Close(UiScreen),
    Reset(UiScreen),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiScreenAudience {
    #[default]
    Any,
    Creative,
    Survival,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct UiScreenRoot {
    pub screen: UiScreen,
    pub audience: UiScreenAudience,
}

impl UiScreenRoot {
    pub const fn new(screen: UiScreen) -> Self {
        Self {
            screen,
            audience: UiScreenAudience::Any,
        }
    }

    pub const fn inventory(audience: UiScreenAudience) -> Self {
        Self {
            screen: UiScreen::Inventory,
            audience,
        }
    }
}

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
mod tests {
    use super::*;

    #[test]
    fn stack_open_is_unique_and_back_restores_previous_screen() {
        let mut stack = UiScreenStack::default();
        stack.open(UiScreen::Inventory);
        stack.open(UiScreen::Modal);
        stack.open(UiScreen::Modal);

        assert_eq!(
            stack.iter().collect::<Vec<_>>(),
            [UiScreen::Inventory, UiScreen::Modal]
        );
        assert_eq!(stack.back(), Some(UiScreen::Modal));
        assert_eq!(stack.top(), Some(UiScreen::Inventory));
    }

    #[test]
    fn replace_and_close_keep_stack_consistent() {
        let mut stack = UiScreenStack::default();
        stack.open(UiScreen::MainMenu);
        stack.replace(UiScreen::PauseMenu);
        assert_eq!(stack.top(), Some(UiScreen::PauseMenu));
        assert!(stack.close(UiScreen::PauseMenu));
        assert!(stack.top().is_none());
    }
}
