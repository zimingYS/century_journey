use super::*;

#[test]
fn render_frame_taps_are_preserved_until_the_target_tick() {
    let mut client = PlayerActionState::default();
    let mut buffer = PlayerCommandBuffer::default();

    client.update(true, [PlayerAction::Jump]);
    buffer.enqueue(PlayerCommand::from_action_state(7, &client, 0.0, 0.0));
    client.update(true, []);
    buffer.enqueue(PlayerCommand::from_action_state(7, &client, 0.0, 0.0));

    let command = buffer.take_for_tick(7);
    let mut simulation = PlayerActionState::default();
    simulation.apply_command(&command);

    assert!(simulation.just_pressed(PlayerAction::Jump));
    assert!(simulation.just_released(PlayerAction::Jump));
    assert!(!simulation.pressed(PlayerAction::Jump));
}

#[test]
fn held_actions_continue_when_no_new_render_command_arrives() {
    let mut client = PlayerActionState::default();
    client.update(true, [PlayerAction::MoveForward]);
    let mut buffer = PlayerCommandBuffer::default();
    buffer.enqueue(PlayerCommand::from_action_state(2, &client, 0.0, 0.0));

    let first = buffer.take_for_tick(2);
    let next = buffer.take_for_tick(3);
    let mut simulation = PlayerActionState::default();
    simulation.apply_command(&first);
    assert!(simulation.just_pressed(PlayerAction::MoveForward));
    simulation.apply_command(&next);
    assert!(simulation.pressed(PlayerAction::MoveForward));
    assert!(!simulation.just_pressed(PlayerAction::MoveForward));
}

#[test]
fn directly_injected_commands_derive_tick_edges() {
    let mut buffer = PlayerCommandBuffer::default();
    buffer.enqueue(PlayerCommand::new(4, [PlayerAction::Attack], 0.25, -0.1));
    let pressed = buffer.take_for_tick(4);
    buffer.enqueue(PlayerCommand::new(5, [], 0.25, -0.1));
    let released = buffer.take_for_tick(5);

    let mut state = PlayerActionState::default();
    state.apply_command(&pressed);
    assert!(state.just_pressed(PlayerAction::Attack));
    state.apply_command(&released);
    assert!(state.just_released(PlayerAction::Attack));
}

#[test]
fn scaled_time_keeps_commands_aligned_with_fixed_steps() {
    // 回归：时间倍率只加速日历，命令目标刻仍须命中下一个固定步。
    let mut clock = WorldSimulationClock::default();
    let mut buffer = PlayerCommandBuffer::default();
    let target_tick = clock.simulation_tick() + 1;
    buffer.enqueue(PlayerCommand::new(
        target_tick,
        [PlayerAction::MoveForward],
        0.5,
        0.0,
    ));

    // 倍率 10 的一个固定步：日历推进 10 刻，模拟刻只加一。
    clock.advance_fixed_step(10);

    let command = buffer.take_for_tick(clock.simulation_tick());
    let mut simulation = PlayerActionState::default();
    simulation.apply_command(&command);
    assert!(simulation.pressed(PlayerAction::MoveForward));
    assert_eq!(command.yaw, 0.5);
}
