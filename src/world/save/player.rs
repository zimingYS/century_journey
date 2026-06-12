use std::io::{Read, Write};
use bevy::prelude::*;
use bincode::Options;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};

use crate::gameplay::gamemode::{GameMode, PlayerGameMode};
use crate::inventory::container::hotbar::HOTBAR_SIZE;
use crate::inventory::container::InventoryContainer;
use crate::inventory::item::id::ItemId;
use crate::inventory::item::stack::ItemStack;
use crate::inventory::state::InventoryState;
use crate::player::components::Player;
use crate::world::save::system::SaveConfig;

// ═══════════════════════════════════════════════════════════════════════════════
// 可序列化数据结构
// ═══════════════════════════════════════════════════════════════════════════════

pub const CURRENT_SAVE_VERSION: u32 = 1;

/// 可序列化的物品堆叠（ItemStack 的持久化等价物）
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SaveItemStack {
    pub item: String,
    pub count: u32,
}

impl SaveItemStack {
    fn air() -> Self {
        Self { item: "century_journey:air".into(), count: 0 }
    }
    fn is_air(&self) -> bool {
        self.item == "century_journey:air" || self.count == 0
    }
}

/// 可序列化的玩家完整状态
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerSaveData {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub gamemode: String,
    pub health: f32,
    pub hunger: f32,
    pub hotbar_active: usize,
    #[serde(with = "serde_arrays")]
    pub hotbar: [SaveItemStack; HOTBAR_SIZE],
    #[serde(with = "serde_arrays")]
    pub backpack: [SaveItemStack; 36],
    #[serde(with = "serde_arrays")]
    pub armor: [SaveItemStack; 4],
    #[serde(with = "serde_arrays")]
    pub accessories: [SaveItemStack; 6],
    pub version: u32,
}

