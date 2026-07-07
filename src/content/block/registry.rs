use crate::client::renderer::tex_atlas::build_texture_atlas;
use crate::content::block::definition::BlockProperty;
use crate::content::block::sound::SoundMaterial;
use crate::content::constant::world::CHUNK_SIZE;
use crate::engine::asset::AssetFiles;
use crate::engine::asset::manager::AssetManager;
use crate::shared::identifier::Identifier;
use crate::shared::states::app_state::AppState;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct BlockRegistry {
    /// 根据运行时分配的动态ID查找属性
    id_to_properties: HashMap<u16, BlockProperty>,
    /// 通过唯一名标识进行查找动态ID
    identifier_to_id: HashMap<Identifier, u16>,
    /// 反向映射：通过动态ID查找唯一名标识
    id_to_identifier: HashMap<u16, Identifier>,
    /// 纹理映射 {(block_id, face_idx) -> layer_id}
    texture_layers: HashMap<(u16, usize), u32>,
    /// 保存基础长条图集纹理
    base_texture: Handle<Image>,
    /// 保存图集布局句柄
    atlas_layout: Handle<TextureAtlasLayout>,
    /// 不透明材质
    opaque_material: Handle<StandardMaterial>,
    /// 镂空材质
    cutout_material: Handle<StandardMaterial>,
    /// 透明材质
    transparent_material: Handle<StandardMaterial>,
    /// 音效路径
    sound_paths: HashMap<SoundMaterial, SoundPaths>,
}

/// 某种音效材质的所有音效路径
#[derive(Debug, Clone, Default)]
pub struct SoundPaths {
    pub break_sound: String,
    pub place_sound: String,
    pub step_sound: String,
    pub dig_sound: String,
    pub fall_on_sound: String,
    pub interact_sound: String,
    pub open_sound: String,
    pub close_sound: String,
    pub reset_sound: String,
    pub grow_sound: String,
    pub ignore_sound: String,
    pub extinguish_sound: String,
    pub flow_sound: String,
}

impl BlockRegistry {
    /// 获取注册的方块属性
    pub fn get(&self, id: u16) -> Option<&BlockProperty> {
        self.id_to_properties.get(&id)
    }

    /// 通过字符串唯一标识获取运行时数字 ID
    pub fn get_id_by_identifier(&self, identifier: &str) -> Option<u16> {
        let key = Identifier::parse(identifier).ok()?;
        self.identifier_to_id.get(&key).copied()
    }

    /// 通过动态ID获取标识符
    pub fn get_identifier_by_id(&self, id: u16) -> Option<&Identifier> {
        self.id_to_identifier.get(&id)
    }

    /// 查询某个方块对应的某个面在 GPU 纹理数组中的 Layer 索引
    pub fn get_layer(&self, id: u16, face_idx: usize) -> u32 {
        *self.texture_layers.get(&(id, face_idx)).unwrap_or(&0)
    }

    /// 纹理图集中唯一纹理的总层数 (用于 UV 归一化)
    pub fn total_layer_count(&self) -> usize {
        self.texture_layers
            .values()
            .copied()
            .max()
            .map(|v| v as usize + 1)
            .unwrap_or(0)
    }

    /// 查询方块图标对应的图集 tile index (仅 Block 图标)
    pub fn get_icon_atlas_index(&self, block_id: &Identifier) -> Option<usize> {
        let runtime_id = *self.identifier_to_id.get(block_id)? as usize;
        let layer = self.get_layer(runtime_id as u16, 4) as usize;
        Some(layer * CHUNK_SIZE * CHUNK_SIZE)
    }

    /// 构建保存存档的ID映射表(将动态ID转换为方块标识符)
    pub fn build_save_id_map(&self) -> Vec<(u16, String)> {
        let mut map: Vec<(u16, String)> = self
            .id_to_identifier
            .iter()
            .map(|(&id, ident)| (id, ident.to_string()))
            .collect();
        map.sort_by_key(|(id, _)| *id);
        map
    }

    /// 构建读取存档的动态ID(将标识符重新读取为对应动态ID的方块)
    pub fn build_id_remap_table(&self, saved_map: &[(u16, String)]) -> HashMap<u16, u16> {
        let mut remap = HashMap::new();

        for (saved_id, identifier) in saved_map {
            if let Ok(key) = Identifier::parse(identifier)
                && let Some(&current_id) = self.identifier_to_id.get(&key)
            {
                remap.insert(*saved_id, current_id);
            }
            // 如果标识符在当前注册表中不存在，不添加映射
            // 加载时未映射的 ID 会被替换为空气
        }
        remap
    }

