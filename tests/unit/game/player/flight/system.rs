//! 飞行切换与垂直速度规则的白盒单元测试。

use crate::game::gameplay::gamemode::{GameMode, PlayerGameMode};
use crate::game::player::control::action::{PlayerAction, PlayerActionState};
use crate::game::player::flight::components::{PlayerFlight, ToggleFlightRequest};
use crate::game::player::flight::system::{
    cleanup_flight_if_not_permitted_system, flight_permitted, flight_vertical_velocity,
    toggle_flight_system,
};
use crate::game::player::identity::Player;
use crate::game::player::physics::components::PlayerGravity;
use bevy::ecs::message::Messages;
use bevy::prelude::*;

/// 判定仅创造模式允许飞行。
#[test]
fn flight_permitted_only_in_creative() {
    assert!(flight_permitted(&PlayerGameMode {
        mode: GameMode::Creative
    }));
    assert!(!flight_permitted(&PlayerGameMode {
        mode: GameMode::Survival
    }));
}

/// 垂直速度按输入方向取正负，无输入时为零。
#[test]
fn vertical_velocity_follows_jump_and_squat() {
    let flight = PlayerFlight::default();
    let mut actions = PlayerActionState::default();

    actions.update(true, [PlayerAction::Jump]);
    assert_eq!(
        flight_vertical_velocity(&actions, &flight),
        flight.fly_speed
    );

    actions.update(true, [PlayerAction::Squat]);
    assert_eq!(
        flight_vertical_velocity(&actions, &flight),
        -flight.fly_speed
    );

    actions.update(true, []);
    assert_eq!(flight_vertical_velocity(&actions, &flight), 0.0);
}

/// 构造带单个玩家实体的测试 App，并注册切换系统。
fn toggle_app(gamemode: GameMode) -> (App, Entity) {
    let mut app = App::new();
    app.add_message::<ToggleFlightRequest>()
        .insert_resource(PlayerGameMode { mode: gamemode })
        .add_systems(Update, toggle_flight_system);
    let player = app
        .world_mut()
        .spawn((Player, PlayerFlight::default(), PlayerGravity::default()))
        .id();
    (app, player)
}

/// 向测试 App 发送一次切换请求并推进一帧。
fn send_toggle(app: &mut App) {
    app.world_mut()
        .resource_mut::<Messages<ToggleFlightRequest>>()
        .write(ToggleFlightRequest);
    app.update();
}

/// 生存模式下切换请求被忽略，飞行保持关闭。
#[test]
fn toggle_flight_ignored_in_survival() {
    let (mut app, player) = toggle_app(GameMode::Survival);
    send_toggle(&mut app);
    assert!(!app.world_mut().get::<PlayerFlight>(player).unwrap().enabled);
}

/// 创造模式下切换请求开启飞行。
#[test]
fn toggle_flight_enables_in_creative() {
    let (mut app, player) = toggle_app(GameMode::Creative);
    send_toggle(&mut app);
    assert!(app.world_mut().get::<PlayerFlight>(player).unwrap().enabled);
}

/// 连续两次请求回到初始关闭状态。
#[test]
fn toggle_flight_twice_returns_to_original() {
    let (mut app, player) = toggle_app(GameMode::Creative);
    send_toggle(&mut app);
    send_toggle(&mut app);
    assert!(!app.world_mut().get::<PlayerFlight>(player).unwrap().enabled);
}

/// 关闭飞行时清零垂直速度，避免遗留上升速度瞬间弹飞。
#[test]
fn toggle_off_clears_vertical_velocity() {
    let mut app = App::new();
    app.add_message::<ToggleFlightRequest>()
        .insert_resource(PlayerGameMode {
            mode: GameMode::Creative,
        })
        .add_systems(Update, toggle_flight_system);
    let player = app
        .world_mut()
        .spawn((
            Player,
            PlayerFlight {
                enabled: true,
                fly_speed: 12.0,
            },
            PlayerGravity {
                velocity_y: 8.0,
                ..Default::default()
            },
        ))
        .id();
    send_toggle(&mut app);
    let enabled = app.world_mut().get::<PlayerFlight>(player).unwrap().enabled;
    let velocity_y = app
        .world_mut()
        .get::<PlayerGravity>(player)
        .unwrap()
        .velocity_y;
    assert!(!enabled);
    assert_eq!(velocity_y, 0.0);
}

/// 模式不再允许飞行时，兜底系统强制落地并清零垂直速度。
#[test]
fn cleanup_lands_when_not_permitted() {
    let mut app = App::new();
    app.insert_resource(PlayerGameMode {
        mode: GameMode::Survival,
    })
    .add_systems(Update, cleanup_flight_if_not_permitted_system);
    let player = app
        .world_mut()
        .spawn((
            Player,
            PlayerFlight {
                enabled: true,
                fly_speed: 12.0,
            },
            PlayerGravity {
                velocity_y: 5.0,
                ..Default::default()
            },
        ))
        .id();
    app.update();
    let enabled = app.world_mut().get::<PlayerFlight>(player).unwrap().enabled;
    let velocity_y = app
        .world_mut()
        .get::<PlayerGravity>(player)
        .unwrap()
        .velocity_y;
    assert!(!enabled);
    assert_eq!(velocity_y, 0.0);
}
