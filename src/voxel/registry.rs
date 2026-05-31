use std::collections::HashMap;
use std::fs;
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::*;
use crate::core::constant::texture::TILE_SIZE;
use crate::core::state::AppState;
use crate::voxel::properties::BlockProperty;

#[derive(Resource,Default)]
pub struct BlockRegistry{
    /// 根据运行时分配的动态ID查找属性
    pub id_to_properties: HashMap<u16, BlockProperty>,
    /// 通过唯一名标识进行查找动态ID
    pub identifier_to_id: HashMap<String, u16>,
    /// 纹理硬射
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

    /// 查询某个方块对应的某个面在 GPU 纹理数组中的 Layer 索引
    pub fn get_layer(&self, id: u16, face_idx: usize) -> u32 {
        *self.texture_layers.get(&(id, face_idx)).unwrap_or(&0)
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

        // 为空气方块6个面分配纹理层
        // 因为纹理中已经为空气创造了透明贴图纹理，所以可以直接使用
        for face_idx in 0..6 {
            let path = air_block.textures.get_face_texture(face_idx);
            let layer_id = path_to_layer.get(path).copied().unwrap_or(0);
            registry.texture_layers.insert((0, face_idx), layer_id);
        }
        registry.id_to_properties.insert(0, air_block);
    } else {
        // 缺少空气方块直接崩溃
        // 若空气数据没有读取，说明资源存在严重缺失
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

/// 构建纹理图集
fn build_texture_atlas(
    registry: &mut BlockRegistry,
    unique_paths: &[String],
    images: &mut Assets<Image>,
    layouts: &mut Assets<TextureAtlasLayout>,
    materials: &mut Assets<StandardMaterial>,
) {
    // 获取纹理层数量，计算图集总宽度
    let layer_count = unique_paths.len() as u32;
    let atlas_width = TILE_SIZE * layer_count;

    // 创建空的图集像素数据
    let mut atlas_data = vec![0u8; (atlas_width * TILE_SIZE * 4) as usize];

    // 遍历所有唯一贴图路径，依次绘制到纹理图集对应图层位置
    for (layer_idx, path) in unique_paths.iter().enumerate() {
        // 拼接完整贴图资源路径
        let full_path = std::path::Path::new("assets").join(path);

        // 加载贴图文件，加载失败则使用缺失占位图
        let image = match image::open(&full_path) {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                error!("无法加载贴图 {}: {}", full_path.display(), e);
                create_missing_texture_placeholder()
            }
        };

        // 将贴图缩放到固定方块大小
        let resized = image::imageops::resize(
            &image, TILE_SIZE, TILE_SIZE,
            image::imageops::FilterType::Nearest,
        );

        // 将缩放后的贴图像素数据复制到图集对应图层区域
        for y in 0..TILE_SIZE {
            let src_start = (y * TILE_SIZE * 4) as usize;
            let src_end = src_start + (TILE_SIZE * 4) as usize;
            let dest_x_offset = layer_idx as u32 * TILE_SIZE;
            let dest_start = ((y * atlas_width + dest_x_offset) * 4) as usize;
            atlas_data[dest_start..dest_start + (TILE_SIZE * 4) as usize]
                .copy_from_slice(&resized.as_raw()[src_start..src_end]);
        }
    }

    // 创建纹理图集图像
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
    // 设置像素风格采样
    array_image.sampler = ImageSampler::nearest();

    // 将生成的图集添加到资源管理器，保存到方块注册表
    let texture_handle = images.add(array_image);
    registry.base_texture = texture_handle.clone();
    // 创建并保存纹理图集布局（按网格划分每个贴图）
    registry.atlas_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(TILE_SIZE), layer_count, 1, None, None,
    ));

    // 创建不透明方块材质
    registry.opaque_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        perceptual_roughness: 0.85,
        ..default()
    });

    // 创建镂空（树叶等）方块材质
    registry.cutout_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        perceptual_roughness: 0.85,
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    });

    // 创建透明混合（玻璃等）方块材质
    registry.transparent_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.85),
        perceptual_roughness: 0.2,
        metallic: 0.05,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
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