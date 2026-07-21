use super::*;

fn temp_path(name: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "century_journey_{name}_{}_{unique}.json",
        std::process::id(),
    ))
}

#[test]
fn json_settings_round_trip_keeps_every_setting() {
    let path = temp_path("settings_round_trip");
    let expected = GameSettings {
        render_distance: 12,
        master_volume: 0.35,
        mouse_sensitivity: 1.4,
        ui_scale: 1.25,
        fullscreen: true,
        vsync: false,
    };

    save_settings_to(&path, &expected).unwrap();
    let loaded = load_settings_from(&path).unwrap();
    assert_eq!(loaded, expected);

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(persistence::backup_path(&path));
}

#[test]
fn unsupported_settings_version_is_rejected() {
    let bytes = serde_json::to_vec(&SettingsFile {
        format_version: SETTINGS_FORMAT_VERSION + 1,
        game_version: "future".into(),
        settings: GameSettings::default(),
    })
    .unwrap();

    assert!(decode_settings(&bytes).is_err());
}
