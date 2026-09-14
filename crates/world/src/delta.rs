//! 世界差异数据

use cj_core::pos::{BlockPos, ChunkPos};
use cj_core::voxel::Voxel;
use std::collections::HashMap;
use std::fmt;

/// 方块变化记录
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockChange {
    /// 修改后的方块
    pub voxel: Voxel,
    /// 是否人为
    pub is_artificial: bool,
}

impl BlockChange {
    /// 玩家放置
    #[inline]
    pub fn placed(voxel: Voxel) -> BlockChange {
        BlockChange {
            voxel,
            is_artificial: true,
        }
    }

    /// 玩家破坏
    #[inline]
    pub fn removed() -> BlockChange {
        BlockChange {
            voxel: Voxel::AIR,
            is_artificial: true,
        }
    }

    /// 自然变化
    #[inline]
    pub fn natural(voxel: Voxel) -> BlockChange {
        BlockChange {
            voxel,
            is_artificial: false,
        }
    }
}

/// 区块改动集
#[derive(Clone)]
pub struct ChunkDelta {
    /// 自身位置
    pos: ChunkPos,
    /// 桶内记录
    blocks: HashMap<BlockPos, BlockChange>,
}

impl ChunkDelta {
    /// 新建指定区块位置的空 Delta
    #[inline]
    pub fn new(pos: ChunkPos) -> Self {
        Self {
            pos,
            blocks: HashMap::new(),
        }
    }

    /// 位置
    #[inline]
    pub fn pos(&self) -> ChunkPos {
        self.pos
    }

    /// 写入记录
    #[inline]
    pub fn set(&mut self, pos: BlockPos, block: BlockChange) {
        debug_assert_eq!(
            pos.chunk_pos(),
            self.pos,
            "block {:?} does not belong to chunk {:?}",
            pos,
            self.pos,
        );

        self.blocks.insert(pos, block);
    }

    /// 读取记录
    #[inline]
    pub fn get(&self, pos: BlockPos) -> Option<&BlockChange> {
        debug_assert_eq!(
            pos.chunk_pos(),
            self.pos,
            "block {:?} does not belong to chunk {:?}",
            pos,
            self.pos,
        );

        self.blocks.get(&pos)
    }

    /// 删除记录
    #[inline]
    pub fn remove(&mut self, pos: BlockPos) -> Option<BlockChange> {
        debug_assert_eq!(
            pos.chunk_pos(),
            self.pos,
            "block {:?} does not belong to chunk {:?}",
            pos,
            self.pos,
        );

        self.blocks.remove(&pos)
    }

    /// 检查记录
    #[inline]
    pub fn contains(&self, pos: BlockPos) -> bool {
        debug_assert_eq!(
            pos.chunk_pos(),
            self.pos,
            "block {:?} does not belong to chunk {:?}",
            pos,
            self.pos,
        );

        self.blocks.contains_key(&pos)
    }

    /// 记录数
    #[inline]
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// 是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// 按引用遍历
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&BlockPos, &BlockChange)> {
        self.blocks.iter()
    }

    /// 按值遍历
    #[inline]
    pub fn iter_copied(&self) -> impl Iterator<Item = (BlockPos, BlockChange)> {
        self.blocks.iter().map(|(&pos, &change)| (pos, change))
    }
}

impl fmt::Debug for ChunkDelta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ChunkDelta(pos: ({}, {}), blocks: {})",
            self.pos.x,
            self.pos.z,
            self.blocks.len(),
        )
    }
}

/// 全局增量
#[derive(Default, Clone)]
pub struct PlayerDelta {
    chunks: HashMap<ChunkPos, ChunkDelta>,
}

impl PlayerDelta {
    #[inline]
    pub fn new() -> Self {
        PlayerDelta::default()
    }

    /// 记录一个方块变化，自动归入对应 ChunkDelta
    #[inline]
    pub fn record(&mut self, pos: BlockPos, change: BlockChange) {
        let chunk_pos = pos.chunk_pos();

        self.chunks
            .entry(chunk_pos)
            .or_insert_with(|| ChunkDelta::new(chunk_pos))
            .set(pos, change);
    }

