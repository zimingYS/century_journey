use super::*;
use crate::game::world::entity::dropped_item::DroppedItem;
use crate::shared::item_id::ItemId;

#[derive(Resource, Default)]
struct AttackEventCount(usize);

#[derive(Resource, Default)]
struct DeathEventCount(usize);

fn count_attack_events(
    mut reader: MessageReader<AttackEvent>,
    mut count: ResMut<AttackEventCount>,
) {
    count.0 += reader.read().count();
}

fn count_death_events(mut reader: MessageReader<DeathEvent>, mut count: ResMut<DeathEventCount>) {
    count.0 += reader.read().count();
}

#[test]
fn feedback_fix_empty_attack_does_not_emit_a_hit_event() {
    let mut app = App::new();
    app.init_resource::<PlayerActionState>()
        .init_resource::<AttackEventCount>()
        .add_message::<AttackEvent>()
        .add_systems(
            Update,
            (melee_attack_input_system, count_attack_events).chain(),
        );
    app.world_mut().spawn((
        Player,
        LocalPlayer,
        Transform::default(),
        PlayerLifecycle::default(),
    ));
    app.world_mut()
        .resource_mut::<PlayerActionState>()
        .update(true, [PlayerAction::Attack]);

    app.update();

    assert_eq!(app.world().resource::<AttackEventCount>().0, 0);
}

#[test]
fn exact_lethal_damage_emits_one_death_and_invalid_damage_is_ignored() {
    let mut app = App::new();
    app.init_resource::<DeathEventCount>()
        .add_message::<DamageEvent>()
        .add_message::<DeathEvent>()
        .add_systems(Update, (damage_system, count_death_events).chain());
    let player = app
        .world_mut()
        .spawn((
            Player,
            Health {
                current: 2.0,
                max: 20.0,
            },
            PlayerLifecycle::default(),
        ))
        .id();

    app.world_mut().write_message(DamageEvent {
        target: player,
        amount: f32::NAN,
        source: DamageSource::Generic,
    });
    app.world_mut().write_message(DamageEvent {
        target: player,
        amount: 2.0,
        source: DamageSource::Generic,
    });
    app.world_mut().write_message(DamageEvent {
        target: player,
        amount: 2.0,
        source: DamageSource::Generic,
    });
    app.update();

    assert_eq!(app.world().get::<Health>(player).unwrap().current, 0.0);
    assert_eq!(
        app.world().get::<PlayerLifecycle>(player).unwrap().state,
        PlayerLifeState::Dead
    );
    assert_eq!(app.world().resource::<DeathEventCount>().0, 1);
}

#[test]
fn stage_seven_damage_death_drop_and_respawn_form_a_state_machine() {
    let mut inventory = InventoryState::default();
    inventory
        .hotbar
        .set_stack(0, ItemStack::new(ItemId::item("century_journey:apple"), 2));
    inventory.survival.backpack[0] = Some(ItemStack::single(ItemId::block("century_journey:dirt")));

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayerGameMode>()
        .init_resource::<DeathRules>()
        .init_resource::<LastDeathInfo>()
        .add_message::<DamageEvent>()
        .add_message::<DeathEvent>()
        .add_message::<RespawnRequest>()
        .add_systems(
            Update,
            (
                damage_system,
                death_system,
                respawn_request_system,
                respawn_transition_system,
            )
                .chain(),
        );
    let respawn_point = Vec3::new(12.0, 75.0, -4.0);
    let player = app
        .world_mut()
        .spawn((
            Player,
            Transform::from_xyz(3.0, 60.0, 5.0),
            Health {
                current: 2.0,
                max: 20.0,
            },
            Hunger::default(),
            PlayerLifecycle::default(),
            RespawnPoint(respawn_point),
            PlayerVelocity::default(),
            PlayerGravity::default(),
            EnvironmentExposure::default(),
            FoodUseState::default(),
            inventory,
        ))
        .id();

    app.world_mut().write_message(DamageEvent {
        target: player,
        amount: 20.0,
        source: DamageSource::Generic,
    });
    app.update();

    assert_eq!(
        app.world().get::<PlayerLifecycle>(player).unwrap().state,
        PlayerLifeState::Dead
    );
    assert!(
        app.world()
            .get::<InventoryState>(player)
            .unwrap()
            .hotbar
            .get_stack(0)
            .is_none()
    );
    assert_eq!(
        app.world()
            .iter_entities()
            .filter(|entity| entity.contains::<DroppedItem>())
            .count(),
        2
    );

    app.world_mut()
        .write_message(RespawnRequest { entity: player });
    app.update();
    assert_eq!(
        app.world().get::<PlayerLifecycle>(player).unwrap().state,
        PlayerLifeState::Respawning
    );
    assert_eq!(
        app.world().get::<Transform>(player).unwrap().translation,
        respawn_point
    );
    assert_eq!(app.world().get::<Health>(player).unwrap().current, 20.0);
    assert_eq!(app.world().get::<Hunger>(player).unwrap().current, 20.0);
    assert!(!app.world().get::<FoodUseState>(player).unwrap().is_active());

    app.world_mut()
        .get_mut::<PlayerLifecycle>(player)
        .unwrap()
        .respawn_remaining = 0.0;
    app.update();
    assert_eq!(
        app.world().get::<PlayerLifecycle>(player).unwrap().state,
        PlayerLifeState::Alive
    );
}
