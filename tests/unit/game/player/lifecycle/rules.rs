//! 验证死亡、物品掉落与重生之间的完整状态转换。

use super::*;
use crate::game::player::survival::events::DamageEvent;
use crate::game::player::survival::health::damage_system;
use crate::game::world::entity::dropped_item::DroppedItem;
use crate::shared::item_id::ItemId;
use bevy::prelude::{App, IntoScheduleConfigs, MinimalPlugins, Update};

#[test]
fn damage_death_drop_and_respawn_form_a_state_machine() {
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