    /// 读取指定位置的记录
    #[inline]
    pub fn get(&self, pos: BlockPos) -> Option<BlockChange> {
        let chunk_pos = pos.chunk_pos();

        self.chunks
            .get(&chunk_pos)
            .and_then(|chunk| chunk.get(pos))
            .copied()
    }

    /// 移除记录
    #[inline]
    pub fn remove(&mut self, pos: BlockPos) -> Option<BlockChange> {
        let chunk_pos = pos.chunk_pos();

        let change = {
            let chunk = self.chunks.get_mut(&chunk_pos)?;
            chunk.remove(pos)
        };

        if self
            .chunks
            .get(&chunk_pos)
            .is_some_and(ChunkDelta::is_empty)
        {
            self.chunks.remove(&chunk_pos);
        }

        change
    }

    /// 指定区块的 Delta
    #[inline]
    pub fn chunk(&self, pos: ChunkPos) -> Option<&ChunkDelta> {
        self.chunks.get(&pos)
    }

    /// 指定区块的可变 Delta
    #[inline]
    pub fn chunk_mut(&mut self, pos: ChunkPos) -> Option<&mut ChunkDelta> {
        self.chunks.get_mut(&pos)
    }

    /// 遍历所有区块
    #[inline]
    pub fn iter_chunks(&self) -> impl Iterator<Item = (&ChunkPos, &ChunkDelta)> {
        self.chunks.iter()
    }

    /// 有记录的区块数
    #[inline]
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// 是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// 记录总数
    #[inline]
    pub fn total_len(&self) -> usize {
        self.chunks.values().map(ChunkDelta::len).sum()
    }

    /// 清空（新建世界或卸载）
    #[inline]
    pub fn clear(&mut self) {
        self.chunks.clear();
    }
}

