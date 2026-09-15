use cj_core::pos::{BlockPos, ChunkPos, SectionLocalPos};
use cj_core::spec::{SECTION_COUNT_USIZE, SECTION_MIN_Y, SECTION_SIZE, SECTION_VOLUME};
use cj_core::voxel::{BlockId, Voxel};
use std::collections::HashMap;
use std::fmt;

/// 方块存储单元
#[derive(PartialEq, Eq)]
pub struct Section {
    voxels: Box<[Voxel; SECTION_VOLUME]>,
}

impl Section {
    /// 创建全空气的空 Section
    pub fn new() -> Self {
        Section {
            voxels: Box::new([Voxel::AIR; SECTION_VOLUME]),
        }
    }

    /// 根据局部坐标查询对应方块
    #[inline]
    pub fn get(&self, local: SectionLocalPos) -> Voxel {
        self.voxels[local.index()]
    }

    /// 根据局部坐标修改对应方块
    #[inline]
    pub fn set(&mut self, local: SectionLocalPos, voxel: Voxel) {
        self.voxels[local.index()] = voxel
    }

    /// 根据线性下标查询对应方块。
    #[inline]
    pub fn get_raw(&self, index: usize) -> Voxel {
        debug_assert!(
            index < SECTION_VOLUME,
            "voxel index out of range: {index} >= {SECTION_VOLUME}"
        );

        self.voxels[index]
    }

    /// 根据线性下标设置对应方块。
    #[inline]
    pub fn set_raw(&mut self, index: usize, voxel: Voxel) {
        debug_assert!(
            index < SECTION_VOLUME,
            "voxel index out of range: {index} >= {SECTION_VOLUME}"
        );

        self.voxels[index] = voxel
    }

    /// 判断整层是否全空气
    #[inline]
    pub fn is_all_air(&self) -> bool {
        self.voxels.iter().all(|voxel| voxel.is_air())
    }

    /// 整个 Section 是否为同一种方块，是则返回其 `BlockId`
    #[inline]
    pub fn is_uniform(&self) -> Option<BlockId> {
        let first = self.voxels[0].block();

        self.voxels
            .iter()
            .all(|voxel| voxel.block() == first)
            .then_some(first)
    }

    /// 计算非空气数量
    #[inline]
    pub fn count_non_air(&self) -> u32 {
        self.voxels.iter().filter(|voxel| !voxel.is_air()).count() as u32
    }
}

impl Default for Section {
    fn default() -> Self {
        Section::new()
    }
}

impl fmt::Debug for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Section(non_air: {})", self.count_non_air())
    }
}

/// 区块存储单元
pub struct Chunk {
    /// 区块坐标
    pos: ChunkPos,
    /// 区块包含的切片
    sections: [Option<Box<Section>>; SECTION_COUNT_USIZE],
}

impl Chunk {
    /// 构造全空区块
    pub fn new(pos: ChunkPos) -> Self {
        Chunk {
            pos,
            sections: std::array::from_fn(|_| None),
        }
    }

    /// 获取自己区块位置
    #[inline]
    pub fn pos(&self) -> ChunkPos {
        self.pos
    }

    /// 获取区块内的方块
    #[inline]
    pub fn get_block(&self, block_pos: BlockPos) -> Voxel {
        debug_assert_eq!(
            block_pos.chunk_pos(),
            self.pos,
            "block {:?} does not belong to chunk {:?}",
            block_pos,
            self.pos,
        );

        let section_pos = block_pos.section_pos();
        let section_index = section_pos.index();
        let local = block_pos.section_local();

        match self.sections[section_index].as_deref() {
            Some(section) => section.get(local),
            None => Voxel::AIR,
        }
    }

    /// 修改区块内方块
    #[inline]
    pub fn set_block(&mut self, block_pos: BlockPos, voxel: Voxel) -> bool {
        debug_assert_eq!(
            block_pos.chunk_pos(),
            self.pos,
            "block {:?} does not belong to chunk {:?}",
            block_pos,
            self.pos,
        );

        let section_pos = block_pos.section_pos();
        let section_index = section_pos.index();
        let local = block_pos.section_local();

        match &mut self.sections[section_index] {
            Some(section) => {
                section.set(local, voxel);
                true
            }
            None if voxel.is_air() => false,
            None => {
                let mut section = Box::new(Section::new());
                section.set(local, voxel);
                self.sections[section_index] = Some(section);
                true
            }
        }
    }

