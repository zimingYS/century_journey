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
mod tests {
    use super::*;
    use crate::content::block::registry::BlockRegistry;
    use crate::game::gameplay::gamemode::PlayerGameMode;
    use crate::game::inventory::item::stack::ItemStack;
    use crate::game::player::action::{PlayerAction, PlayerActionState};
    use crate::game::player::command::{
        PlayerCommand, PlayerCommandBuffer, apply_player_command_system,
    };
    use crate::game::player::components::stats::Hunger;
    use crate::game::player::components::stats::{Defense, Health};
    use crate::game::player::components::{
        LocalPlayer, Player, PlayerAim, PlayerCollider, PlayerGravity, PlayerLifecycle,
        PlayerMovement, PlayerVelocity,
    };
    use crate::game::player::events::{AttackEvent, DamageEvent, DeathEvent};
    use crate::game::world::entity::dropped_item::{DroppedItem, DroppedItemVelocity};
    use crate::game::world::storage::WorldStorage;
    use crate::game::world::time::{
        GameDayElapsed, GameHourElapsed, GameMinuteElapsed, GameYearElapsed, SeasonChanged,
        SolarTermChanged, WorldSimulationClock, advance_world_simulation_clock,
    };
    use crate::shared::item_id::ItemId;
    use crate::shared::random::RandomSource;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    #[test]
    fn event_streams_are_reproducible_and_context_scoped() {
        let rng = SimulationRng::new(99);
        let mut first = rng.for_event(LOOT_RANDOM_DOMAIN, 12, 34);
        let mut second = rng.for_event(LOOT_RANDOM_DOMAIN, 12, 34);
        let mut later = rng.for_event(LOOT_RANDOM_DOMAIN, 13, 34);

        assert_eq!(first.next_u64(), second.next_u64());
        assert_ne!(first.next_u64(), later.next_u64());
    }

    #[derive(Resource, Default)]
    struct ScriptedInput(PlayerActionState);

    #[derive(Resource, Default)]
    struct RandomProbe(u64);

    fn collect_scripted_command(
        clock: Res<WorldSimulationClock>,
        mut input: ResMut<ScriptedInput>,
        mut buffer: ResMut<PlayerCommandBuffer>,
    ) {
        let tick = clock.simulation_tick().saturating_add(1);
        let mut active = Vec::new();
        if tick <= 180 {
            active.push(PlayerAction::MoveForward);
        }
        if (40..=100).contains(&tick) {
            active.push(PlayerAction::Sprint);
        }
        if tick == 10 {
            active.push(PlayerAction::Jump);
        }
        if tick == 1 {
            active.push(PlayerAction::Attack);
        }
        if (120..=220).contains(&tick) {
            active.push(PlayerAction::MoveRight);
        }
        input.0.update(true, active);
        let yaw = if tick < 90 { 0.0 } else { 0.65 };
        buffer.enqueue(PlayerCommand::from_action_state(tick, &input.0, yaw, -0.15));
    }

    fn sample_deterministic_random_stream(
        clock: Res<WorldSimulationClock>,
        random: Res<SimulationRng>,
        mut probe: ResMut<RandomProbe>,
    ) {
        let tick = clock.simulation_tick();
        let mut stream = random.for_event(0x5445_5354, tick, tick.rotate_left(17));
        probe.0 = hash_word(probe.0, stream.next_u64());
    }

