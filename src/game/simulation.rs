use crate::shared::random::DeterministicRng;
use bevy::prelude::*;

pub const LOOT_RANDOM_DOMAIN: u64 = 0x4C4F_4F54;
const INTERPOLATION_SNAP_DISTANCE: f32 = 4.0;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimulationSet {
    Clock,
    Commands,
    Movement,
    Physics,
    Targeting,
    Interaction,
    Survival,
    Combat,
    Entities,
}

/// Stores the two latest authoritative transforms for render interpolation.
///
/// The component never replaces the entity's simulation transform. Client presentation
/// entities consume these snapshots and apply the visual offset to child nodes instead.
#[derive(Component, Debug, Clone, Copy)]
pub struct SimulationTransformHistory {
    previous: Transform,
    current: Transform,
}

impl SimulationTransformHistory {
    pub const fn new(transform: Transform) -> Self {
        Self {
            previous: transform,
            current: transform,
        }
    }

    pub const fn current(&self) -> Transform {
        self.current
    }

    pub fn interpolated(&self, alpha: f32) -> Transform {
        let alpha = alpha.clamp(0.0, 1.0);
        Transform {
            translation: self
                .previous
                .translation
                .lerp(self.current.translation, alpha),
            rotation: self.previous.rotation.slerp(self.current.rotation, alpha),
            scale: self.previous.scale.lerp(self.current.scale, alpha),
        }
    }

    pub fn visual_transform(&self, authoritative: Transform, alpha: f32) -> Transform {
        if self
            .current
            .translation
            .distance_squared(authoritative.translation)
            > INTERPOLATION_SNAP_DISTANCE * INTERPOLATION_SNAP_DISTANCE
        {
            authoritative
        } else {
            self.interpolated(alpha)
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct SimulationRng {
    world_seed: u64,
}

impl Default for SimulationRng {
    fn default() -> Self {
        Self::new(12_345)
    }
}

impl SimulationRng {
    pub const fn new(world_seed: u64) -> Self {
        Self { world_seed }
    }

    pub fn set_world_seed(&mut self, world_seed: u64) {
        self.world_seed = world_seed;
    }

    pub fn for_event(&self, domain: u64, tick: u64, event_key: u64) -> DeterministicRng {
        DeterministicRng::new(mix64(
            self.world_seed ^ mix64(domain) ^ mix64(tick) ^ mix64(event_key),
        ))
    }

    pub fn block_event_key(position: IVec3, block_id: u16) -> u64 {
        mix64(position.x as u32 as u64)
            ^ mix64((position.y as u32 as u64).rotate_left(21))
            ^ mix64((position.z as u32 as u64).rotate_left(42))
            ^ block_id as u64
    }
}

fn mix64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

fn begin_simulation_transform_tick(
    mut query: Query<(&Transform, &mut SimulationTransformHistory)>,
) {
    let snap_distance_squared = INTERPOLATION_SNAP_DISTANCE * INTERPOLATION_SNAP_DISTANCE;
    for (transform, mut history) in &mut query {
        if history
            .current
            .translation
            .distance_squared(transform.translation)
            > snap_distance_squared
        {
            history.previous = *transform;
            history.current = *transform;
        } else {
            history.previous = history.current;
        }
    }
}

fn capture_simulation_transforms(mut query: Query<(&Transform, &mut SimulationTransformHistory)>) {
    let snap_distance_squared = INTERPOLATION_SNAP_DISTANCE * INTERPOLATION_SNAP_DISTANCE;
    for (transform, mut history) in &mut query {
        history.current = *transform;
        if history
            .previous
            .translation
            .distance_squared(history.current.translation)
            > snap_distance_squared
        {
            history.previous = history.current;
        }
    }
}

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationRng>()
            .configure_sets(
                FixedUpdate,
                (
                    SimulationSet::Clock,
                    SimulationSet::Commands,
                    SimulationSet::Movement,
                    SimulationSet::Physics,
                    SimulationSet::Targeting,
                    SimulationSet::Interaction,
                    SimulationSet::Survival,
                    SimulationSet::Combat,
                    SimulationSet::Entities,
                )
                    .chain(),
            )
            .add_systems(
                FixedUpdate,
                begin_simulation_transform_tick
                    .after(SimulationSet::Clock)
                    .before(SimulationSet::Commands),
            )
            .add_systems(
                FixedUpdate,
                capture_simulation_transforms.after(SimulationSet::Entities),
            );
    }
}

#[cfg(test)]
#[path = "../../tests/unit/game/simulation.rs"]
mod tests;
