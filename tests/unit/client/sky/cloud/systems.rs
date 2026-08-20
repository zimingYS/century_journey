use super::*;

#[test]
fn weather_state_normalizes_external_inputs() {
    let state = CloudWeatherState {
        coverage: 2.0,
        wind_multiplier: -1.0,
        visibility: f32::NAN,
    }
    .normalized();

    assert_eq!(state.coverage, 1.0);
    assert_eq!(state.wind_multiplier, 0.0);
    assert_eq!(state.visibility, 1.0);
}
