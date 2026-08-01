use super::*;
use crate::game::inventory::container::InventoryContainer;
use crate::game::save::player::SaveItemStack;
use crate::game::save::world::chunk::region::RegionManager;
use bincode::Options;
use flate2::Compression;
use flate2::write::GzEncoder;
use serde::Serialize;
use std::io::Write;

#[test]
fn player_save_is_nested_inside_its_world_root() {
    assert_eq!(
        player_save_path("isolated_world"),
        RegionManager::save_root("isolated_world")
            .join("players")
            .join("singleplayer.dat")
    );
}

impl IdentifierCountStack {
    fn air() -> Self {
        Self {
            item: "century_journey:air".into(),
            count: 0,
        }
    }
}

impl DurabilityStack {
    fn air() -> Self {
        Self {
            item: "century_journey:air".into(),
            count: 0,
            durability: None,
        }
    }
}

impl RuntimeMappedStack {
    fn air() -> Self {
        Self {
            runtime_id: None,
            item: "century_journey:air".into(),
            count: 0,
            durability: None,
        }
    }
}

fn encode_legacy<T: Serialize>(value: &T, with_magic: bool) -> Vec<u8> {
    let serialized = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(value)
        .unwrap();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&serialized).unwrap();
    let compressed = encoder.finish().unwrap();
    if with_magic {
        let mut encoded = LEGACY_PLAYER_MAGIC.to_vec();
        encoded.extend(compressed);
        encoded
    } else {
        compressed
    }
}

