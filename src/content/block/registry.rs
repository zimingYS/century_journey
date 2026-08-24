//! 维护方块标识、运行时 ID 与定义之间的映射。

use crate::content::block::definition::BlockProperty;
use crate::content::validation::ContentCompilation;
use crate::shared::identifier::Identifier;
use crate::shared::states::app_state::AppState;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
/// 维护方块运行时 ID、标识符、属性和纹理层映射的注册表。
pub struct BlockRegistry {
    id_to_properties: HashMap<u16, BlockProperty>,
    identifier_to_id: HashMap<Identifier, u16>,
    id_to_identifier: HashMap<u16, Identifier>,
    texture_layers: HashMap<(u16, usize), u32>,
    texture_paths: Vec<String>,
}

impl BlockRegistry {
    /// 返回指定键或索引对应的只读值。
    pub fn get(&self, id: u16) -> Option<&BlockProperty> {
        self.id_to_properties.get(&id)
    }

    /// 根据稳定标识符查询本次内容编译得到的运行时方块 ID。
    pub fn get_id(&self, identifier: &Identifier) -> Option<u16> {
        self.identifier_to_id.get(identifier).copied()
    }

    /// 解析字符串标识符并查询本次内容编译得到的运行时方块 ID。
    pub fn get_id_by_identifier(&self, identifier: &str) -> Option<u16> {
        let key = Identifier::parse(identifier).ok()?;
        self.get_id(&key)
    }

    /// 返回运行时方块 ID 对应的稳定标识。
    pub fn get_identifier_by_id(&self, id: u16) -> Option<&Identifier> {
        self.id_to_identifier.get(&id)
    }

    /// 返回方块指定面的纹理层索引。
    pub fn get_layer(&self, id: u16, face_idx: usize) -> u32 {
        *self.texture_layers.get(&(id, face_idx)).unwrap_or(&0)
    }

    /// 返回 atlas 所需的方块纹理层总数。
    pub fn total_layer_count(&self) -> usize {
        self.texture_layers
            .values()
            .copied()
            .max()
            .map(|v| v as usize + 1)
            .unwrap_or(0)
    }

    /// 导出写入存档的运行时 ID 与稳定标识映射。
    pub fn build_save_id_map(&self) -> Vec<(u16, String)> {
        let mut map: Vec<(u16, String)> = self
            .id_to_identifier
            .iter()
            .map(|(&id, ident)| (id, ident.to_string()))
            .collect();
        map.sort_by_key(|(id, _)| *id);
        map
    }

    /// 根据存档 ID 映射构建到当前注册表的重映射表。
    pub fn build_id_remap_table(&self, saved_map: &[(u16, String)]) -> HashMap<u16, u16> {
        let mut remap = HashMap::new();

        for (saved_id, identifier) in saved_map {
            if let Ok(key) = Identifier::parse(identifier)
                && let Some(&current_id) = self.identifier_to_id.get(&key)
            {
                remap.insert(*saved_id, current_id);
            }
        }

        remap
    }

    /// 遍历全部方块运行时 ID 与属性。
    pub fn iter_properties(&self) -> impl Iterator<Item = (&u16, &BlockProperty)> {
        self.id_to_properties.iter()
    }

    /// 遍历注册表中的全部方块标识。
    pub fn identifiers(&self) -> impl Iterator<Item = &Identifier> {
        self.identifier_to_id.keys()
    }

    /// 遍历方块面到纹理层的映射。
    pub fn texture_layers_iter(&self) -> impl Iterator<Item = (&(u16, usize), &u32)> {
        self.texture_layers.iter()
    }

    /// 返回按层索引排列的方块纹理路径。
    pub fn texture_paths(&self) -> &[String] {
        &self.texture_paths
    }

    /// 遍历方块运行时 ID 与稳定标识对。
    pub fn id_identifier_pairs(&self) -> impl Iterator<Item = (&u16, &Identifier)> {
        self.id_to_identifier.iter()
    }

    /// 返回注册表使用的最大方块纹理层。
    pub fn max_texture_layer(&self) -> u32 {
        self.texture_layers.values().copied().max().unwrap_or(0) + 1
    }
}

/// 根据已编译内容重建方块注册表。
pub fn init_block_registry_system(
    mut commands: Commands,
    compilation: Res<ContentCompilation>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !compilation.is_valid() {
        next_state.set(AppState::MainMenu);
        return;
    }

    let mut registry = BlockRegistry::default();
    register_blocks(&mut registry, compilation.content.blocks.clone());

    commands.insert_resource(registry);
    next_state.set(AppState::MainMenu);

    info!("[方块] 已注册方块定义，切换到主菜单");
}

fn register_blocks(registry: &mut BlockRegistry, mut raw_configs: Vec<BlockProperty>) {
    let mut unique_paths = Vec::new();

    for prop in &raw_configs {
        for face_idx in 0..6 {
            let path = prop.textures.get_face_texture(face_idx).to_string();
            if !unique_paths.contains(&path) {
                unique_paths.push(path);
            }
        }
    }

    let path_to_layer: HashMap<String, u32> = unique_paths
        .iter()
        .enumerate()
        .map(|(idx, path)| (path.clone(), idx as u32))
        .collect();

    if let Some(air_idx) = raw_configs
        .iter()
        .position(|p| p.identifier == "century_journey:air")
    {
        let air_block = raw_configs.remove(air_idx);

        registry
            .identifier_to_id
            .insert(air_block.identifier.clone(), 0);
        registry
            .id_to_identifier
            .insert(0, air_block.identifier.clone());

        for face_idx in 0..6 {
            let path = air_block.textures.get_face_texture(face_idx);
            let layer_id = path_to_layer.get(path).copied().unwrap_or(0);
            registry.texture_layers.insert((0, face_idx), layer_id);
        }
        registry.id_to_properties.insert(0, air_block);
    } else {
        panic!(
            "missing required block definition: assets/definitions/blocks/century_journey/air.json"
        );
    }

    for (assigned_id, block) in (1u16..).zip(raw_configs) {
        registry
            .identifier_to_id
            .insert(block.identifier.clone(), assigned_id);
        registry
            .id_to_identifier
            .insert(assigned_id, block.identifier.clone());

        for face_idx in 0..6 {
            let path = block.textures.get_face_texture(face_idx);
            let layer_id = path_to_layer[path];
            registry
                .texture_layers
                .insert((assigned_id, face_idx), layer_id);
        }

        registry.id_to_properties.insert(assigned_id, block);
    }

    registry.texture_paths = unique_paths;
}
