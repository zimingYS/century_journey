use super::*;
use crate::game::inventory::container::InventoryContainer;

impl LegacySaveItemStack {
    fn air() -> Self {
        Self {
            item: "century_journey:air".into(),
            count: 0,
        }
    }
}

impl LegacySaveItemStackV6 {
    fn air() -> Self {
        Self {
            item: "century_journey:air".into(),
            count: 0,
            durability: None,
        }
    }
}

#[test]
fn v3_layout_migrates_armor_accessories_and_overflow() {
    let mut legacy = LegacyPlayerSaveDataV3 {
        version: 3,
        position: [1.0, 2.0, 3.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        camera_pitch: 0.25,
        gamemode: "survival".into(),
        health: 18.0,
        hunger: 12.0,
        hotbar_active: 2,
        hotbar: std::array::from_fn(|_| LegacySaveItemStack::air()),
        backpack: std::array::from_fn(|_| LegacySaveItemStack::air()),
        armor: std::array::from_fn(|_| LegacySaveItemStack::air()),
        accessories: std::array::from_fn(|_| LegacySaveItemStack::air()),
    };
    legacy.armor[0] = LegacySaveItemStack {
        item: "century_journey:test_helmet".into(),
        count: 1,
    };
    legacy.accessories[0] = LegacySaveItemStack {
        item: "century_journey:test_ring".into(),
        count: 1,
    };
    legacy.backpack[27] = LegacySaveItemStack {
        item: "century_journey:legacy_overflow".into(),
        count: 3,
    };

    let migrated: PlayerSaveData = legacy.into();
    let inventory = migrated.restore_inventory();

    assert_eq!(
        inventory
            .survival
            .get_stack(
                crate::game::inventory::container::survival::SurvivalInventory::equipment_index(0)
            )
            .map(|stack| stack.count),
        Some(1)
    );
    assert_eq!(
        inventory
            .survival
            .get_stack(
                crate::game::inventory::container::survival::SurvivalInventory::accessory_index(0)
            )
            .map(|stack| stack.count),
        Some(1)
    );
    assert_eq!(
        inventory.survival.get_stack(0).map(|stack| stack.count),
        Some(3)
    );
}

#[test]
fn current_player_file_round_trip_keeps_game_version_and_stats() {
    let mut data = PlayerSaveData {
        health: 11.5,
        hunger: 6.25,
        ..PlayerSaveData::default()
    };
    data.hotbar[0] = SaveItemStack {
        runtime_id: None,
        item: "century_journey:test_item".into(),
        count: 9,
        durability: Some(17),
    };

    let decoded = decode_player_data(&encode_player_data(&data).unwrap()).unwrap();
    assert_eq!(decoded.version, SAVE_VERSION);
    assert_eq!(decoded.game_version, env!("CARGO_PKG_VERSION"));
    assert_eq!(decoded.health, 11.5);
    assert_eq!(decoded.hunger, 6.25);
    assert_eq!(decoded.hotbar[0].count, 9);
    assert_eq!(decoded.hotbar[0].durability, Some(17));
}

#[test]
fn v6_player_file_migrates_to_v7_identifier_mapping() {
    let mut legacy = LegacyPlayerSaveDataV6 {
        version: 6,
        game_version: "0.2.0".into(),
        position: [2.0, 71.0, 4.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        camera_pitch: 0.1,
        gamemode: "survival".into(),
        health: 17.0,
        hunger: 12.0,
        saturation: 4.0,
        respawn_point: [0.0, 70.0, 0.0],
        hotbar_active: 0,
        hotbar: std::array::from_fn(|_| LegacySaveItemStackV6::air()),
        backpack: std::array::from_fn(|_| LegacySaveItemStackV6::air()),
        equipment: std::array::from_fn(|_| LegacySaveItemStackV6::air()),
        accessories: vec![LegacySaveItemStackV6::air(); 6],
    };
    legacy.hotbar[0] = LegacySaveItemStackV6 {
        item: "century_journey:wooden_axe".into(),
        count: 1,
        durability: Some(31),
    };
    let serialized = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(&legacy)
        .unwrap();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&serialized).unwrap();
    let mut encoded = PLAYER_MAGIC.to_vec();
    encoded.extend(encoder.finish().unwrap());

    let decoded = decode_player_data(&encoded).unwrap();

    assert_eq!(decoded.version, SAVE_VERSION);
    assert!(decoded.item_id_map.is_empty());
    assert_eq!(decoded.hotbar[0].runtime_id, None);
    assert_eq!(decoded.hotbar[0].durability, Some(31));
}

#[test]
fn stage_seven_v5_player_file_migrates_to_v6_defaults() {
    let mut legacy = LegacyPlayerSaveDataV5 {
        version: 5,
        game_version: "0.2.0".into(),
        position: [4.0, 70.0, -3.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        camera_pitch: 0.2,
        gamemode: "survival".into(),
        health: 15.0,
        hunger: 11.0,
        hotbar_active: 0,
        hotbar: std::array::from_fn(|_| LegacySaveItemStack::air()),
        backpack: std::array::from_fn(|_| LegacySaveItemStack::air()),
        equipment: std::array::from_fn(|_| LegacySaveItemStack::air()),
        accessories: vec![LegacySaveItemStack::air(); 6],
    };
    legacy.hotbar[0] = LegacySaveItemStack {
        item: "century_journey:wooden_axe".into(),
        count: 1,
    };
    let serialized = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(&legacy)
        .unwrap();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&serialized).unwrap();
    let mut encoded = PLAYER_MAGIC.to_vec();
    encoded.extend(encoder.finish().unwrap());

    let decoded = decode_player_data(&encoded).unwrap();

    assert_eq!(decoded.version, SAVE_VERSION);
    assert_eq!(decoded.saturation, 5.0);
    assert_eq!(decoded.respawn_point, [0.0, 70.0, 0.0]);
    assert_eq!(decoded.hotbar[0].durability, None);
}

#[test]
fn v4_layout_has_an_explicit_migration() {
    let mut legacy = LegacyPlayerSaveDataV4 {
        version: 4,
        position: [1.0, 2.0, 3.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        camera_pitch: 0.1,
        gamemode: "survival".into(),
        health: 9.0,
        hunger: 8.0,
        hotbar_active: 0,
        hotbar: std::array::from_fn(|_| LegacySaveItemStack::air()),
        backpack: std::array::from_fn(|_| LegacySaveItemStack::air()),
        equipment: std::array::from_fn(|_| LegacySaveItemStack::air()),
        accessories: vec![LegacySaveItemStack::air(); 6],
    };
    legacy.equipment[6] = LegacySaveItemStack {
        item: "century_journey:test_backpack".into(),
        count: 1,
    };
    let serialized = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(&legacy)
        .unwrap();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&serialized).unwrap();
    let decoded = decode_player_data(&encoder.finish().unwrap()).unwrap();

    assert_eq!(decoded.version, SAVE_VERSION);
    assert_eq!(decoded.health, 9.0);
    assert_eq!(decoded.hunger, 8.0);
    assert_eq!(decoded.equipment[6].count, 1);
}