    /// 根据索引获取切片
    #[inline]
    pub fn section(&self, index: usize) -> Option<&Section> {
        self.sections[index].as_deref()
    }

    /// 根据索引获取可变切片
    #[inline]
    pub fn section_mut(&mut self, index: usize) -> Option<&mut Section> {
        self.sections[index].as_deref_mut()
    }

    /// 根据世界 Y 获取切片
    #[inline]
    pub fn section_at(&self, block_y: i32) -> Option<&Section> {
        let section_y = block_y.div_euclid(SECTION_SIZE);
        let index = section_y - SECTION_MIN_Y;

        if !(0..SECTION_COUNT_USIZE as i32).contains(&index) {
            return None;
        }

        self.sections[index as usize].as_deref()
    }

    /// 根据世界 Y 获取可变切片
    #[inline]
    pub fn section_at_mut(&mut self, block_y: i32) -> Option<&mut Section> {
        let section_y = block_y.div_euclid(SECTION_SIZE);
        let index = section_y - SECTION_MIN_Y;

        if !(0..SECTION_COUNT_USIZE as i32).contains(&index) {
            return None;
        }

        self.sections[index as usize].as_deref_mut()
    }

    /// 将区块切片插入到对应区块
    #[inline]
    pub fn insert_section(&mut self, index: usize, section: Section) {
        debug_assert!(
            index < SECTION_COUNT_USIZE,
            "section index out of range: {index} >= {SECTION_COUNT_USIZE}"
        );

        self.sections[index] = Some(Box::new(section));
    }

    /// 判断是否存在区块对应切片
    #[inline]
    pub fn has_section(&self, index: usize) -> bool {
        self.sections[index].is_some()
    }

    /// 遍历对应区块切片
    #[inline]
    pub fn iter_sections(&self) -> impl Iterator<Item = (usize, &Section)> {
        self.sections
            .iter()
            .enumerate()
            .filter_map(|(index, section)| section.as_deref().map(|section| (index, section)))
    }

    /// 已分配的切片数
    #[inline]
    pub fn section_count(&self) -> usize {
        self.sections
            .iter()
            .filter(|section| section.is_some())
            .count()
    }

    /// 是否没有已分配切片
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.sections.iter().all(|section| section.is_none())
    }

    /// 全 Chunk 非空气格数
    #[inline]
    pub fn non_air_count(&self) -> u32 {
        self.iter_sections()
            .map(|(_, section)| section.count_non_air())
            .sum()
    }
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Chunk(pos: ({}, {}), sections: {}/{}, non_air: {})",
            self.pos.x,
            self.pos.z,
            self.section_count(),
            SECTION_COUNT_USIZE,
            self.non_air_count(),
        )
    }
}

/// 世界存储容器
#[derive(Default)]
pub struct ChunkStore {
    chunks: HashMap<ChunkPos, Chunk>,
}

impl ChunkStore {
    /// 根据区块位置获取对应区块数据
    #[inline]
    pub fn get(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    /// 根据区块位置获取对应可变区块数据
    #[inline]
    pub fn get_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos)
    }

    /// 判断对应位置的区块是否存在
    #[inline]
    pub fn contains(&self, pos: ChunkPos) -> bool {
        self.chunks.contains_key(&pos)
    }

    /// 获取区块数据长度
    #[inline]
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// 判断区块数据是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// 遍历区块数据迭代器
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&ChunkPos, &Chunk)> {
        self.chunks.iter()
    }

    /// 替换旧区块数据
    #[inline]
    pub fn insert(&mut self, chunk: Chunk) -> Option<Chunk> {
        self.chunks.insert(chunk.pos, chunk)
    }

    /// 移除（卸载）区块
    #[inline]
    pub fn remove(&mut self, pos: ChunkPos) -> Option<Chunk> {
        self.chunks.remove(&pos)
    }

    /// 获取或由闭包构造区块
    #[inline]
    pub fn get_or_insert_with<F>(&mut self, pos: ChunkPos, f: F) -> &mut Chunk
    where
        F: FnOnce() -> Chunk,
    {
        self.chunks.entry(pos).or_insert_with(f)
    }

    /// 获取或创建空区块
    #[inline]
    pub fn get_or_default(&mut self, pos: ChunkPos) -> &mut Chunk {
        self.get_or_insert_with(pos, || Chunk::new(pos))
    }
}