impl Default for PlayerSaveData {
    fn default() -> Self {
        Self {
            position: [0.0, 70.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            gamemode: "survival".into(),
            health: 20.0,
            hunger: 20.0,
            hotbar_active: 0,
            hotbar: std::array::from_fn(|_| SaveItemStack::air()),
            backpack: std::array::from_fn(|_| SaveItemStack::air()),
            armor: std::array::from_fn(|_| SaveItemStack::air()),
            accessories: std::array::from_fn(|_| SaveItemStack::air()),
            version: CURRENT_SAVE_VERSION,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 转换辅助
// ═══════════════════════════════════════════════════════════════════════════════

fn item_id_to_string(id: &ItemId) -> String {
    id.to_string()
}

fn string_to_item_id(s: &str) -> ItemId {
    ItemId::block(s.to_string())
}

fn optional_stack_to_save(opt: Option<&ItemStack>) -> SaveItemStack {
    match opt {
        Some(s) if !s.is_empty() => SaveItemStack {
            item: item_id_to_string(&s.item),
            count: s.count,
        },
        _ => SaveItemStack::air(),
    }
}

fn save_to_optional_stack(slot: &SaveItemStack) -> Option<ItemStack> {
    if slot.is_air() {
        None
    } else {
        Some(ItemStack::new(string_to_item_id(&slot.item), slot.count))
    }
}

fn gamemode_to_string(mode: GameMode) -> String {
    match mode {
        GameMode::Survival => "survival".into(),
        GameMode::Creative => "creative".into(),
    }
}

fn string_to_gamemode(s: &str) -> GameMode {
    match s {
        "creative" => GameMode::Creative,
        _ => GameMode::Survival,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Runtime → SaveData
// ═══════════════════════════════════════════════════════════════════════════════

impl PlayerSaveData {
    pub fn from_runtime(
        position: Vec3,
        rotation: Quat,
        gamemode: &PlayerGameMode,
        inventory: &InventoryState,
        health: f32,
        hunger: f32,
    ) -> Self {
        let hotbar = std::array::from_fn(|i| {
            optional_stack_to_save(inventory.hotbar.get_stack(i))
        });

        let backpack = std::array::from_fn(|i| {
            optional_stack_to_save(inventory.survival.get_stack(i))
        });

        let armor = std::array::from_fn(|i| {
            optional_stack_to_save(inventory.survival.get_stack(36 + i))
        });

        let accessories = std::array::from_fn(|i| {
            optional_stack_to_save(inventory.survival.get_stack(40 + i))
        });

        Self {
            position: [position.x, position.y, position.z],
            rotation: [rotation.x, rotation.y, rotation.z, rotation.w],
            gamemode: gamemode_to_string(gamemode.mode),
            health,
            hunger,
            hotbar_active: inventory.hotbar.active_index,
            hotbar,
            backpack,
            armor,
            accessories,
            version: CURRENT_SAVE_VERSION,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SaveData → Runtime
// ═══════════════════════════════════════════════════════════════════════════════

impl PlayerSaveData {
    pub fn restore_gamemode(&self) -> PlayerGameMode {
        PlayerGameMode { mode: string_to_gamemode(&self.gamemode) }
    }

    pub fn restore_inventory(&self) -> InventoryState {
        let mut state = InventoryState::default();

        for (i, slot) in self.hotbar.iter().enumerate() {
            if let Some(stack) = save_to_optional_stack(slot) {
                state.hotbar.set_stack(i, stack);
            }
        }
        state.hotbar.active_index = self.hotbar_active.min(HOTBAR_SIZE - 1);

        for (i, slot) in self.backpack.iter().enumerate() {
            if let Some(stack) = save_to_optional_stack(slot) {
                state.survival.set_stack(i, stack);
            }
        }
        for (i, slot) in self.armor.iter().enumerate() {
            if let Some(stack) = save_to_optional_stack(slot) {
                state.survival.set_stack(36 + i, stack);
            }
        }
        for (i, slot) in self.accessories.iter().enumerate() {
            if let Some(stack) = save_to_optional_stack(slot) {
                state.survival.set_stack(40 + i, stack);
            }
        }

        state
    }

    pub fn restore_transform(&self) -> Transform {
        let [x, y, z] = self.position;
        let [rx, ry, rz, rw] = self.rotation;
        Transform {
            translation: Vec3::new(x, y, z),
            rotation: Quat::from_xyzw(rx, ry, rz, rw),
            scale: Vec3::ONE,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PlayerSaveManager 资源
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Resource, Debug)]
pub struct PlayerSaveManager {
    pub dirty: bool,
    auto_save_timer: f32,
}

impl Default for PlayerSaveManager {
    fn default() -> Self {
        Self {
            dirty: false,
            auto_save_timer: 30.0,
        }
    }
}

impl PlayerSaveManager {
    pub const AUTO_SAVE_INTERVAL: f32 = 30.0;

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn reset_timer(&mut self) {
        self.auto_save_timer = Self::AUTO_SAVE_INTERVAL;
    }

    fn tick(&mut self, dt: f32) -> bool {
        if !self.dirty {
            return false;
        }
        self.auto_save_timer -= dt;
        if self.auto_save_timer <= 0.0 {
            self.reset_timer();
            true
        } else {
            false
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 序列化 / 反序列化
// ═══════════════════════════════════════════════════════════════════════════════

fn write_player_data(data: &PlayerSaveData, path: &std::path::Path) -> Result<(), String> {
    let serialized = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(data)
        .map_err(|e| format!("bincode serialize: {e}"))?;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&serialized).map_err(|e| format!("gzip write: {e}"))?;
    let compressed = encoder.finish().map_err(|e| format!("gzip finish: {e}"))?;

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path, compressed).map_err(|e| format!("file write: {e}"))?;
    Ok(())
}

fn read_player_data(path: &std::path::Path) -> Result<PlayerSaveData, String> {
    let compressed = std::fs::read(path).map_err(|e| format!("file read: {e}"))?;
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).map_err(|e| format!("gzip decompress: {e}"))?;

    let data: PlayerSaveData = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .deserialize(&decompressed)
        .map_err(|e| format!("bincode deserialize: {e}"))?;

    Ok(data)
}

// ═══════════════════════════════════════════════════════════════════════════════
// 辅助: 构建路径
// ═══════════════════════════════════════════════════════════════════════════════

fn player_save_path(world_name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("saves").join(world_name).join("players").join("singleplayer.dat")
}

// ═══════════════════════════════════════════════════════════════════════════════
// 统一保存
// ═══════════════════════════════════════════════════════════════════════════════

fn perform_save(
    world_name: &str,
    gamemode: &PlayerGameMode,
    inventory: &InventoryState,
    player_query: &Query<&Transform, With<Player>>,
    save_manager: &mut PlayerSaveManager,
) {
    let transform = player_query.single().cloned().unwrap_or_default();
    let data = PlayerSaveData::from_runtime(
        transform.translation,
        transform.rotation,
        gamemode,
        inventory,
        20.0,
        20.0,
    );

    let path = player_save_path(world_name);
    match write_player_data(&data, &path) {
        Ok(()) => {
            save_manager.dirty = false;
            log::info!("[SaveV2] player saved to {:?}", path);
        }
        Err(e) => {
            log::error!("[SaveV2] save failed: {e}");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ECS 系统
// ═══════════════════════════════════════════════════════════════════════════════

/// OnEnter(InGame) — 加载玩家存档
pub fn load_player_on_enter_system(
    save_config: Res<SaveConfig>,
    mut gamemode: ResMut<PlayerGameMode>,
    mut inventory: ResMut<InventoryState>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let save_path = player_save_path(&save_config.world_name);

    let save_data = if save_path.exists() {
        match read_player_data(&save_path) {
            Ok(data) => {
                log::info!("[SaveV2] loaded player from {:?}", save_path);
                data
            }
            Err(e) => {
                log::warn!("[SaveV2] load failed: {}, using defaults", e);
                PlayerSaveData::default()
            }
        }
    } else {
        log::info!("[SaveV2] no save found, creating default player");
        PlayerSaveData::default()
    };

    *gamemode = save_data.restore_gamemode();
    let restored = save_data.restore_inventory();
    inventory.hotbar = restored.hotbar;
    inventory.survival = restored.survival;

    if let Ok(mut transform) = player_query.single_mut() {
        *transform = save_data.restore_transform();
    }

    log::info!(
        "[SaveV2] player restored: pos={:?}, mode={}",
        save_data.position, save_data.gamemode
    );
}

/// 自动保存（每 30 秒，仅 dirty 时）
pub fn auto_save_player_system(
    time: Res<Time>,
    save_config: Res<SaveConfig>,
    gamemode: Res<PlayerGameMode>,
    inventory: Res<InventoryState>,
    player_query: Query<&Transform, With<Player>>,
    mut save_manager: ResMut<PlayerSaveManager>,
) {
    if !save_manager.tick(time.delta_secs()) {
        return;
    }
    perform_save(
        &save_config.world_name,
        &gamemode,
        &inventory,
        &player_query,
        &mut save_manager,
    );
}

/// 退出前最后一帧保存（Last schedule，每帧检查仅 dirty 时写入）
pub fn save_on_exit_system(
    save_config: Res<SaveConfig>,
    gamemode: Res<PlayerGameMode>,
    inventory: Res<InventoryState>,
    player_query: Query<&Transform, With<Player>>,
    mut save_manager: ResMut<PlayerSaveManager>,
) {
    if !save_manager.dirty {
        return;
    }
    log::info!("[SaveV2] flushing player save...");
    perform_save(
        &save_config.world_name,
        &gamemode,
        &inventory,
        &player_query,
        &mut save_manager,
    );
}