#[test]
fn expanded_inventory_layout_migrates_equipment_accessories_and_overflow() {
    let mut legacy = ExpandedInventoryLayout {
        layout_marker: 3,
        position: [1.0, 2.0, 3.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        camera_pitch: 0.25,
        gamemode: "survival".into(),
        health: 18.0,
        hunger: 12.0,
        hotbar_active: 2,
        hotbar: std::array::from_fn(|_| IdentifierCountStack::air()),
        backpack: std::array::from_fn(|_| IdentifierCountStack::air()),
        armor: std::array::from_fn(|_| IdentifierCountStack::air()),
        accessories: std::array::from_fn(|_| IdentifierCountStack::air()),
    };
    legacy.armor[0] = IdentifierCountStack {
        item: "century_journey:test_helmet".into(),
        count: 1,
    };
    legacy.accessories[0] = IdentifierCountStack {
        item: "century_journey:test_ring".into(),
        count: 1,
    };
    legacy.backpack[27] = IdentifierCountStack {
        item: "century_journey:legacy_overflow".into(),
        count: 3,
    };

    let migrated = decode_player_data(&encode_legacy(&legacy, false)).unwrap();
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

    let encoded = encode_player_data(&data).unwrap();
    let decoded = decode_player_data(&encoded).unwrap();

    assert_eq!(&encoded[..4], &PLAYER_DOCUMENT_MAGIC);
    assert_eq!(
        u32::from_le_bytes(encoded[4..8].try_into().unwrap()),
        PLAYER_DOCUMENT_FORMAT
    );
    assert_eq!(decoded.game_version, env!("CARGO_PKG_VERSION"));
    assert_eq!(decoded.health, 11.5);
    assert_eq!(decoded.hunger, 6.25);
    assert_eq!(decoded.hotbar[0].count, 9);
    assert_eq!(decoded.hotbar[0].durability, Some(17));
}

#[derive(Serialize)]
struct SparsePlayerDocument {
    health: f32,
    food_level: f32,
    removed_field: String,
}

#[test]
fn named_fields_default_missing_data_accept_alias_and_ignore_unknown_data() {
    let sparse = SparsePlayerDocument {
        health: 7.0,
        food_level: 13.0,
        removed_field: "ignored".into(),
    };
    let encoded =
        document::encode_named(PLAYER_DOCUMENT_MAGIC, PLAYER_DOCUMENT_FORMAT, &sparse).unwrap();
    let decoded = decode_player_data(&encoded).unwrap();

    assert_eq!(decoded.health, 7.0);
    assert_eq!(decoded.hunger, 13.0);
    assert_eq!(decoded.position, PlayerSaveData::default().position);
    assert_eq!(decoded.hotbar, PlayerSaveData::default().hotbar);
}

#[test]
fn unknown_player_document_format_is_rejected() {
    let mut encoded = encode_player_data(&PlayerSaveData::default()).unwrap();
    encoded[4..8].copy_from_slice(&(PLAYER_DOCUMENT_FORMAT + 1).to_le_bytes());

    let error = decode_player_data(&encoded).unwrap_err();

    assert!(error.contains("不支持的文档格式"));
}

#[test]
fn runtime_id_map_layout_keeps_identifier_mapping() {
    let mut legacy = RuntimeIdMapLayout {
        layout_marker: 7,
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
        hotbar: std::array::from_fn(|_| RuntimeMappedStack::air()),
        backpack: std::array::from_fn(|_| RuntimeMappedStack::air()),
        equipment: std::array::from_fn(|_| RuntimeMappedStack::air()),
        accessories: vec![RuntimeMappedStack::air(); 6],
        item_id_map: vec![(4, "century_journey:wooden_axe".into())],
    };
    legacy.hotbar[0] = RuntimeMappedStack {
        runtime_id: Some(4),
        item: "century_journey:wooden_axe".into(),
        count: 1,
        durability: Some(31),
    };

    let decoded = decode_player_data(&encode_legacy(&legacy, true)).unwrap();

    assert_eq!(
        decoded.item_id_map,
        vec![(4, "century_journey:wooden_axe".into())]
    );
    assert_eq!(decoded.hotbar[0].runtime_id, Some(4));
    assert_eq!(decoded.hotbar[0].durability, Some(31));

    let upgraded = encode_player_data(&decoded).unwrap();
    assert_eq!(&upgraded[..4], &PLAYER_DOCUMENT_MAGIC);
    assert_eq!(
        decode_player_data(&upgraded).unwrap().item_id_map,
        decoded.item_id_map
    );
}

#[test]
fn durability_layout_defaults_runtime_identifier_mapping() {
    let mut legacy = DurabilityLayout {
        layout_marker: 6,
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
        hotbar: std::array::from_fn(|_| DurabilityStack::air()),
        backpack: std::array::from_fn(|_| DurabilityStack::air()),
        equipment: std::array::from_fn(|_| DurabilityStack::air()),
        accessories: vec![DurabilityStack::air(); 6],
    };
    legacy.hotbar[0] = DurabilityStack {
        item: "century_journey:wooden_axe".into(),
        count: 1,
        durability: Some(31),
    };

    let decoded = decode_player_data(&encode_legacy(&legacy, true)).unwrap();

    assert!(decoded.item_id_map.is_empty());
    assert_eq!(decoded.hotbar[0].runtime_id, None);
    assert_eq!(decoded.hotbar[0].durability, Some(31));
}

#[test]
fn game_build_layout_defaults_later_survival_fields() {
    let mut legacy = GameBuildLayout {
        layout_marker: 5,
        game_version: "0.2.0".into(),
        position: [4.0, 70.0, -3.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        camera_pitch: 0.2,
        gamemode: "survival".into(),
        health: 15.0,
        hunger: 11.0,
        hotbar_active: 0,
        hotbar: std::array::from_fn(|_| IdentifierCountStack::air()),
        backpack: std::array::from_fn(|_| IdentifierCountStack::air()),
        equipment: std::array::from_fn(|_| IdentifierCountStack::air()),
        accessories: vec![IdentifierCountStack::air(); 6],
    };
    legacy.hotbar[0] = IdentifierCountStack {
        item: "century_journey:wooden_axe".into(),
        count: 1,
    };

    let decoded = decode_player_data(&encode_legacy(&legacy, true)).unwrap();

    assert_eq!(decoded.saturation, 5.0);
    assert_eq!(decoded.respawn_point, [0.0, 70.0, 0.0]);
    assert_eq!(decoded.hotbar[0].durability, None);
}

#[test]
fn equipment_inventory_layout_keeps_equipment_slots() {
    let mut legacy = EquipmentInventoryLayout {
        layout_marker: 4,
        position: [1.0, 2.0, 3.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        camera_pitch: 0.1,
        gamemode: "survival".into(),
        health: 9.0,
        hunger: 8.0,
        hotbar_active: 0,
        hotbar: std::array::from_fn(|_| IdentifierCountStack::air()),
        backpack: std::array::from_fn(|_| IdentifierCountStack::air()),
        equipment: std::array::from_fn(|_| IdentifierCountStack::air()),
        accessories: vec![IdentifierCountStack::air(); 6],
    };
    legacy.equipment[6] = IdentifierCountStack {
        item: "century_journey:test_backpack".into(),
        count: 1,
    };

    let decoded = decode_player_data(&encode_legacy(&legacy, false)).unwrap();

    assert_eq!(decoded.health, 9.0);
    assert_eq!(decoded.hunger, 8.0);
    assert_eq!(decoded.equipment[6].count, 1);
}