    // ─── 属性访问 ────────────────────────────────────────
    /// 按 ID 遍历所有方块属性
    pub fn iter_properties(&self) -> impl Iterator<Item = (&u16, &BlockProperty)> {
        self.id_to_properties.iter()
    }

    /// 获取所有已注册方块标识符的迭代器
    pub fn identifiers(&self) -> impl Iterator<Item = &Identifier> {
        self.identifier_to_id.keys()
    }

    /// 获取所有纹理层映射的迭代器
    pub fn texture_layers_iter(&self) -> impl Iterator<Item = (&(u16, usize), &u32)> {
        self.texture_layers.iter()
    }

    /// 遍历所有 ID→标识符 映射
    pub fn id_identifier_pairs(&self) -> impl Iterator<Item = (&u16, &Identifier)> {
        self.id_to_identifier.iter()
    }

    // ─── 渲染资源访问 ────────────────────────────────────
    /// 获取基础纹理图集 Handle
    pub fn base_texture(&self) -> &Handle<Image> {
        &self.base_texture
    }

    /// 获取图集布局 Handle
    pub fn atlas_layout(&self) -> &Handle<TextureAtlasLayout> {
        &self.atlas_layout
    }

    /// 设置基础纹理图集 Handle
    pub fn set_base_texture(&mut self, handle: Handle<Image>) {
        self.base_texture = handle;
    }

    /// 设置图集布局 Handle
    pub fn set_atlas_layout(&mut self, layout: Handle<TextureAtlasLayout>) {
        self.atlas_layout = layout;
    }

    /// 获取指定渲染模式的材质 Handle
    pub fn material(
        &self,
        mode: crate::content::block::definition::RenderMode,
    ) -> &Handle<StandardMaterial> {
        match mode {
            crate::content::block::definition::RenderMode::Opaque => &self.opaque_material,
            crate::content::block::definition::RenderMode::Transparent => {
                &self.transparent_material
            }
            _ => &self.cutout_material,
        }
    }

    /// 获取不透明材质 Handle
    pub fn opaque_material(&self) -> &Handle<StandardMaterial> {
        &self.opaque_material
    }

    /// 获取镂空材质 Handle
    pub fn cutout_material(&self) -> &Handle<StandardMaterial> {
        &self.cutout_material
    }

    /// 获取透明材质 Handle
    pub fn transparent_material(&self) -> &Handle<StandardMaterial> {
        &self.transparent_material
    }

    /// 设置不透明材质 Handle
    pub fn set_opaque_material(&mut self, handle: Handle<StandardMaterial>) {
        self.opaque_material = handle;
    }

    /// 设置镂空材质 Handle
    pub fn set_cutout_material(&mut self, handle: Handle<StandardMaterial>) {
        self.cutout_material = handle;
    }

    /// 设置透明材质 Handle
    pub fn set_transparent_material(&mut self, handle: Handle<StandardMaterial>) {
        self.transparent_material = handle;
    }

    /// 获取所有纹理层中最大的层索引 + 1
    pub fn max_texture_layer(&self) -> u32 {
        self.texture_layers.values().copied().max().unwrap_or(0) + 1
    }

    /// 注册内置音效路径
    fn register_builtin_sounds(&mut self) {
        self.sound_paths.insert(
            SoundMaterial::Stone,
            SoundPaths {
                break_sound: "sounds/block/stone/break.ogg".to_string(),
                place_sound: "sounds/block/stone/place.ogg".to_string(),
                step_sound: "sounds/block/stone/step.ogg".to_string(),
                ..default()
            },
        );
        self.sound_paths.insert(
            SoundMaterial::Dirt,
            SoundPaths {
                break_sound: "sounds/block/dirt/break.ogg".to_string(),
                place_sound: "sounds/block/dirt/place.ogg".to_string(),
                step_sound: "sounds/block/dirt/step.ogg".to_string(),
                ..default()
            },
        );
        self.sound_paths.insert(
            SoundMaterial::Grass,
            SoundPaths {
                break_sound: "sounds/block/grass/break.ogg".to_string(),
                place_sound: "sounds/block/grass/place.ogg".to_string(),
                step_sound: "sounds/block/grass/step.ogg".to_string(),
                ..default()
            },
        );
        self.sound_paths.insert(
            SoundMaterial::Wood,
            SoundPaths {
                break_sound: "sounds/block/wood/break.ogg".to_string(),
                place_sound: "sounds/block/wood/place.ogg".to_string(),
                step_sound: "sounds/block/wood/step.ogg".to_string(),
                ..default()
            },
        );
        self.sound_paths.insert(
            SoundMaterial::Sand,
            SoundPaths {
                break_sound: "sounds/block/sand/break.ogg".to_string(),
                place_sound: "sounds/block/sand/place.ogg".to_string(),
                step_sound: "sounds/block/sand/step.ogg".to_string(),
                ..default()
            },
        );
        self.sound_paths.insert(
            SoundMaterial::Glass,
            SoundPaths {
                break_sound: "sounds/block/glass/break.ogg".to_string(),
                place_sound: "sounds/block/glass/place.ogg".to_string(),
                step_sound: "sounds/block/glass/step.ogg".to_string(),
                ..default()
            },
        );
        self.sound_paths.insert(
            SoundMaterial::Snow,
            SoundPaths {
                break_sound: "sounds/block/snow/break.ogg".to_string(),
                place_sound: "sounds/block/snow/place.ogg".to_string(),
                step_sound: "sounds/block/snow/step.ogg".to_string(),
                ..default()
            },
        );
    }
}