    fn simulate_at_render_rate(fps: u32) -> u64 {
        const TARGET_TICKS: u64 = 240;
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(SimulationPlugin)
            .insert_resource(Time::<Fixed>::from_hz(20.0))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1.0 / fps as f64,
            )))
            .init_resource::<WorldSimulationClock>()
            .init_resource::<PlayerActionState>()
            .init_resource::<PlayerCommandBuffer>()
            .init_resource::<ScriptedInput>()
            .init_resource::<RandomProbe>()
            .init_resource::<WorldStorage>()
            .init_resource::<BlockRegistry>()
            .init_resource::<PlayerGameMode>()
            .add_message::<AttackEvent>()
            .add_message::<DamageEvent>()
            .add_message::<DeathEvent>()
            .add_message::<GameMinuteElapsed>()
            .add_message::<GameHourElapsed>()
            .add_message::<GameDayElapsed>()
            .add_message::<SolarTermChanged>()
            .add_message::<SeasonChanged>()
            .add_message::<GameYearElapsed>()
            .add_systems(PreUpdate, collect_scripted_command)
            .add_systems(
                FixedUpdate,
                advance_world_simulation_clock.in_set(SimulationSet::Clock),
            )
            .add_systems(
                FixedUpdate,
                apply_player_command_system.in_set(SimulationSet::Commands),
            )
            .add_systems(
                FixedUpdate,
                crate::game::player::systems::movement::player_movement_system
                    .in_set(SimulationSet::Movement),
            )
            .add_systems(
                FixedUpdate,
                crate::game::player::systems::gravity::player_gravity_system
                    .in_set(SimulationSet::Physics),
            )
            .add_systems(
                FixedUpdate,
                crate::game::player::systems::hunger::action_cost_system
                    .in_set(SimulationSet::Survival),
            )
            .add_systems(
                FixedUpdate,
                (
                    crate::game::player::systems::combat::melee_attack_input_system,
                    crate::game::player::systems::combat::attack_damage_system,
                    crate::game::player::systems::combat::damage_system,
                )
                    .chain()
                    .in_set(SimulationSet::Combat),
            )
            .add_systems(
                FixedUpdate,
                (
                    crate::game::world::entity::dropped_item::dropped_item_physics_system,
                    sample_deterministic_random_stream,
                )
                    .chain()
                    .in_set(SimulationSet::Entities),
            );

        app.world_mut().spawn((
            Player,
            LocalPlayer,
            PlayerAim::default(),
            PlayerCollider::default(),
            PlayerMovement::default(),
            PlayerVelocity::default(),
            PlayerGravity::default(),
            PlayerLifecycle::default(),
            Hunger::default(),
            Transform::from_xyz(0.0, 70.0, 0.0),
            SimulationTransformHistory::new(Transform::from_xyz(0.0, 70.0, 0.0)),
        ));
        app.world_mut().spawn((
            Player,
            PlayerLifecycle::default(),
            Health::default(),
            Defense::default(),
            Transform::from_xyz(0.0, 70.0, -2.0),
        ));
        app.world_mut().spawn((
            DroppedItem::new(ItemStack::single(ItemId::item("test:drop"))),
            DroppedItemVelocity::thrown(Vec3::X),
            Transform::from_xyz(0.0, 80.0, 0.0),
            SimulationTransformHistory::new(Transform::from_xyz(0.0, 80.0, 0.0)),
        ));

        while app
            .world()
            .resource::<WorldSimulationClock>()
            .simulation_tick()
            < TARGET_TICKS
        {
            app.update();
        }

        let world = app.world_mut();
        let (
            player_translation,
            player_rotation,
            player_velocity,
            gravity_velocity,
            fall_distance,
            grounded,
            hunger_current,
            hunger_saturation,
            aim_pitch,
        ) = {
            let mut query = world.query::<(
                &Transform,
                &PlayerVelocity,
                &PlayerGravity,
                &Hunger,
                &PlayerAim,
            )>();
            let (transform, velocity, gravity, hunger, aim) = query.single(world).unwrap();
            (
                transform.translation,
                transform.rotation,
                velocity.horizontal,
                gravity.velocity_y,
                gravity.fall_distance,
                gravity.is_grounded,
                hunger.current,
                hunger.saturation,
                aim.pitch,
            )
        };
        let target_health = {
            let mut query = world.query_filtered::<&Health, Without<LocalPlayer>>();
            query.single(world).unwrap().current
        };
        let (drop_translation, drop_rotation, drop_linear, drop_angular, drop_age) = {
            let mut query = world.query::<(&Transform, &DroppedItemVelocity, &DroppedItem)>();
            let (transform, velocity, dropped) = query.single(world).unwrap();
            (
                transform.translation,
                transform.rotation,
                velocity.linear,
                velocity.angular,
                dropped.age,
            )
        };
        let mut hash = 0xcbf2_9ce4_8422_2325;
        hash = hash_word(
            hash,
            world.resource::<WorldSimulationClock>().simulation_tick(),
        );
        for value in player_translation.to_array() {
            hash = hash_word(hash, value.to_bits() as u64);
        }
        for value in player_rotation.to_array() {
            hash = hash_word(hash, value.to_bits() as u64);
        }
        for value in player_velocity.to_array() {
            hash = hash_word(hash, value.to_bits() as u64);
        }
        hash = hash_word(hash, gravity_velocity.to_bits() as u64);
        hash = hash_word(hash, fall_distance.to_bits() as u64);
        hash = hash_word(hash, grounded as u64);
        hash = hash_word(hash, hunger_current.to_bits() as u64);
        hash = hash_word(hash, hunger_saturation.to_bits() as u64);
        hash = hash_word(hash, aim_pitch.to_bits() as u64);
        hash = hash_word(hash, target_health.to_bits() as u64);
        for value in drop_translation.to_array() {
            hash = hash_word(hash, value.to_bits() as u64);
        }
        for value in drop_rotation.to_array() {
            hash = hash_word(hash, value.to_bits() as u64);
        }
        for value in drop_linear.to_array() {
            hash = hash_word(hash, value.to_bits() as u64);
        }
        for value in drop_angular.to_array() {
            hash = hash_word(hash, value.to_bits() as u64);
        }
        hash = hash_word(hash, drop_age.to_bits() as u64);
        hash_word(hash, world.resource::<RandomProbe>().0)
    }

    fn hash_word(hash: u64, word: u64) -> u64 {
        (hash ^ word).wrapping_mul(0x0000_0100_0000_01B3)
    }

    #[test]
    fn identical_command_stream_has_the_same_state_hash_at_30_60_and_144_fps() {
        let at_30 = simulate_at_render_rate(30);
        let at_60 = simulate_at_render_rate(60);
        let at_144 = simulate_at_render_rate(144);

        assert_eq!(at_30, at_60);
        assert_eq!(at_60, at_144);
    }

    #[test]
    fn interpolation_snaps_large_teleports_instead_of_crossing_the_world() {
        let mut app = App::new();
        app.add_systems(Update, capture_simulation_transforms);
        let start = Transform::from_xyz(0.0, 70.0, 0.0);
        let destination = Transform::from_xyz(100.0, 80.0, -100.0);
        let entity = app
            .world_mut()
            .spawn((destination, SimulationTransformHistory::new(start)))
            .id();

        app.update();

        let history = app
            .world()
            .entity(entity)
            .get::<SimulationTransformHistory>()
            .unwrap();
        assert_eq!(
            history.interpolated(0.25).translation,
            destination.translation
        );
    }
}
