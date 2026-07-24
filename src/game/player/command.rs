use std::collections::BTreeMap;

use crate::game::player::action::{PlayerAction, PlayerActionState};
use crate::game::player::components::{LocalPlayer, PlayerAim};
use crate::game::world::time::WorldSimulationClock;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerCommand {
    pub tick: u64,
    pub(crate) active: [bool; PlayerAction::COUNT],
    pub(crate) pressed: [bool; PlayerAction::COUNT],
    pub(crate) released: [bool; PlayerAction::COUNT],
    pub(crate) cancelled: [bool; PlayerAction::COUNT],
    pub yaw: f32,
    pub pitch: f32,
}

impl PlayerCommand {
    pub fn new(
        tick: u64,
        actions: impl IntoIterator<Item = PlayerAction>,
        yaw: f32,
        pitch: f32,
    ) -> Self {
        let mut active = [false; PlayerAction::COUNT];
        for action in actions {
            active[action.index()] = true;
        }
        Self::held(tick, active, yaw, pitch)
    }

    pub fn from_action_state(tick: u64, state: &PlayerActionState, yaw: f32, pitch: f32) -> Self {
        Self {
            tick,
            active: state.active_snapshot(),
            pressed: state.pressed_snapshot(),
            released: state.released_snapshot(),
            cancelled: state.cancelled_snapshot(),
            yaw,
            pitch,
        }
    }

    fn held(tick: u64, active: [bool; PlayerAction::COUNT], yaw: f32, pitch: f32) -> Self {
        Self {
            tick,
            active,
            pressed: [false; PlayerAction::COUNT],
            released: [false; PlayerAction::COUNT],
            cancelled: [false; PlayerAction::COUNT],
            yaw,
            pitch,
        }
    }

    fn merge(&mut self, newer: Self) {
        self.active = newer.active;
        for index in 0..PlayerAction::COUNT {
            self.pressed[index] |= newer.pressed[index];
            self.released[index] |= newer.released[index];
            self.cancelled[index] |= newer.cancelled[index];
        }
        self.yaw = newer.yaw;
        self.pitch = newer.pitch;
    }
}

#[derive(Resource, Debug, Clone)]
pub struct PlayerCommandBuffer {
    pending: BTreeMap<u64, PlayerCommand>,
    held: [bool; PlayerAction::COUNT],
    yaw: f32,
    pitch: f32,
}

impl Default for PlayerCommandBuffer {
    fn default() -> Self {
        Self {
            pending: BTreeMap::new(),
            held: [false; PlayerAction::COUNT],
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

impl PlayerCommandBuffer {
    pub fn enqueue(&mut self, command: PlayerCommand) {
        self.pending
            .entry(command.tick)
            .and_modify(|queued| queued.merge(command))
            .or_insert(command);
    }

    pub fn take_for_tick(&mut self, tick: u64) -> PlayerCommand {
        self.pending.retain(|pending_tick, _| *pending_tick >= tick);
        if let Some(mut command) = self.pending.remove(&tick) {
            for index in 0..PlayerAction::COUNT {
                command.pressed[index] |= command.active[index] && !self.held[index];
                command.released[index] |= !command.active[index] && self.held[index];
            }
            self.held = command.active;
            self.yaw = command.yaw;
            self.pitch = command.pitch;
            command
        } else {
            PlayerCommand::held(tick, self.held, self.yaw, self.pitch)
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

pub fn apply_player_command_system(
    clock: Res<WorldSimulationClock>,
    mut commands: ResMut<PlayerCommandBuffer>,
    mut actions: ResMut<PlayerActionState>,
    mut player_query: Query<(&mut Transform, &mut PlayerAim), With<LocalPlayer>>,
) {
    let command = commands.take_for_tick(clock.simulation_tick());
    actions.apply_command(&command);

    if let Ok((mut transform, mut aim)) = player_query.single_mut() {
        if command.yaw.is_finite() {
            transform.rotation = Quat::from_rotation_y(command.yaw);
        }
        if command.pitch.is_finite() {
            aim.pitch = command.pitch.clamp(-1.5, 1.5);
        }
    }
}

pub fn reset_player_command_pipeline_system(
    mut commands: ResMut<PlayerCommandBuffer>,
    mut actions: ResMut<PlayerActionState>,
) {
    commands.clear();
    actions.clear();
}

#[cfg(test)]
#[path = "../../../tests/unit/game/player/command.rs"]
mod tests;