impl fmt::Debug for ChunkStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChunkStore(chunks: {})", self.chunks.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cj_core::spec::{WORLD_BOTTOM, WORLD_TOP};
    use cj_core::voxel::{ShapeId, VoxelState};

    // 构造一个方块体素
    fn stone() -> Voxel {
        Voxel::cube(BlockId::from_raw(1))
    }

    // 构造属于 chunk 的方块坐标
    fn block_in(chunk: ChunkPos, lx: i32, y: i32, lz: i32) -> BlockPos {
        debug_assert!((0..16).contains(&lx) && (0..16).contains(&lz));
        BlockPos::new(chunk.x * 16 + lx, y, chunk.z * 16 + lz)
    }

    // --- Section ---

    #[test]
    fn section_new_is_all_air() {
        let section = Section::new();
        assert!(section.is_all_air());
        assert_eq!(section.count_non_air(), 0);
        assert_eq!(section.is_uniform(), Some(BlockId::AIR));
    }

    #[test]
    fn section_default_equals_new() {
        assert_eq!(Section::default(), Section::new());
    }

    #[test]
    fn section_set_get_round_trip() {
        let mut section = Section::new();
        let local = SectionLocalPos { x: 3, y: 7, z: 11 };

        assert_eq!(section.get(local), Voxel::AIR);
        section.set(local, stone());
        assert_eq!(section.get(local), stone());
    }

    #[test]
    fn section_raw_and_local_index_agree() {
        let mut section = Section::new();
        let local = SectionLocalPos { x: 15, y: 2, z: 9 };

        section.set(local, stone());
        assert_eq!(section.get_raw(local.index()), stone());
    }

    #[test]
    fn section_uniform_detects_single_block() {
        let mut section = Section::new();
        for i in 0..SECTION_VOLUME {
            section.set_raw(i, stone());
        }

        assert_eq!(section.is_uniform(), Some(BlockId::from_raw(1)));
        assert!(!section.is_all_air());
    }

    #[test]
    fn section_uniform_detects_mixture() {
        let mut section = Section::new();
        for i in 0..SECTION_VOLUME {
            section.set_raw(i, stone());
        }
        section.set_raw(0, Voxel::AIR);

        assert_eq!(section.is_uniform(), None);
        assert!(!section.is_all_air());
        assert_eq!(section.count_non_air(), SECTION_VOLUME as u32 - 1);
    }

    #[test]
    fn section_uniform_by_block_ignores_shape_state() {
        let mut section = Section::new();
        let a = Voxel::new(BlockId::from_raw(4), ShapeId::CUBE, VoxelState::DEFAULT);
        let b = Voxel::new(
            BlockId::from_raw(4),
            ShapeId::from_raw(2),
            VoxelState::from_raw(9),
        );

        section.set_raw(0, a);
        for i in 1..SECTION_VOLUME {
            section.set_raw(i, b);
        }

        assert_eq!(section.is_uniform(), Some(BlockId::from_raw(4)));
    }

    #[test]
    fn section_count_non_air_counts_all() {
        let mut section = Section::new();
        assert_eq!(section.count_non_air(), 0);

        section.set_raw(0, stone());
        section.set_raw(4095, stone());
        assert_eq!(section.count_non_air(), 2);
    }

    // --- Chunk::section_at / section_at_mut（上一轮 P0 bug 的回归锁）---

    #[test]
    fn section_at_rejects_out_of_range_y() {
        let chunk = Chunk::new(ChunkPos { x: 0, z: 0 });

        assert!(chunk.section_at(WORLD_BOTTOM - 1).is_none());
        assert!(chunk.section_at(WORLD_TOP).is_none());
        assert!(chunk.section_at(WORLD_TOP + 1000).is_none());
        assert!(chunk.section_at(i32::MIN).is_none());
        assert!(chunk.section_at(i32::MAX).is_none());
    }

    #[test]
    fn section_at_handles_negative_y() {
        let chunk = Chunk::new(ChunkPos { x: 0, z: 0 });

        assert!(chunk.section_at(-1).is_none());
        assert!(chunk.section_at(-16).is_none());
        assert!(chunk.section_at(-257).is_none(), "below world bottom");
    }

    #[test]
    fn section_at_maps_world_y_to_section_index() {
        let mut chunk = Chunk::new(ChunkPos { x: 0, z: 0 });

        chunk.insert_section(0, Section::new());
        assert!(chunk.section_at(WORLD_BOTTOM).is_some());
        assert!(chunk.section_at(WORLD_BOTTOM + 15).is_some());
        assert!(
            chunk.section_at(WORLD_BOTTOM + 16).is_none(),
            "crossing into index 1, which was never inserted"
        );
    }

    #[test]
    fn section_at_last_index_boundary() {
        let mut chunk = Chunk::new(ChunkPos { x: 0, z: 0 });
        let last = SECTION_COUNT_USIZE - 1;

        chunk.insert_section(last, Section::new());

        assert!(chunk.section_at(WORLD_TOP - 1).is_some());
        assert!(chunk.section_at(WORLD_TOP - 16).is_some());
        assert!(
            chunk.section_at(WORLD_TOP).is_none(),
            "world top is exclusive"
        );
    }

    #[test]
    fn section_at_mut_writes_through() {
        let mut chunk = Chunk::new(ChunkPos { x: 0, z: 0 });
        chunk.insert_section(0, Section::new());

        let y = WORLD_BOTTOM;
        chunk.set_block(BlockPos::new(1, y, 2), stone());
        assert_eq!(chunk.section_at(y).map(Section::count_non_air), Some(1));

        if let Some(section) = chunk.section_at_mut(y) {
            section.set(SectionLocalPos { x: 5, y: 5, z: 5 }, stone());
        }
        assert_eq!(chunk.section_at(y).map(Section::count_non_air), Some(2));
        assert!(chunk.section_at_mut(WORLD_TOP).is_none());
    }

    // --- Chunk 方块读写 ---

    #[test]
    fn chunk_get_block_on_missing_section_is_air() {
        let chunk = Chunk::new(ChunkPos { x: 0, z: 0 });

        assert_eq!(chunk.get_block(BlockPos::new(0, 0, 0)), Voxel::AIR);
        assert_eq!(chunk.get_block(BlockPos::new(15, -200, 15)), Voxel::AIR);
    }

    #[test]
    fn chunk_set_block_creates_section_lazily() {
        let mut chunk = Chunk::new(ChunkPos { x: 0, z: 0 });
        assert!(chunk.is_empty());

        let wrote = chunk.set_block(BlockPos::new(1, 5, 2), stone());
        assert!(wrote, "write into a newly created section must report true");
        assert_eq!(chunk.section_count(), 1);
        assert_eq!(chunk.get_block(BlockPos::new(1, 5, 2)), stone());
    }

    #[test]
    fn chunk_set_air_on_missing_section_returns_false() {
        let mut chunk = Chunk::new(ChunkPos { x: 0, z: 0 });

        let wrote = chunk.set_block(BlockPos::new(0, 0, 0), Voxel::AIR);
        assert!(!wrote);
        assert_eq!(chunk.section_count(), 0, "air write must not allocate");
    }

    #[test]
    fn chunk_set_block_negative_y() {
        let mut chunk = Chunk::new(ChunkPos { x: 0, z: 0 });

        chunk.set_block(BlockPos::new(0, -1, 0), stone());
        assert_eq!(chunk.get_block(BlockPos::new(0, -1, 0)), stone());
        assert!(
            chunk.section(15).is_some(),
            "y = -1 belongs to section 15, covering y in [-16, 0)"
        );
        assert!(
            chunk.section(14).is_none(),
            "neighbor section must stay empty"
        );
    }

    #[test]
    fn chunk_distinguishes_negative_and_positive_columns() {
        let mut chunk = Chunk::new(ChunkPos { x: -1, z: -1 });

        chunk.set_block(BlockPos::new(-1, 0, -1), stone());
        assert_eq!(chunk.get_block(BlockPos::new(-1, 0, -1)), stone());
        assert_eq!(
            BlockPos::new(-1, 0, -1).section_local(),
            SectionLocalPos { x: 15, y: 0, z: 15 },
            "chunk(-1,-1) covers x,z in [-16,-1]"
        );
    }

    #[test]
    fn chunk_section_count_and_non_air_count() {
        let mut chunk = Chunk::new(ChunkPos { x: 0, z: 0 });

        assert_eq!(chunk.section_count(), 0);
        assert_eq!(chunk.non_air_count(), 0);
        assert!(chunk.is_empty());

        chunk.set_block(BlockPos::new(0, 0, 0), stone());
        chunk.set_block(BlockPos::new(1, 0, 0), stone());
        assert_eq!(chunk.section_count(), 1);
        assert_eq!(chunk.non_air_count(), 2);

        chunk.set_block(BlockPos::new(0, 200, 0), stone());
        assert_eq!(chunk.section_count(), 2);
        assert_eq!(chunk.non_air_count(), 3);
        assert!(!chunk.is_empty());
    }

    #[test]
    fn chunk_iter_sections_yields_stored_only() {
        let mut chunk = Chunk::new(ChunkPos { x: 0, z: 0 });
        chunk.insert_section(0, Section::new());
        chunk.insert_section(31, Section::new());

        let indices: Vec<usize> = chunk.iter_sections().map(|(i, _)| i).collect();
        assert_eq!(indices, vec![0, 31]);
    }

    #[test]
    fn chunk_has_section_matches_storage() {
        let mut chunk = Chunk::new(ChunkPos { x: 0, z: 0 });
        assert!(!chunk.has_section(0));
        chunk.insert_section(0, Section::new());
        assert!(chunk.has_section(0));
        assert!(!chunk.has_section(1));
    }

    #[test]
    fn chunk_insert_section_replaces_existing() {
        let mut chunk = Chunk::new(ChunkPos { x: 0, z: 0 });

        let mut filled = Section::new();
        for i in 0..SECTION_VOLUME {
            filled.set_raw(i, stone());
        }

        chunk.insert_section(0, filled);
        assert_eq!(chunk.non_air_count(), SECTION_VOLUME as u32);

        chunk.insert_section(0, Section::new());
        assert_eq!(chunk.section_count(), 1, "insert must replace, not stack");
        assert_eq!(chunk.non_air_count(), 0);
    }

    // --- ChunkStore ---

    #[test]
    fn store_insert_returns_replaced_chunk() {
        let mut store = ChunkStore::default();
        let pos = ChunkPos { x: 1, z: 2 };
        let inside = block_in(pos, 1, 0, 1);

        assert!(store.insert(Chunk::new(pos)).is_none());

        let mut old = Chunk::new(pos);
        old.set_block(inside, stone());
        store.insert(old);

        let replaced = store.insert(Chunk::new(pos));
        assert!(
            replaced.is_some(),
            "second insert must return the old chunk"
        );
        assert_eq!(replaced.unwrap().non_air_count(), 1);
        assert_eq!(store.get(pos).unwrap().non_air_count(), 0);
    }

    #[test]
    fn store_len_contains_remove() {
        let mut store = ChunkStore::default();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);

        let pos = ChunkPos { x: -3, z: 7 };
        assert!(!store.contains(pos));

        store.insert(Chunk::new(pos));
        assert!(store.contains(pos));
        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());

        assert!(store.remove(pos).is_some());
        assert!(!store.contains(pos));
        assert!(store.remove(pos).is_none());
        assert!(store.is_empty());
    }

    #[test]
    fn store_get_or_default_creates_once() {
        let mut store = ChunkStore::default();
        let pos = ChunkPos { x: 5, z: 5 };

        store
            .get_or_default(pos)
            .set_block(block_in(pos, 0, 0, 0), stone());

        assert_eq!(
            store.get_or_default(pos).non_air_count(),
            1,
            "second access must reuse the same chunk"
        );
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn store_get_or_insert_with_uses_closure_once() {
        let mut store = ChunkStore::default();
        let pos = ChunkPos { x: 0, z: 0 };

        let mut filled = Chunk::new(pos);
        filled.set_block(block_in(pos, 3, 3, 3), stone());

        store.get_or_insert_with(pos, || filled);
        assert_eq!(store.get(pos).unwrap().non_air_count(), 1);

        let mut called = false;
        store.get_or_insert_with(pos, || {
            called = true;
            Chunk::new(pos)
        });
        assert!(!called, "closure must not run when chunk already exists");
    }

    #[test]
    fn store_iter_yields_all_chunks() {
        let mut store = ChunkStore::default();
        for x in 0..3 {
            store.insert(Chunk::new(ChunkPos { x, z: 0 }));
        }

        let mut positions: Vec<i32> = store.iter().map(|(pos, _)| pos.x).collect();
        positions.sort_unstable();
        assert_eq!(positions, vec![0, 1, 2]);
    }

    #[test]
    fn store_get_mut_writes_through() {
        let mut store = ChunkStore::default();
        let pos = ChunkPos { x: 0, z: 0 };
        store.insert(Chunk::new(pos));

        if let Some(chunk) = store.get_mut(pos) {
            chunk.set_block(BlockPos::new(0, 0, 0), stone());
        }

        assert_eq!(store.get(pos).unwrap().non_air_count(), 1);
        assert!(store.get_mut(ChunkPos { x: 99, z: 99 }).is_none());
    }
}