/// 注册方块系统
pub fn init_block_registry_system(
    mut commands: Commands,
    asset: Res<AssetManager>,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let raw_configs = load_block_configs(&asset);

    // 注册方块并分配动态ID
    let mut registry = BlockRegistry::default();
    registry.register_builtin_sounds();
    let unique_paths = register_blocks(&mut registry, raw_configs);

    // 构建纹理图集并创建材质
    build_texture_atlas(
        &mut registry,
        &unique_paths,
        &mut images,
        &mut layouts,
        &mut materials,
        &asset,
    );

    // 插入资源并切换状态
    commands.insert_resource(registry);
    next_state.set(AppState::InGame);

    info!("[方块注册] 核心方块资产注册完毕，游戏状态切入 InGame，正在生成 3D 噪声地形...");
}

/// 通过 AssetManager 加载所有方块 JSON 配置
fn load_block_configs(asset: &AssetManager) -> Vec<BlockProperty> {
    let files = AssetFiles::new(asset.resolver());
    let pairs = files.read_json_dir::<BlockProperty>("definitions/blocks");
    let count = pairs.len();
    info!("[方块注册] 通过 AssetManager 加载了 {} 个方块配置", count);
    pairs.into_iter().map(|(_, prop)| prop).collect()
}

/// 注册方块动态ID
fn register_blocks(
    registry: &mut BlockRegistry,
    mut raw_configs: Vec<BlockProperty>,
) -> Vec<String> {
    // 收集所有独立贴图路径
    let mut unique_paths = Vec::new();

    // 遍历所有方块配置，收集6个面的贴图路径并去重
    for prop in &raw_configs {
        for face_idx in 0..6 {
            let path = prop.textures.get_face_texture(face_idx).to_string();
            if !unique_paths.contains(&path) {
                unique_paths.push(path);
            }
        }
    }

    // 为每个唯一贴图分配一个数字ID
    let path_to_layer: HashMap<String, u32> = unique_paths
        .iter()
        .enumerate()
        .map(|(idx, path)| (path.clone(), idx as u32))
        .collect();

    // 单独处理空气方块
    if let Some(air_idx) = raw_configs
        .iter()
        .position(|p| p.identifier == "century_journey:air")
    {
        // 从配置列表中移除空气方块
        let air_block = raw_configs.remove(air_idx);

        // 注册方块标识符
        registry
            .identifier_to_id
            .insert(air_block.identifier.clone(), 0);
        // 注册反向映射
        registry
            .id_to_identifier
            .insert(0, air_block.identifier.clone());

        // 为空气方块6个面分配纹理层
        for face_idx in 0..6 {
            let path = air_block.textures.get_face_texture(face_idx);
            let layer_id = path_to_layer.get(path).copied().unwrap_or(0);
            registry.texture_layers.insert((0, face_idx), layer_id);
        }
        registry.id_to_properties.insert(0, air_block);
    } else {
        // 缺少空气方块直接崩溃
        panic!("严重错误：在 assets/definitions/blocks/ 中未找到 air.json！");
    }

    // 处理其余所有普通方块
    let mut current_max_id = 1u16;
    for block in raw_configs {
        let assigned_id = current_max_id;

        // 注册标识符与ID映射
        registry
            .identifier_to_id
            .insert(block.identifier.clone(), assigned_id);
        // 注册反向映射
        registry
            .id_to_identifier
            .insert(assigned_id, block.identifier.clone());

        // 为当前方块的6个面设置纹理层
        for face_idx in 0..6 {
            let path = block.textures.get_face_texture(face_idx);
            let layer_id = path_to_layer[path];
            registry
                .texture_layers
                .insert((assigned_id, face_idx), layer_id);
        }

        // 注册方块属性
        registry.id_to_properties.insert(assigned_id, block);

        // 动态增加ID编号
        current_max_id += 1;
    }
    unique_paths
}
