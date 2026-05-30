use std::collections::HashMap;
use std::fs;
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::*;
use crate::core::constant::TILE_SIZE;
use crate::core::state::AppState;
use crate::ui::resources::inventory_ui_state::InventoryUiState;
use crate::voxel::properties::BlockProperty;

#[derive(Resource,Default)]
pub struct BlockRegistry{
    /// 根据运行时分配的动态ID查找属性
    pub id_to_properties: HashMap<u16, BlockProperty>,
    /// 通过唯一名标识进行查找动态ID
    pub identifier_to_id: HashMap<String, u16>,
    /// 纹理硬射
    pub texture_layers: HashMap<(u16, usize), u32>,
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

    /// 查询某个方块对应的某个面在 GPU 纹理数组中的 Layer 索引
    pub fn get_layer(&self, id: u16, face_idx: usize) -> u32 {
        *self.texture_layers.get(&(id, face_idx)).unwrap_or(&0)
    }
}


/// 注册方块系统
pub fn init_block_registry_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<AppState>>,
){
    // 初始化注册容器
    let mut registry = BlockRegistry::default();
    let mut raw_configs: Vec<BlockProperty> = Vec::new();

    // 方块数据目录
    let block_dir = "assets/definitions/blocks";
    if let Ok(entries) = std::fs::read_dir(block_dir) {
        // 读取每一个文件
        for entry in entries.flatten() {
            let path = entry.path();
            // 确保仅读取JSON
            if path.extension().and_then(|s| s.to_str()) == Some("json"){
                if let Ok(json_content) = fs::read_to_string(&path) {
                    match serde_json::from_str::<BlockProperty>(&json_content) {
                        Ok(prop) => {
                            raw_configs.push(prop);
                        }
                        Err(err) => {
                            error!("解析方块配置文件出错 {:?}: {:?}!", path, err);
                        }
                    }
                }
            }
        }
    }else {
        error!("找不到方块资产定义目录: {}!", block_dir);
        let _ = fs::create_dir_all(block_dir);
    }

    info!("模块化资源系统：成功扫描并加载了 {} 个独立方块配置文件！", raw_configs.len());

    // 收集所有独立贴图
    let mut unique_paths = Vec::new();
    for prop in &raw_configs {
        for face_idx in 0..6 {
            let path = prop.textures.get_face_texture(face_idx).to_string();
            if !unique_paths.contains(&path) {
                unique_paths.push(path);
            }
        }
    }

    // 将每个路径映射为一个在图集中的2D索引
    let path_to_layer: HashMap<String, u32> = unique_paths
        .iter()
        .enumerate()
        .map(|(idx, path)| (path.clone(), idx as u32))
        .collect();

    if let Some(air_idx) = raw_configs.iter().position(|p| p.identifier == "century_journey:air") {
        let mut air_block = raw_configs.remove(air_idx); // 从待分配列表中移除
        let assigned_id = 0; // 强制分配 0
        air_block.runtime_id = assigned_id;

        registry.identifier_to_id.insert(air_block.identifier.clone(), assigned_id);

        // 绑定空气的 6 个面的贴图层号（即使空气不渲染网格，也建立绑定防止后续边界查询 panic）
        for face_idx in 0..6 {
            let path = air_block.textures.get_face_texture(face_idx);
            if let Some(&layer_id) = path_to_layer.get(path) {
                registry.texture_layers.insert((assigned_id, face_idx), layer_id);
            } else {
                registry.texture_layers.insert((assigned_id, face_idx), 0);
            }
        }
        registry.id_to_properties.insert(assigned_id, air_block);
    } else {
        // 严重警报：如果没有 air.json，直接崩溃防止内存错乱
        panic!("严重错误：在 assets/definitions/blocks/ 中未找到 air.json！");
    }

    // 运行时动态分配ID
    let mut current_max_id = 1; // 从1开始，0分配给空气

    for mut block in raw_configs{
        let assigned_id = current_max_id;

        block.runtime_id = assigned_id;
        // 根据字符标识生成数字ID
        registry.identifier_to_id.insert(block.identifier.clone(), assigned_id);

        // 绑定该方块六个面对应的纹理层号
        for face_idx in 0..6 {
            let path = block.textures.get_face_texture(face_idx);
            let layer_id = path_to_layer[path];
            registry.texture_layers.insert((assigned_id, face_idx), layer_id);
        }

        // 映射动态ID
        registry.id_to_properties.insert(assigned_id, block);
        current_max_id += 1;
    }

    let layer_count = unique_paths.len() as u32;
    let atlas_width = TILE_SIZE * layer_count;

    let mut atlas_data = vec![0u8; (atlas_width * TILE_SIZE * 4) as usize];

    for (layer_idx, path) in unique_paths.iter().enumerate() {
        let full_path = std::path::Path::new("assets").join(path);

        // 加载贴图，失败则使用紫黑格占位符
        let image = match image::open(&full_path) {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                error!("无法加载贴图 {}: {}", full_path.display(), e);
                create_missing_texture_placeholder()
            }
        };

        // 缩放到标准尺寸
        let resized = image::imageops::resize(
            &image,
            TILE_SIZE,
            TILE_SIZE,
            image::imageops::FilterType::Nearest
        );

        for y in 0..TILE_SIZE {
            let src_start = (y * TILE_SIZE * 4) as usize;
            let src_end = src_start + (TILE_SIZE * 4) as usize;

            let dest_x_offset = layer_idx as u32 * TILE_SIZE;
            let dest_start = ((y * atlas_width + dest_x_offset) * 4) as usize;

            atlas_data[dest_start..dest_start + (TILE_SIZE * 4) as usize]
                .copy_from_slice(&resized.as_raw()[src_start..src_end]);
        }
    }

    let mut array_image = Image::new(
        Extent3d {
            width: TILE_SIZE * layer_count,
            height: TILE_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        atlas_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );

    array_image.sampler = ImageSampler::nearest();

    let texture_handle = images.add(array_image);

    // 不透明材质
    registry.opaque_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        perceptual_roughness: 0.85,
        ..default()
    });

    // 镂空材质
    registry.cutout_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        perceptual_roughness: 0.85,
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    });

    // 透明混合材质
    registry.transparent_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.85),
        perceptual_roughness: 0.2,
        metallic: 0.05,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let mut ui_state = InventoryUiState::default();

    for identifier in registry.identifier_to_id.keys() {
        if identifier != "century_journey:air" {
            ui_state.creative_palette.push(identifier.clone());
        }
    }

    ui_state.creative_palette.sort();
    commands.insert_resource(registry);
    commands.insert_resource(ui_state);
    next_state.set(AppState::InGame);

    info!("[世纪之旅] 核心方块资产注册完毕，游戏状态切入 InGame，正在生成 3D 噪声地形...");
}

/// 创建紫黑格缺失贴图占位符
fn create_missing_texture_placeholder() -> image::RgbaImage {
    let mut img = image::RgbaImage::new(TILE_SIZE, TILE_SIZE);
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let color = if (x / 4 + y / 4) % 2 == 0 {
                image::Rgba([255, 0, 255, 255]) // 紫色
            } else {
                image::Rgba([0, 0, 0, 255]) // 黑色
            };
            img.put_pixel(x, y, color);
        }
    }
    img
}