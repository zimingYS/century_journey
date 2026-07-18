use crate::content::block::registry::BlockRegistry;
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::shared::identifier::Identifier;
use crate::shared::tag::identifier::TagId;
use bevy::prelude::*;
use noise::Perlin;
use std::collections::HashSet;

/// 生成地形主要使用的方块缓存
#[derive(Clone)]
pub struct GenerationBlockIds {
    pub air: u16,
    pub grass: u16,
    pub dirt: u16,
    pub stone: u16,
    pub sand: u16,
    pub water: u16,
    pub snow: u16,
    pub leaves: u16,
    pub wood: u16,
    /// 可种树地表方块ID集合
    pub tree_plantable_ids: HashSet<u16>,
    /// 自然方块ID集合
    pub natural_ids: HashSet<u16>,
    /// 可替换方块ID集合
    pub overworld_replaceable_ids: HashSet<u16>,
}

impl GenerationBlockIds {
    /// 游戏在调用生成前，从 Bevy 的中央注册表中一次性把名字翻译成数字 ID
    pub fn from_registry(registry: &BlockRegistry, tag_registry: &RuntimeTagRegistry) -> Self {
        let tree_plantable_tag = TagId::new("century_journey", "tree_plantable");
        let natural_tag = TagId::new("century_journey", "natural");
        let overworld_replaceable_tag = TagId::new("century_journey", "overworld_replaceable");

        Self {
            air: 0,
            grass: registry
                .get_id_by_identifier("century_journey:grass")
                .unwrap_or(0),
            dirt: registry
                .get_id_by_identifier("century_journey:dirt")
                .unwrap_or(0),
            stone: registry
                .get_id_by_identifier("century_journey:stone")
                .unwrap_or(0),
            sand: registry
                .get_id_by_identifier("century_journey:sand")
                .unwrap_or(0),
            water: registry
                .get_id_by_identifier("century_journey:water")
                .unwrap_or(0),
            snow: registry
                .get_id_by_identifier("century_journey:snow")
                .unwrap_or(0),
            leaves: registry
                .get_id_by_identifier("century_journey:leaves")
                .unwrap_or(0),
            wood: registry
                .get_id_by_identifier("century_journey:wood")
                .unwrap_or(0),
            tree_plantable_ids: tag_registry
                .get_ids(&tree_plantable_tag)
                .cloned()
                .unwrap_or_else(|| {
                    let mut set = HashSet::new();
                    if let Some(id) = registry.get_id_by_identifier("century_journey:grass") {
                        set.insert(id);
                    }
                    set
                }),
            natural_ids: tag_registry
                .get_ids(&natural_tag)
                .cloned()
                .unwrap_or_default(),
            overworld_replaceable_ids: tag_registry
                .get_ids(&overworld_replaceable_tag)
                .cloned()
                .unwrap_or_default(),
        }
    }

    /// 查询方块是否可在该地表种树
    pub fn is_tree_plantable(&self, block_id: u16) -> bool {
        self.tree_plantable_ids.contains(&block_id)
    }

    /// 查询方块是否为自然方块
    pub fn is_natural(&self, block_id: u16) -> bool {
        self.natural_ids.contains(&block_id)
    }

    /// 查询方块是否可被主世界替换
    pub fn is_overworld_replaceable(&self, block_id: u16) -> bool {
        self.overworld_replaceable_ids.contains(&block_id)
    }

    /// 从方块标识符解析到ID（支持群系定义中的 Identifier 类型）
    pub fn resolve_block_id(&self, identifier: &Identifier) -> u16 {
        match (identifier.namespace(), identifier.path()) {
            ("century_journey", "grass") => self.grass,
            ("century_journey", "dirt") => self.dirt,
            ("century_journey", "stone") => self.stone,
            ("century_journey", "sand") => self.sand,
            ("century_journey", "water") => self.water,
            ("century_journey", "snow") => self.snow,
            ("century_journey", "leaves") => self.leaves,
            ("century_journey", "wood") => self.wood,
            _ => self.grass,
        }
    }
}

/// 缓存方块ID资源，避免每帧重建
#[derive(Resource, Clone)]
pub struct CachedBlockIds(pub GenerationBlockIds);

/// 多层噪声采样器
pub struct NoiseSampler {
    /// 种子
    pub seed: u32,
    /// 主地形噪声（大尺度起伏）
    pub terrain_primary: Perlin,
    /// 地形细节噪声（小尺度变化）
    pub terrain_detail: Perlin,
    /// 粗糙度噪声
    pub roughness: Perlin,
    /// 洞穴噪声
    pub cave: Perlin,
}

impl NoiseSampler {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            terrain_primary: Perlin::new(seed),
            terrain_detail: Perlin::new(seed.wrapping_add(100)),
            roughness: Perlin::new(seed.wrapping_add(200)),
            cave: Perlin::new(seed.wrapping_add(300)),
        }
    }
}

impl Clone for NoiseSampler {
    fn clone(&self) -> Self {
        Self::new(self.seed)
    }
}