impl fmt::Debug for PlayerDelta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PlayerDelta(chunks: {})", self.chunks.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stone() -> Voxel {
        Voxel::cube(cj_core::voxel::BlockId::from_raw(1))
    }

    // 构造属于 chunk 的方块坐标
    fn block_in(chunk: ChunkPos, lx: i32, y: i32, lz: i32) -> BlockPos {
        debug_assert!((0..16).contains(&lx) && (0..16).contains(&lz));
        BlockPos::new(chunk.x * 16 + lx, y, chunk.z * 16 + lz)
    }

    // --- BlockChange ---

    #[test]
    fn block_change_constructors_set_origin_flag() {
        let placed = BlockChange::placed(stone());
        assert_eq!(placed.voxel, stone());
        assert!(placed.is_artificial);

        let natural = BlockChange::natural(stone());
        assert_eq!(natural.voxel, stone());
        assert!(!natural.is_artificial);
    }

    #[test]
    fn block_change_removed_is_air_and_artificial() {
        let removed = BlockChange::removed();
        assert_eq!(removed.voxel, Voxel::AIR);
        assert!(removed.is_artificial, "player removal is artificial");
    }

    #[test]
    fn block_change_natural_air_is_distinguishable() {
        // 自然变成空气（如烧毁）与玩家挖掉必须可区分
        let natural = BlockChange::natural(Voxel::AIR);
        assert_ne!(natural.is_artificial, BlockChange::removed().is_artificial);
    }

    // --- ChunkDelta ---

    #[test]
    fn chunk_delta_set_get_round_trip() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut delta = ChunkDelta::new(pos);
        let block = block_in(pos, 1, 2, 3);

        assert!(delta.get(block).is_none());
        assert!(!delta.contains(block));

        delta.set(block, BlockChange::placed(stone()));
        assert!(delta.contains(block));
        assert_eq!(delta.get(block), Some(&BlockChange::placed(stone())));
    }

    #[test]
    fn chunk_delta_overwrite_keeps_latest() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut delta = ChunkDelta::new(pos);
        let block = block_in(pos, 1, 2, 3);

        delta.set(block, BlockChange::placed(stone()));
        delta.set(block, BlockChange::removed());

        assert_eq!(delta.len(), 1, "same position must overwrite, not append");
        assert_eq!(delta.get(block), Some(&BlockChange::removed()));
    }

    #[test]
    fn chunk_delta_remove_returns_value() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut delta = ChunkDelta::new(pos);
        let block = block_in(pos, 4, 5, 6);

        delta.set(block, BlockChange::placed(stone()));
        assert_eq!(delta.remove(block), Some(BlockChange::placed(stone())));
        assert!(!delta.contains(block));
        assert!(delta.remove(block).is_none());
    }

    #[test]
    fn chunk_delta_len_and_is_empty() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut delta = ChunkDelta::new(pos);

        assert!(delta.is_empty());
        assert_eq!(delta.len(), 0);

        delta.set(block_in(pos, 0, 0, 0), BlockChange::placed(stone()));
        delta.set(block_in(pos, 1, 0, 0), BlockChange::removed());
        assert_eq!(delta.len(), 2);
        assert!(!delta.is_empty());
    }

    #[test]
    fn chunk_delta_iter_yields_all() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut delta = ChunkDelta::new(pos);
        delta.set(block_in(pos, 0, 0, 0), BlockChange::placed(stone()));
        delta.set(block_in(pos, 1, 0, 0), BlockChange::removed());

        let mut by_x: Vec<i32> = delta.iter().map(|(p, _)| p.x).collect();
        by_x.sort_unstable();
        assert_eq!(by_x, vec![0, 1]);

        let mut by_x_copied: Vec<i32> = delta.iter_copied().map(|(p, _)| p.x).collect();
        by_x_copied.sort_unstable();
        assert_eq!(by_x_copied, vec![0, 1]);
    }

    #[test]
    fn chunk_delta_pos_is_preserved() {
        let pos = ChunkPos { x: -3, z: 4 };
        let delta = ChunkDelta::new(pos);
        assert_eq!(delta.pos(), pos);
    }

    // --- PlayerDelta ---

    #[test]
    fn player_delta_buckets_by_chunk() {
        let mut delta = PlayerDelta::default();

        delta.record(
            block_in(ChunkPos { x: 0, z: 0 }, 0, 0, 0),
            BlockChange::placed(stone()),
        );
        delta.record(
            block_in(ChunkPos { x: 1, z: 0 }, 0, 0, 0),
            BlockChange::placed(stone()),
        );
        delta.record(
            block_in(ChunkPos { x: 0, z: 0 }, 15, 0, 15),
            BlockChange::removed(),
        );

        assert_eq!(delta.chunk_count(), 2, "two distinct chunks touched");
        assert_eq!(delta.total_len(), 3);
        assert_eq!(delta.chunk(ChunkPos { x: 0, z: 0 }).unwrap().len(), 2);
        assert_eq!(delta.chunk(ChunkPos { x: 1, z: 0 }).unwrap().len(), 1);
    }

    #[test]
    fn player_delta_get_spans_buckets() {
        let mut delta = PlayerDelta::default();
        let pos = ChunkPos { x: -2, z: -3 };
        let block = block_in(pos, 5, 7, 9);

        delta.record(block, BlockChange::natural(stone()));
        assert_eq!(delta.get(block), Some(BlockChange::natural(stone())));

        // 未记录的位置（即使是同一个 chunk）必须返回 None
        assert!(delta.get(block_in(pos, 6, 7, 9)).is_none());
    }

    #[test]
    fn player_delta_negative_coordinates_bucket_correctly() {
        let mut delta = PlayerDelta::default();

        // -1 与 -16 同属 chunk -1；-17 进入 chunk -2
        delta.record(BlockPos::new(-1, 0, -1), BlockChange::placed(stone()));
        delta.record(BlockPos::new(-16, 0, -16), BlockChange::placed(stone()));
        delta.record(BlockPos::new(-17, 0, -17), BlockChange::placed(stone()));

        assert_eq!(delta.chunk_count(), 2);
        assert_eq!(
            delta.chunk(ChunkPos { x: -1, z: -1 }).unwrap().len(),
            2,
            "-1 and -16 both belong to chunk -1"
        );
        assert_eq!(delta.chunk(ChunkPos { x: -2, z: -2 }).unwrap().len(), 1);
    }

    #[test]
    fn player_delta_remove_drops_empty_bucket() {
        let mut delta = PlayerDelta::default();
        let pos = ChunkPos { x: 3, z: 3 };
        let block = block_in(pos, 1, 0, 1);

        delta.record(block, BlockChange::placed(stone()));
        assert_eq!(delta.chunk_count(), 1);

        assert_eq!(delta.remove(block), Some(BlockChange::placed(stone())));
        assert_eq!(
            delta.chunk_count(),
            0,
            "emptied bucket must be removed, not kept"
        );
        assert!(delta.is_empty());
    }

    #[test]
    fn player_delta_remove_keeps_non_empty_bucket() {
        let mut delta = PlayerDelta::default();
        let pos = ChunkPos { x: 3, z: 3 };
        let a = block_in(pos, 1, 0, 1);
        let b = block_in(pos, 2, 0, 2);

        delta.record(a, BlockChange::placed(stone()));
        delta.record(b, BlockChange::placed(stone()));

        delta.remove(a);
        assert_eq!(delta.chunk_count(), 1, "bucket still holds entry b");
        assert!(delta.get(b).is_some());
    }

    #[test]
    fn player_delta_remove_missing_returns_none() {
        let mut delta = PlayerDelta::default();

        // bucket 不存在
        assert!(delta.remove(BlockPos::new(0, 0, 0)).is_none());
        // bucket 存在但条目不存在
        delta.record(BlockPos::new(0, 0, 0), BlockChange::placed(stone()));
        assert!(delta.remove(BlockPos::new(1, 0, 1)).is_none());
        assert_eq!(delta.chunk_count(), 1, "failed remove must not drop bucket");
    }

    #[test]
    fn player_delta_chunk_mut_writes_through() {
        let mut delta = PlayerDelta::default();
        let pos = ChunkPos { x: 0, z: 0 };
        let block = block_in(pos, 0, 0, 0);

        delta.record(block, BlockChange::placed(stone()));
        delta.chunk_mut(pos).unwrap().remove(block);

        assert!(delta.get(block).is_none());
    }

    #[test]
    fn player_delta_iter_chunks_yields_all() {
        let mut delta = PlayerDelta::default();
        for x in 0..3 {
            delta.record(
                block_in(ChunkPos { x, z: 0 }, 0, 0, 0),
                BlockChange::placed(stone()),
            );
        }

        let mut xs: Vec<i32> = delta.iter_chunks().map(|(pos, _)| pos.x).collect();
        xs.sort_unstable();
        assert_eq!(xs, vec![0, 1, 2]);
    }

    #[test]
    fn player_delta_clear_empties_everything() {
        let mut delta = PlayerDelta::default();
        delta.record(BlockPos::new(0, 0, 0), BlockChange::placed(stone()));
        delta.record(BlockPos::new(20, 0, 20), BlockChange::placed(stone()));

        assert_eq!(delta.chunk_count(), 2);
        delta.clear();

        assert!(delta.is_empty());
        assert_eq!(delta.chunk_count(), 0);
        assert_eq!(delta.total_len(), 0);
    }

    #[test]
    fn player_delta_new_equals_default() {
        let mut a = PlayerDelta::new();
        let mut b = PlayerDelta::default();
        a.record(BlockPos::new(0, 0, 0), BlockChange::placed(stone()));
        b.record(BlockPos::new(0, 0, 0), BlockChange::placed(stone()));

        assert_eq!(a.total_len(), b.total_len());
        assert!(a.chunk(ChunkPos { x: 0, z: 0 }).is_some());
        assert!(b.chunk(ChunkPos { x: 0, z: 0 }).is_some());
    }
}
