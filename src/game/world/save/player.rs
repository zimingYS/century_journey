use crate::game::gameplay::gamemode::{GameMode, PlayerGameMode};
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::container::hotbar::HOTBAR_SIZE;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::state::InventoryState;
use crate::game::player::components::Player;
use crate::game::player::components::stats::{Health, Hunger};
use crate::game::world::save::events::SaveDirtySource;
use crate::game::world::save::system::SaveConfig;
use crate::shared::components::camera::FpsCamera;
use crate::shared::item_id::ItemId;
use bevy::prelude::*;
use bincode::Options;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

pub const SAVE_VERSION: u32 = 3;

/// 玩家位置变化超过此距离才标记Dirty
const POSITION_DIRTY_THRESHOLD_SQ: f32 = 0.25;

// 可序列化数据结构
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SaveItemStack {
    pub item: String,
    pub count: u32,
}

impl SaveItemStack {
    /// 设置为空气
    fn air() -> Self {
        Self {
            item: "century_journey:air".into(),
            count: 0,
        }
    }
    /// 判断是否是空气
    fn is_air(&self) -> bool {
        self.item == "century_journey:air" || self.count == 0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerSaveData {
    pub version: u32,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    /// FPS相机弧度
    #[serde(default)]
    pub camera_pitch: f32,
    /// 游戏模式
    pub gamemode: String,
    /// 生命值
    #[serde(default)]
    pub health: f32,
    /// 饥饿值
    #[serde(default)]
    pub hunger: f32,
    /// 快捷栏选位
    pub hotbar_active: usize,
    #[serde(with = "serde_arrays")]
    /// 快捷栏
    pub hotbar: [SaveItemStack; HOTBAR_SIZE],
    /// 物品栏
    #[serde(with = "serde_arrays")]
    pub backpack: [SaveItemStack; 36],
    /// 装备栏
    #[serde(with = "serde_arrays")]
    pub armor: [SaveItemStack; 4],
    /// 饰品栏
    #[serde(with = "serde_arrays")]
    pub accessories: [SaveItemStack; 6],
}

impl Default for PlayerSaveData {
    fn default() -> Self {
        Self {
            version: SAVE_VERSION,
            position: [0.0, 70.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            camera_pitch: 0.0,
            gamemode: "survival".into(),
            health: 20.0,
            hunger: 20.0,
            hotbar_active: 0,
            hotbar: std::array::from_fn(|_| SaveItemStack::air()),
            backpack: std::array::from_fn(|_| SaveItemStack::air()),
            armor: std::array::from_fn(|_| SaveItemStack::air()),
            accessories: std::array::from_fn(|_| SaveItemStack::air()),
        }
    }
}

fn item_id_to_string(id: &ItemId) -> String {
    id.to_string()
}

fn string_to_item_id(s: &str) -> ItemId {
    if let Some(rest) = s.strip_prefix("item:") {
        ItemId::item(rest.to_string())
    } else if let Some(rest) = s.strip_prefix("block:") {
        ItemId::block(rest.to_string())
    } else {
        // 兼容旧存档（无前缀）
        ItemId::block(s.to_string())
    }
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

impl PlayerSaveData {
    pub fn from_runtime(
        position: Vec3,
        rotation: Quat,
        camera_pitch: f32,
        gamemode: &PlayerGameMode,
        inventory: &InventoryState,
        health: f32,
        hunger: f32,
    ) -> Self {
        let hotbar = std::array::from_fn(|i| optional_stack_to_save(inventory.hotbar.get_stack(i)));
        let backpack =
            std::array::from_fn(|i| optional_stack_to_save(inventory.survival.get_stack(i)));
        let armor =
            std::array::from_fn(|i| optional_stack_to_save(inventory.survival.get_stack(36 + i)));
        let accessories =
            std::array::from_fn(|i| optional_stack_to_save(inventory.survival.get_stack(40 + i)));

        Self {
            version: SAVE_VERSION,
            position: [position.x, position.y, position.z],
            rotation: [rotation.x, rotation.y, rotation.z, rotation.w],
            camera_pitch,
            gamemode: gamemode_to_string(gamemode.mode),
            health,
            hunger,
            hotbar_active: inventory.hotbar.active_index,
            hotbar,
            backpack,
            armor,
            accessories,
        }
    }
}

impl PlayerSaveData {
    pub fn restore_gamemode(&self) -> PlayerGameMode {
        PlayerGameMode {
            mode: string_to_gamemode(&self.gamemode),
        }
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

    pub fn camera_pitch(&self) -> f32 {
        self.camera_pitch
    }
}

// 存档健康检查
fn validate_player_data(data: &PlayerSaveData) -> PlayerSaveData {
    let mut data = data.clone();
    let mut repaired = false;

    // 位置 NaN / Infinite 检查
    if data.position.iter().any(|v| v.is_nan() || v.is_infinite()) {
        log::warn!("[存档系统] 无效位置{:?}，已重置为世界原点", data.position);
        data.position = [0.0, 70.0, 0.0];
        repaired = true;
    }
    // 旋转 NaN / Infinite 检查
    if data.rotation.iter().any(|v| v.is_nan() || v.is_infinite()) {
        log::warn!("[存档系统] 旋转无效 {:?}, 已重置为恒等矩阵", data.rotation);
        data.rotation = [0.0, 0.0, 0.0, 1.0];
        repaired = true;
    }
    // camera_pitch 合法性
    if data.camera_pitch.is_nan() || data.camera_pitch.is_infinite() {
        log::warn!(
            "[存档系统] 相机俯仰角{}无效, 已重置为0.0",
            data.camera_pitch
        );
        data.camera_pitch = 0.0;
        repaired = true;
    }
    // gamemode 合法性
    if !matches!(data.gamemode.as_str(), "survival" | "creative") {
        log::warn!(
            "[存档系统] 未知游戏模式: '{}', 已重置为生存模式",
            data.gamemode
        );
        data.gamemode = "survival".into();
        repaired = true;
    }
    // hotbar_active 越界
    if data.hotbar_active >= HOTBAR_SIZE {
        log::warn!(
            "[存档系统] 快捷栏索引 {} 超出索引范围,已重置为0",
            data.hotbar_active
        );
        data.hotbar_active = 0;
        repaired = true;
    }
    // 物品合法性
    for (slot, kind) in data
        .hotbar
        .iter_mut()
        .map(|s| (s, "hotbar"))
        .chain(data.backpack.iter_mut().map(|s| (s, "backpack")))
        .chain(data.armor.iter_mut().map(|s| (s, "armor")))
        .chain(data.accessories.iter_mut().map(|s| (s, "accessories")))
    {
        if slot.is_air() {
            continue;
        }
        if slot.item.is_empty() || !slot.item.contains(':') {
            log::warn!(
                "[存档系统] '{}'中的物品{}无效,已替换为空气",
                slot.item,
                kind
            );
            *slot = SaveItemStack::air();
            repaired = true;
        }
    }

    if repaired {
        log::warn!("[存档系统] 保存数据出现问题 — 已自动修复");
    }
    data
}

#[derive(Resource, Debug)]
pub struct PlayerSaveManager {
    /// 是否需要保存
    pub dirty: bool,
    /// 最近一次脏数据的来源
    pub last_dirty_source: Option<SaveDirtySource>,
    /// 累计保存次数
    pub total_saves: u64,
    /// 上次保存时间 (游戏时间, 秒)
    pub last_save_time: f64,
    /// 上次保存时的玩家位置 (用于距离阈值判断)
    last_saved_position: Vec3,
    /// 自动保存计时器
    auto_save_timer: f32,
}

impl Default for PlayerSaveManager {
    fn default() -> Self {
        Self {
            dirty: false,
            last_dirty_source: None,
            total_saves: 0,
            last_save_time: 0.0,
            last_saved_position: Vec3::ZERO,
            auto_save_timer: 30.0,
        }
    }
}

impl PlayerSaveManager {
    pub const AUTO_SAVE_INTERVAL: f32 = 30.0;

    /// 设置脏标记, 记录来源
    pub fn set_dirty(&mut self, source: SaveDirtySource) {
        if !self.dirty {
            self.dirty = true;
            self.last_dirty_source = Some(source);
        }
    }

    /// 检查位置是否需要标记脏 (阈值: 0.5 block)
    pub fn check_position_dirty(&mut self, current_pos: Vec3) -> bool {
        let dist_sq = current_pos.distance_squared(self.last_saved_position);
        if dist_sq > POSITION_DIRTY_THRESHOLD_SQ {
            self.set_dirty(SaveDirtySource::Position);
            true
        } else {
            false
        }
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

// 序列化 / 反序列化
fn write_player_data(data: &PlayerSaveData, path: &std::path::Path) -> Result<(), String> {
    let serialized = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(data)
        .map_err(|e| format!("bincode serialize: {e}"))?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder
        .write_all(&serialized)
        .map_err(|e| format!("gzip write: {e}"))?;
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
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| format!("gzip decompress: {e}"))?;
    let data: PlayerSaveData = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .deserialize(&decompressed)
        .map_err(|e| format!("bincode deserialize: {e}"))?;
    Ok(data)
}

fn player_save_path(world_name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("saves")
        .join(world_name)
        .join("players")
        .join("singleplayer.dat")
}

// 统一保存 (V3 — 含统计更新)
fn perform_save(
    world_name: &str,
    gamemode: &PlayerGameMode,
    inventory: &InventoryState,
    player_query: &Query<&Transform, With<Player>>,
    camera_query: &Query<&FpsCamera, With<Camera3d>>,
    save_manager: &mut PlayerSaveManager,
    time: &Time,
) {
    let transform = player_query.single().cloned().unwrap_or_default();
    let pitch = camera_query.single().map(|c| c.pitch).unwrap_or(0.0);
    let data = PlayerSaveData::from_runtime(
        transform.translation,
        transform.rotation,
        pitch,
        gamemode,
        inventory,
        20.0,
        20.0,
    );

    let path = player_save_path(world_name);
    match write_player_data(&data, &path) {
        Ok(()) => {
            save_manager.dirty = false;
            save_manager.last_dirty_source = None;
            save_manager.total_saves += 1;
            save_manager.last_save_time = time.elapsed_secs() as f64;
            save_manager.last_saved_position = transform.translation;
            log::info!(
                "[存档系统] 玩家已保存(共计: {}),保存到{:?}",
                save_manager.total_saves,
                path
            );
        }
        Err(e) => {
            log::error!("[存档系统] 保存失败: {e} !");
        }
    }
}

pub fn load_player_on_enter_system(
    save_config: Res<SaveConfig>,
    mut gamemode: ResMut<PlayerGameMode>,
    mut inventory: ResMut<InventoryState>,
    mut player_query: Query<(&mut Transform, &mut Health, &mut Hunger), With<Player>>,
    mut camera_query: Query<&mut FpsCamera, With<Camera3d>>,
    mut save_manager: ResMut<PlayerSaveManager>,
    time: Res<Time>,
) {
    let save_path = player_save_path(&save_config.world_name);
    let raw_data = if save_path.exists() {
        match read_player_data(&save_path) {
            Ok(data) => {
                log::info!("[存档系统] 从 {:?} 加载数据成功", save_path);
                data
            }
            Err(e) => {
                log::warn!("[存档系统] 从 {} 加载失败, 已使用默认值", e);
                PlayerSaveData::default()
            }
        }
    } else {
        log::info!("[存档系统] 存档不存在，正在已默认值创建");
        PlayerSaveData::default()
    };

    // 健康检查 + 自动修复
    let save_data = validate_player_data(&raw_data);

    // 恢复 GameMode
    *gamemode = save_data.restore_gamemode();

    // 恢复 Inventory
    let restored = save_data.restore_inventory();
    inventory.hotbar = restored.hotbar;
    inventory.survival = restored.survival;

    // 恢复 Transform + Health + Hunger
    if let Ok((mut transform, mut health, mut hunger)) = player_query.single_mut() {
        *transform = save_data.restore_transform();
        save_manager.last_saved_position = transform.translation;

        health.current = save_data.health.clamp(0.0, health.max);
        hunger.current = save_data.hunger.clamp(0.0, hunger.max);
        hunger.saturation = 5.0;
    }

    // 恢复 Camera Pitch
    if let Ok(mut fps_camera) = camera_query.single_mut() {
        fps_camera.pitch = save_data.camera_pitch();
    }

    // 标记初始保存位置, 确保首次退出前一定写入
    save_manager.set_dirty(SaveDirtySource::Position);

    // 强制标记 inventory 变化，确保下一帧视觉同步刷新所有图标
    inventory.set_changed();

    log::info!(
        "[存档系统] 玩家已生成,位置:{:?},游戏模式:{}",
        save_data.position,
        save_data.gamemode
    );
}

/// 玩家位置脏数据追踪
pub fn player_position_dirty_system(
    player_query: Query<&Transform, (With<Player>, Changed<Transform>)>,
    mut save_manager: ResMut<PlayerSaveManager>,
) {
    for transform in &player_query {
        save_manager.check_position_dirty(transform.translation);
    }
}

/// 背包变化脏数据追踪
pub fn inventory_dirty_tracking_system(
    inventory: Res<InventoryState>,
    mut save_manager: ResMut<PlayerSaveManager>,
) {
    if inventory.is_changed() {
        save_manager.set_dirty(SaveDirtySource::Inventory);
    }
}

/// 游戏模式变化脏数据追踪
pub fn gamemode_dirty_tracking_system(
    gamemode: Res<PlayerGameMode>,
    mut save_manager: ResMut<PlayerSaveManager>,
) {
    if gamemode.is_changed() {
        save_manager.set_dirty(SaveDirtySource::GameMode);
    }
}

// 自动保存
pub fn auto_save_player_system(
    time: Res<Time>,
    save_config: Res<SaveConfig>,
    gamemode: Res<PlayerGameMode>,
    inventory: Res<InventoryState>,
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&FpsCamera, With<Camera3d>>,
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
        &camera_query,
        &mut save_manager,
        &time,
    );
}

/// AppExit 事件触发立即保存
pub fn save_on_exit_system(
    mut exit_reader: MessageReader<AppExit>,
    save_config: Res<SaveConfig>,
    gamemode: Res<PlayerGameMode>,
    inventory: Res<InventoryState>,
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&FpsCamera, With<Camera3d>>,
    mut save_manager: ResMut<PlayerSaveManager>,
    time: Res<Time>,
) {
    if exit_reader.read().next().is_none() {
        return;
    }
    if !save_manager.dirty {
        return;
    }
    log::info!("[存档系统] 检测到游戏退出,正在保存游戏...");
    perform_save(
        &save_config.world_name,
        &gamemode,
        &inventory,
        &player_query,
        &camera_query,
        &mut save_manager,
        &time,
    );
}
