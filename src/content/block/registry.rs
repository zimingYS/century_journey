use crate::content::block::definition::BlockProperty;
use crate::content::constant::world::CHUNK_SIZE;
use crate::content::format::load_versioned_json_dir;
use crate::engine::asset::AssetFiles;
use crate::engine::asset::manager::AssetManager;
use crate::shared::identifier::Identifier;
use crate::shared::states::app_state::AppState;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct BlockRegistry {
    id_to_properties: HashMap<u16, BlockProperty>,
    identifier_to_id: HashMap<Identifier, u16>,
    id_to_identifier: HashMap<u16, Identifier>,
    texture_layers: HashMap<(u16, usize), u32>,
    texture_paths: Vec<String>,
}

impl BlockRegistry {
    pub fn get(&self, id: u16) -> Option<&BlockProperty> {
        self.id_to_properties.get(&id)
    }

    pub fn get_id_by_identifier(&self, identifier: &str) -> Option<u16> {
        let key = Identifier::parse(identifier).ok()?;
        self.identifier_to_id.get(&key).copied()
    }

    pub fn get_identifier_by_id(&self, id: u16) -> Option<&Identifier> {
        self.id_to_identifier.get(&id)
    }

    pub fn get_layer(&self, id: u16, face_idx: usize) -> u32 {
        *self.texture_layers.get(&(id, face_idx)).unwrap_or(&0)
    }

    pub fn total_layer_count(&self) -> usize {
        self.texture_layers
            .values()
            .copied()
            .max()
            .map(|v| v as usize + 1)
            .unwrap_or(0)
    }

    pub fn get_icon_atlas_index(&self, block_id: &Identifier) -> Option<usize> {
        let runtime_id = *self.identifier_to_id.get(block_id)? as usize;
        let layer = self.get_layer(runtime_id as u16, 4) as usize;
        Some(layer * CHUNK_SIZE * CHUNK_SIZE)
    }

    pub fn build_save_id_map(&self) -> Vec<(u16, String)> {
        let mut map: Vec<(u16, String)> = self
            .id_to_identifier
            .iter()
            .map(|(&id, ident)| (id, ident.to_string()))
            .collect();
        map.sort_by_key(|(id, _)| *id);
        map
    }

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

    pub fn iter_properties(&self) -> impl Iterator<Item = (&u16, &BlockProperty)> {
        self.id_to_properties.iter()
    }

    pub fn identifiers(&self) -> impl Iterator<Item = &Identifier> {
        self.identifier_to_id.keys()
    }

    pub fn texture_layers_iter(&self) -> impl Iterator<Item = (&(u16, usize), &u32)> {
        self.texture_layers.iter()
    }

    pub fn texture_paths(&self) -> &[String] {
        &self.texture_paths
    }

    pub fn id_identifier_pairs(&self) -> impl Iterator<Item = (&u16, &Identifier)> {
        self.id_to_identifier.iter()
    }

    pub fn max_texture_layer(&self) -> u32 {
        self.texture_layers.values().copied().max().unwrap_or(0) + 1
    }
}

pub fn init_block_registry_system(
    mut commands: Commands,
    asset: Res<AssetManager>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let raw_configs = load_block_configs(&asset);

    let mut registry = BlockRegistry::default();
    register_blocks(&mut registry, raw_configs);

    commands.insert_resource(registry);
    next_state.set(AppState::MainMenu);

    info!("[block registry] loaded block definitions and switched to MainMenu");
}

fn load_block_configs(asset: &AssetManager) -> Vec<BlockProperty> {
    let files = AssetFiles::new(asset.resolver());
    let pairs = load_versioned_json_dir::<BlockProperty>(&files, "definitions/blocks");
    let count = pairs.len();
    info!("[block registry] loaded {count} block definitions through AssetManager");
    pairs.into_iter().map(|(_, prop)| prop).collect()
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
        panic!("missing required block definition: assets/definitions/blocks/air.json");
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
