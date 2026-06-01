use std::collections::HashMap;
use std::fs;
use bevy::prelude::*;
use crate::core::state::app_state::AppState;
use crate::voxel::properties::BlockProperty;
use crate::voxel::texture_atlas::build_texture_atlas;

#[derive(Resource,Default)]
pub struct BlockRegistry{
    /// 根据运行时分配的动态ID查找属性
    pub id_to_properties: HashMap<u16, BlockProperty>,
    /// 通过唯一名标识进行查找动态ID
    pub identifier_to_id: HashMap<String, u16>,
    /// 反向映射：通过动态ID查找唯一名标识
    pub id_to_identifier: HashMap<u16, String>,
    /// 纹理映射
    pub texture_layers: HashMap<(u16, usize), u32>,
    /// 保存基础长条图集纹理
    pub base_texture: Handle<Image>,
    /// 保存图集布局句柄
    pub atlas_layout: Handle<TextureAtlasLayout>,
    /// 不透明材质
    pub opaque_material: Handle<StandardMaterial>,
    /// 镂空材质
    pub cutout_material: Handle<StandardMaterial>,
    /// 透明材质
    pub transparent_material: Handle<StandardMaterial>,
}

impl BlockRegistry{
    /// 获取注册的方块属性
    pub fn get(&self, id: u16) -> Option<&BlockProperty> {
        self.id_to_properties.get(&id)
    }

    /// 通过字符串唯一标识获取运行时数字 ID
    pub fn get_id_by_identifier(&self, identifier: &str) -> Option<u16> {
        self.identifier_to_id.get(identifier).copied()
    }

    /// 通过动态ID获取字符串唯一标识
    pub fn get_identifier_by_id(&self, id: u16) -> Option<&str> {
        self.id_to_identifier.get(&id).map(|s| s.as_str())
    }

    /// 查询某个方块对应的某个面在 GPU 纹理数组中的 Layer 索引
    pub fn get_layer(&self, id: u16, face_idx: usize) -> u32 {
        *self.texture_layers.get(&(id, face_idx)).unwrap_or(&0)
    }

    /// 构建保存存档的ID映射表(将动态ID转换为方块标识符)
    pub fn build_save_id_map(&self) -> Vec<(u16, String)> {
        // self.blocks: HashMap<String, BlockProperty>
        // BlockProperty.runtime_id: u16 (#[serde(skip)])
        let mut map: Vec<(u16, String)> = self
            .id_to_identifier
            .iter()
            .map(|(&id, name)| (id, name.clone()))
            .collect();
        map.sort_by_key(|(id, _)| *id);
        map
    }

    /// 构建读取存档的动态ID(将标识符重新读取为对应动态ID的方块)
    pub fn build_id_remap_table(
        &self,
        saved_map: &[(u16, String)],
    ) -> HashMap<u16, u16> {
        let mut remap = HashMap::new();

        for (saved_id, identifier) in saved_map {
            if let Some(&current_id) = self.identifier_to_id.get(identifier) {
                remap.insert(*saved_id, current_id);
            }
            // 如果标识符在当前注册表中不存在，不添加映射
            // 加载时未映射的 ID 会被替换为空气
        }
        remap
    }
}


/// 注册方块系统
pub fn init_block_registry_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // 加载方块配置
    let raw_configs = load_block_configs();

    // 注册方块并分配动态ID
    let mut registry = BlockRegistry::default();
    let unique_paths = register_blocks(&mut registry, raw_configs);

    // 构建纹理图集并创建材质
    build_texture_atlas(&mut registry, &unique_paths, &mut images, &mut layouts, &mut materials);

    // 插入资源并切换状态
    commands.insert_resource(registry);
    next_state.set(AppState::InGame);

    info!("[世纪之旅] 核心方块资产注册完毕，游戏状态切入 InGame，正在生成 3D 噪声地形...");
}

/// 从文件系统加载所有方块的JSON配置
fn load_block_configs() -> Vec<BlockProperty> {
    let block_dir = "assets/definitions/blocks";
    let mut raw_configs: Vec<BlockProperty> = Vec::new();

    if let Ok(entries) = fs::read_dir(block_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(json_content) = fs::read_to_string(&path) {
                    match serde_json::from_str::<BlockProperty>(&json_content) {
                        Ok(prop) => raw_configs.push(prop),
                        Err(err) => error!("解析方块配置文件出错 {:?}: {:?}!", path, err),
                    }
                }
            }
        }
    } else {
        error!("找不到方块资产定义目录: {}!", block_dir);
        let _ = fs::create_dir_all(block_dir);
    }

    info!("模块化资源系统：成功扫描并加载了 {} 个独立方块配置文件！", raw_configs.len());
    raw_configs
}

/// 注册方块动态ID
fn register_blocks(
    registry: &mut BlockRegistry,
    mut raw_configs: Vec<BlockProperty>,
) -> Vec<String>{
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
    if let Some(air_idx) = raw_configs.iter().position(|p| p.identifier == "century_journey:air") {
        // 从配置列表中移除空气方块
        let mut air_block = raw_configs.remove(air_idx);

        // 强制空气方块运行时ID为0
        air_block.runtime_id = 0;

        // 注册方块标识符
        registry.identifier_to_id.insert(air_block.identifier.clone(), 0);
        // 注册反向映射
        registry.id_to_identifier.insert(0, air_block.identifier.clone());

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
    for mut block in raw_configs {
        let assigned_id = current_max_id;
        // 设置方块运行时ID
        block.runtime_id = assigned_id;

        // 注册标识符与ID映射
        registry.identifier_to_id.insert(block.identifier.clone(), assigned_id);
        // 注册反向映射
        registry.id_to_identifier.insert(assigned_id, block.identifier.clone());

        // 为当前方块的6个面设置纹理层
        for face_idx in 0..6 {
            let path = block.textures.get_face_texture(face_idx);
            let layer_id = path_to_layer[path];
            registry.texture_layers.insert((assigned_id, face_idx), layer_id);
        }

        // 注册方块属性
        registry.id_to_properties.insert(assigned_id, block);

        // 动态增加ID编号
        current_max_id += 1;
    }

    unique_paths
}