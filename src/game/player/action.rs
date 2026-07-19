use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PlayerAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    Sprint,
    Jump,
    BreakBlock,
    Attack,
    PlaceBlock,
    Use,
    DropItem,
    HotbarPrevious,
    HotbarNext,
    Hotbar1,
    Hotbar2,
    Hotbar3,
    Hotbar4,
    Hotbar5,
    Hotbar6,
    Hotbar7,
    Hotbar8,
    Hotbar9,
    TogglePerspective,
}

impl PlayerAction {
    pub const COUNT: usize = 23;

    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerActionPhase {
    Pressed,
    Held,
    Released,
    Cancelled,
}

#[derive(Resource, Debug, Clone)]
pub struct PlayerActionState {
    active: [bool; PlayerAction::COUNT],
    pressed: [bool; PlayerAction::COUNT],
    released: [bool; PlayerAction::COUNT],
    cancelled: [bool; PlayerAction::COUNT],
}

impl Default for PlayerActionState {
    fn default() -> Self {
        Self {
            active: [false; PlayerAction::COUNT],
            pressed: [false; PlayerAction::COUNT],
            released: [false; PlayerAction::COUNT],
            cancelled: [false; PlayerAction::COUNT],
        }
    }
}

impl PlayerActionState {
    pub fn update(&mut self, enabled: bool, actions: impl IntoIterator<Item = PlayerAction>) {
        let previous = self.active;
        let mut next = [false; PlayerAction::COUNT];
        if enabled {
            for action in actions {
                next[action.index()] = true;
            }
        }

        for index in 0..PlayerAction::COUNT {
            self.pressed[index] = enabled && next[index] && !previous[index];
            self.released[index] = enabled && !next[index] && previous[index];
            self.cancelled[index] = !enabled && previous[index];
        }
        self.active = next;
    }

    pub fn pressed(&self, action: PlayerAction) -> bool {
        self.active[action.index()]
    }

    pub fn just_pressed(&self, action: PlayerAction) -> bool {
        self.pressed[action.index()]
    }

    pub fn just_released(&self, action: PlayerAction) -> bool {
        self.released[action.index()]
    }

    pub fn cancelled(&self, action: PlayerAction) -> bool {
        self.cancelled[action.index()]
    }

    pub fn phase(&self, action: PlayerAction) -> Option<PlayerActionPhase> {
        if self.just_pressed(action) {
            Some(PlayerActionPhase::Pressed)
        } else if self.pressed(action) {
            Some(PlayerActionPhase::Held)
        } else if self.just_released(action) {
            Some(PlayerActionPhase::Released)
        } else if self.cancelled(action) {
            Some(PlayerActionPhase::Cancelled)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn active_snapshot(&self) -> [bool; PlayerAction::COUNT] {
        self.active
    }

    pub(crate) fn pressed_snapshot(&self) -> [bool; PlayerAction::COUNT] {
        self.pressed
    }

    pub(crate) fn released_snapshot(&self) -> [bool; PlayerAction::COUNT] {
        self.released
    }

    pub(crate) fn cancelled_snapshot(&self) -> [bool; PlayerAction::COUNT] {
        self.cancelled
    }

    pub(crate) fn apply_command(&mut self, command: &super::command::PlayerCommand) {
        self.active = command.active;
        self.pressed = command.pressed;
        self.released = command.released;
        self.cancelled = command.cancelled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_have_press_hold_release_lifecycle() {
        let mut state = PlayerActionState::default();
        state.update(true, [PlayerAction::Jump]);
        assert_eq!(
            state.phase(PlayerAction::Jump),
            Some(PlayerActionPhase::Pressed)
        );

        state.update(true, [PlayerAction::Jump]);
        assert_eq!(
            state.phase(PlayerAction::Jump),
            Some(PlayerActionPhase::Held)
        );

        state.update(true, []);
        assert_eq!(
            state.phase(PlayerAction::Jump),
            Some(PlayerActionPhase::Released)
        );
    }

    #[test]
    fn losing_gameplay_context_cancels_held_actions() {
        let mut state = PlayerActionState::default();
        state.update(true, [PlayerAction::BreakBlock]);
        state.update(false, []);

        assert!(state.cancelled(PlayerAction::BreakBlock));
        assert!(!state.pressed(PlayerAction::BreakBlock));
    }
}
