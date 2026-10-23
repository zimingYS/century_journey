use crate::input::SectionSnapshot;
use bevy::platform::collections::HashMap;
use bevy::prelude::{Entity, Handle, Resource, StandardMaterial};
use cj_core::pos::SectionPos;

/// 渲染区域的 Chunk 半径
pub const RENDER_CHUNK_RADIUS: i32 = 8;

/// 渲染所需的 Section 快照集合
#[derive(Resource, Default)]
pub struct SectionSnapshots {
    /// 当前渲染区域内的 Section 快照
    sections: Vec<SectionSnapshot>,
}

impl SectionSnapshots {
    /// 使用已有的 Section 快照集合创建资源
    pub fn new(sections: Vec<SectionSnapshot>) -> Self {
        Self { sections }
    }

    /// 获取全部 Section 快照
    #[inline]
    pub fn sections(&self) -> &[SectionSnapshot] {
        &self.sections
    }

    /// 判断当前是否没有任何 Section 快照
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }
}

/// 脏区块队列
#[derive(Resource, Default)]
pub struct DirtySections {
    sections: Vec<(SectionPos, SectionSnapshot)>,
}

impl DirtySections {
    /// 入队
    pub fn push(&mut self, section: SectionPos, snapshot: SectionSnapshot) {
        self.sections.push((section, snapshot));
    }

    /// 取走并清空
    pub fn take(&mut self) -> Vec<(SectionPos, SectionSnapshot)> {
        std::mem::take(&mut self.sections)
    }
}

/// 已建网格的登记簿
#[derive(Resource)]
pub struct SectionMeshes {
    entities: HashMap<SectionPos, Entity>,
    material: Option<Handle<StandardMaterial>>,
}

impl Default for SectionMeshes {
    fn default() -> Self {
        Self {
            entities: HashMap::new(),
            material: None,
        }
    }
}

impl SectionMeshes {
    /// 查找实体
    pub fn entity(&self, section: SectionPos) -> Option<Entity> {
        self.entities.get(&section).copied()
    }

    /// 插入实体
    pub fn insert(&mut self, section: SectionPos, entity: Entity) {
        self.entities.insert(section, entity);
    }

    /// 注销实体
    pub fn remove(&mut self, section: SectionPos) -> Option<Entity> {
        self.entities.remove(&section)
    }

    /// 获取材质
    pub fn material(&self) -> Option<&Handle<StandardMaterial>> {
        self.material.as_ref()
    }

    /// 存材质
    pub fn set_material(&mut self, material: Handle<StandardMaterial>) {
        self.material = Some(material);
    }
}
