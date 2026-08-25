use super::*;

fn temp_path(name: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "century_journey_{name}_{}_{unique}.dat",
        std::process::id(),
    ))
}

#[test]
fn named_settings_round_trip_keeps_every_setting() {
    let path = temp_path("settings_round_trip");
    let expected = GameSettings {
        render_distance: 12,
        master_volume: 0.35,
        mouse_sensitivity: 1.4,
        ui_scale: 1.25,
        fullscreen: true,
        vsync: false,
        language: "en-US".to_string(),
    };

    save_settings_to(&path, &expected).unwrap();
    let loaded = load_settings_from(&path).unwrap();
    assert_eq!(loaded, expected);

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(persistence::backup_path(&path));
}

#[test]
fn named_settings_default_missing_fields_and_ignore_unknown_fields() {
    #[derive(Serialize)]
    struct EarlierSettings {
        render_distance: u32,
        master_volume: f32,
        retired_bloom_strength: f32,
    }

    #[derive(Serialize)]
    struct EarlierDocument {
        game_version: String,
        settings: EarlierSettings,
    }

    let bytes = document::encode_named(
        SETTINGS_MAGIC,
        SETTINGS_DOCUMENT_FORMAT,
        &EarlierDocument {
            game_version: "0.2.0".into(),
            settings: EarlierSettings {
                render_distance: 14,
                master_volume: 0.5,
                retired_bloom_strength: 2.0,
            },
        },
    )
    .unwrap();
    let loaded = decode_settings(&bytes).unwrap();

    assert_eq!(loaded.render_distance, 14);
    assert_eq!(loaded.master_volume, 0.5);
    assert_eq!(loaded.mouse_sensitivity, 1.0);
    assert_eq!(loaded.ui_scale, 1.0);
    assert!(!loaded.fullscreen);
    assert!(loaded.vsync);
}

#[test]
fn unknown_settings_document_format_is_rejected() {
    let file = SettingsDocument::default();
    let mut bytes =
        document::encode_named(SETTINGS_MAGIC, SETTINGS_DOCUMENT_FORMAT, &file).unwrap();
    bytes[4..8].copy_from_slice(&(SETTINGS_DOCUMENT_FORMAT + 1).to_le_bytes());

    assert!(decode_settings(&bytes).is_err());
}

#[test]
fn legacy_json_settings_are_still_readable() {
    let bytes = serde_json::to_vec(&LegacyJsonSettings {
        format_version: 1,
        game_version: "0.2.0".into(),
        settings: GameSettings {
            render_distance: 10,
            master_volume: 0.4,
            ..GameSettings::default()
        },
    })
    .unwrap();

    let loaded = decode_legacy_settings(&bytes).unwrap();
    assert_eq!(loaded.render_distance, 10);
    assert_eq!(loaded.master_volume, 0.4);
}

#[test]
fn unsupported_legacy_json_version_is_rejected() {
    let bytes = serde_json::to_vec(&LegacyJsonSettings {
        format_version: 2,
        game_version: "future".into(),
        settings: GameSettings::default(),
    })
    .unwrap();

    assert!(decode_legacy_settings(&bytes).is_err());
}
