use super::*;

#[test]
fn initial_visual_time_is_daytime() {
    let time = TimeOfDay::default();

    assert_eq!(time.current_time, INITIAL_VISUAL_HOUR);
    assert_eq!(time.phase(), TimePhase::Day);
}

#[test]
fn twilight_factor_tracks_sunrise_and_sunset() {
    let sunrise = TimeOfDay { current_time: 6.0 };
    let sunset = TimeOfDay { current_time: 18.0 };

    assert_eq!(sunrise.phase(), TimePhase::Sunrise);
    assert_eq!(sunset.phase(), TimePhase::Sunset);
    assert!((sunrise.twilight_factor() - 0.5).abs() < f32::EPSILON);
    assert!((sunset.twilight_factor() - 0.5).abs() < f32::EPSILON);
}
