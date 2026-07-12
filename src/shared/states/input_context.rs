use bevy::prelude::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputContext {
    #[default]
    Gameplay,
    Inventory,
    Menu,
    TextInput,
}

impl InputContext {
    pub const fn priority(self) -> u8 {
        match self {
            Self::Gameplay => 0,
            Self::Inventory => 1,
            Self::Menu => 2,
            Self::TextInput => 3,
        }
    }

    pub fn resolve(candidates: impl IntoIterator<Item = Self>) -> Self {
        candidates
            .into_iter()
            .max_by_key(|context| context.priority())
            .unwrap_or_default()
    }

    pub const fn allows_gameplay(self) -> bool {
        matches!(self, Self::Gameplay)
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct InputContextState {
    active: InputContext,
    menu_open: bool,
}

impl InputContextState {
    pub const fn active(&self) -> InputContext {
        self.active
    }

    pub const fn menu_open(&self) -> bool {
        self.menu_open
    }

    pub fn set_active(&mut self, active: InputContext) {
        self.active = active;
    }

    pub fn set_menu_open(&mut self, open: bool) {
        self.menu_open = open;
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputSet {
    Interface,
    ResolveContext,
    CollectActions,
    SyncPresentation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highest_priority_context_wins() {
        assert_eq!(
            InputContext::resolve([
                InputContext::Gameplay,
                InputContext::Inventory,
                InputContext::TextInput,
                InputContext::Menu,
            ]),
            InputContext::TextInput
        );
    }
}
