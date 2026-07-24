use super::*;
use crate::game::player::action::PlayerActionState;
use crate::game::player::components::PlayerGravity;

#[test]
fn feedback_fix_empty_right_click_does_not_start_an_animation() {
    assert_eq!(choose_behavior(AnimationSignals::default()), None);
}

#[test]
fn feedback_fix_consumed_food_starts_using_animation() {
    assert_eq!(
        choose_behavior(AnimationSignals {
            used: true,
            ..default()
        }),
        Some(PlayerBehaviorState::Using)
    );
}

#[test]
fn death_and_hurt_override_regular_actions() {
    let all_actions = AnimationSignals {
        died: true,
        hurt: true,
        mining: true,
        placed: true,
        used: true,
        attacked: true,
    };
    assert_eq!(
        choose_behavior(all_actions),
        Some(PlayerBehaviorState::Death)
    );
    assert!(PlayerBehaviorState::Hurt.priority() > PlayerBehaviorState::Attacking.priority());
}

#[test]
fn layer_transition_is_smooth_and_reaches_target() {
    let mut layer = AnimationLayer::new(PlayerLocomotionState::Idle, 1.0);
    layer.transition_to(PlayerLocomotionState::Run, 0.2, 1.0);
    layer.tick(0.1);
    assert_eq!(layer.previous, PlayerLocomotionState::Idle);
    assert_eq!(layer.current, PlayerLocomotionState::Run);
    assert!((layer.blend_factor() - 0.5).abs() < 0.001);
    layer.tick(0.1);
    assert_eq!(layer.previous, PlayerLocomotionState::Run);
    assert_eq!(layer.blend_factor(), 1.0);
}

#[test]
fn playback_emits_marker_once_per_cycle() {
    let mut playback = AnimationPlayback::default();
    playback.start(1.0);
    let before = playback.tick(0.4, 1.0, false, Some(0.5));
    let crossing = playback.tick(0.2, 1.0, false, Some(0.5));
    let after = playback.tick(0.2, 1.0, false, Some(0.5));
    assert!(!before.marker_crossed);
    assert!(crossing.marker_crossed);
    assert!(!after.marker_crossed);
}

#[test]
fn looping_marker_keeps_the_cycle_where_it_was_crossed() {
    let mut playback = AnimationPlayback::default();
    playback.start(1.0);
    let first = playback.tick(0.6, 1.0, true, Some(0.5));
    let wrap = playback.tick(0.5, 1.0, true, Some(0.5));
    let second = playback.tick(0.5, 1.0, true, Some(0.5));

    assert!(first.marker_crossed);
    assert_eq!(first.marker_cycle, 0);
    assert!(!wrap.marker_crossed);
    assert!(second.marker_crossed);
    assert_eq!(second.marker_cycle, 1);
}

#[test]
fn looping_marker_is_not_lost_when_a_frame_crosses_the_cycle_boundary() {
    let mut playback = AnimationPlayback::default();
    playback.start(1.0);

    let first = playback.tick(0.75, 1.0, true, Some(0.5));
    let long_frame = playback.tick(0.90, 1.0, true, Some(0.5));

    assert!(first.marker_crossed);
    assert_eq!(first.marker_cycle, 0);
    assert!(long_frame.marker_crossed);
    assert_eq!(long_frame.marker_cycle, 1);
}

#[test]
fn locomotion_phase_is_stable_across_render_rates() {
    fn simulate(fps: u32) -> PlayerAnimationState {
        let mut state = PlayerAnimationState::default();
        let gravity = PlayerGravity {
            is_grounded: true,
            ..default()
        };
        let config = PlayerAnimationConfig::default();
        let actions = PlayerActionState::default();
        let delta = 1.0 / fps as f32;

        for _ in 0..fps {
            update_motion_parameters(&mut state, 10.0, &gravity, true, delta, &config, &actions);
        }
        state
    }

    let at_10 = simulate(10);
    let at_20 = simulate(20);
    let at_30 = simulate(30);
    let at_60 = simulate(60);
    let at_144 = simulate(144);

    assert_eq!(at_10.lower_body.current, PlayerLocomotionState::Walk);
    assert_eq!(at_20.lower_body.current, PlayerLocomotionState::Walk);
    assert!((at_10.parameters.locomotion_phase - at_20.parameters.locomotion_phase).abs() < 0.001);
    assert!((at_20.parameters.locomotion_phase - at_30.parameters.locomotion_phase).abs() < 0.001);
    assert!((at_30.parameters.locomotion_phase - at_60.parameters.locomotion_phase).abs() < 0.001);
    assert!((at_60.parameters.locomotion_phase - at_144.parameters.locomotion_phase).abs() < 0.001);
}
