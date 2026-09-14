//! 自然积分量
//!
//! 记录无法由当前环境瞬时重建的自然累积状态。

use cj_core::field_ref::{FieldRef, SampleValue};
use cj_core::pos::{ChunkPos, ColumnPos};
use std::collections::HashMap;
use std::fmt;

/// 全局自然积分量
#[derive(Default, Clone)]
pub struct NaturalIntegral {
    /// 按 Chunk 分桶
    chunks: HashMap<ChunkPos, ChunkIntegral>,
}

impl NaturalIntegral {
    /// 新建空数据
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录一条积分数据，自动归入对应 Chunk
    #[inline]
    pub fn record(&mut self, column: ColumnPos, field: FieldRef, sample: SampleValue) {
        let chunk_pos = column.chunk_pos();

        self.chunks
            .entry(chunk_pos)
            .or_insert_with(|| ChunkIntegral::new(chunk_pos))
            .set(column, field, sample);
    }

    /// 读取指定列的积分数据
    #[inline]
    pub fn get(&self, column: ColumnPos, field: FieldRef) -> Option<SampleValue> {
        let chunk_pos = column.chunk_pos();

        self.chunks
            .get(&chunk_pos)
            .and_then(|chunk| chunk.get(column, field))
    }

    /// 删除指定列的积分数据
    #[inline]
    pub fn remove(&mut self, column: ColumnPos, field: FieldRef) -> Option<SampleValue> {
        let chunk_pos = column.chunk_pos();

        let value = {
            let chunk = self.chunks.get_mut(&chunk_pos)?;
            chunk.remove(column, field)
        };

        if self
            .chunks
            .get(&chunk_pos)
            .is_some_and(ChunkIntegral::is_empty)
        {
            self.chunks.remove(&chunk_pos);
        }

        value
    }

    /// 指定列是否存在积分记录
    #[inline]
    pub fn contains(&self, column: ColumnPos, field: FieldRef) -> bool {
        let chunk_pos = column.chunk_pos();

        self.chunks
            .get(&chunk_pos)
            .is_some_and(|chunk| chunk.contains(column, field))
    }

    /// 指定 Chunk 的积分数据
    #[inline]
    pub fn chunk(&self, pos: ChunkPos) -> Option<&ChunkIntegral> {
        self.chunks.get(&pos)
    }

    /// 指定 Chunk 的可变积分数据
    #[inline]
    pub fn chunk_mut(&mut self, pos: ChunkPos) -> Option<&mut ChunkIntegral> {
        self.chunks.get_mut(&pos)
    }

    /// 遍历所有 Chunk
    #[inline]
    pub fn iter_chunks(&self) -> impl Iterator<Item = (&ChunkPos, &ChunkIntegral)> {
        self.chunks.iter()
    }

    /// 有记录的 Chunk 数
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
        self.chunks.values().map(ChunkIntegral::len).sum()
    }

    /// 清空
    #[inline]
    pub fn clear(&mut self) {
        self.chunks.clear();
    }
}

impl fmt::Debug for NaturalIntegral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NaturalIntegral(chunks: {}, entries: {})",
            self.chunk_count(),
            self.total_len(),
        )
    }
}

/// Chunk 的自然积分量
#[derive(Clone)]
pub struct ChunkIntegral {
    /// 自身位置
    pos: ChunkPos,

    /// 桶内记录
    columns: HashMap<(ColumnPos, FieldRef), SampleValue>,
}

impl ChunkIntegral {
    /// 新建指定 Chunk 位置的空数据
    #[inline]
    pub fn new(pos: ChunkPos) -> Self {
        Self {
            pos,
            columns: HashMap::new(),
        }
    }

    /// 所属 Chunk 位置
    #[inline]
    pub fn pos(&self) -> ChunkPos {
        self.pos
    }

    /// 写入指定列的积分数据
    #[inline]
    pub fn set(&mut self, column: ColumnPos, field: FieldRef, sample: SampleValue) {
        debug_assert_eq!(
            column.chunk_pos(),
            self.pos,
            "column {:?} does not belong to chunk integral {:?}",
            column,
            self.pos,
        );

        self.columns.insert((column, field), sample);
    }

    /// 读取指定列的积分数据
    #[inline]
    pub fn get(&self, column: ColumnPos, field: FieldRef) -> Option<SampleValue> {
        debug_assert_eq!(
            column.chunk_pos(),
            self.pos,
            "column {:?} does not belong to chunk integral {:?}",
            column,
            self.pos,
        );

        self.columns.get(&(column, field)).copied()
    }

    /// 删除指定列的积分数据
    #[inline]
    pub fn remove(&mut self, column: ColumnPos, field: FieldRef) -> Option<SampleValue> {
        debug_assert_eq!(
            column.chunk_pos(),
            self.pos,
            "column {:?} does not belong to chunk integral {:?}",
            column,
            self.pos,
        );

        self.columns.remove(&(column, field))
    }

    /// 指定列是否存在积分记录
    #[inline]
    pub fn contains(&self, column: ColumnPos, field: FieldRef) -> bool {
        debug_assert_eq!(
            column.chunk_pos(),
            self.pos,
            "column {:?} does not belong to chunk integral {:?}",
            column,
            self.pos,
        );

        self.columns.contains_key(&(column, field))
    }

    /// 记录数量
    #[inline]
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    /// 是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// 遍历所有记录
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = ((ColumnPos, FieldRef), SampleValue)> + '_ {
        self.columns.iter().map(|(&key, &value)| (key, value))
    }
}

impl fmt::Debug for ChunkIntegral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ChunkIntegral(pos: ({}, {}), entries: {})",
            self.pos.x,
            self.pos.z,
            self.columns.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- ChunkIntegral ---

    #[test]
    fn chunk_integral_set_get_round_trip() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut integral = ChunkIntegral::new(pos);
        let column = ColumnPos::new(3, 7);

        assert!(integral.get(column, FieldRef::WaterTable).is_none());

        integral.set(column, FieldRef::WaterTable, SampleValue::Scalar(12.5));
        assert_eq!(
            integral.get(column, FieldRef::WaterTable),
            Some(SampleValue::Scalar(12.5))
        );
    }

    #[test]
    fn chunk_integral_fields_are_independent() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut integral = ChunkIntegral::new(pos);
        let column = ColumnPos::new(1, 1);

        integral.set(column, FieldRef::WaterTable, SampleValue::Scalar(5.0));
        integral.set(column, FieldRef::Snowpack, SampleValue::Scalar(0.3));

        assert_eq!(integral.len(), 2, "same column, different fields");
        assert_eq!(
            integral.get(column, FieldRef::WaterTable),
            Some(SampleValue::Scalar(5.0))
        );
        assert_eq!(
            integral.get(column, FieldRef::Snowpack),
            Some(SampleValue::Scalar(0.3))
        );
    }

    #[test]
    fn chunk_integral_none_sample_is_distinguishable_from_absent() {
        // 核心契约：未记录 => None；记录 SampleValue::None => Some(None)
        let pos = ChunkPos { x: 0, z: 0 };
        let mut integral = ChunkIntegral::new(pos);
        let column = ColumnPos::new(2, 2);

        assert!(integral.get(column, FieldRef::IceThickness).is_none());

        integral.set(column, FieldRef::IceThickness, SampleValue::None);
        assert_eq!(
            integral.get(column, FieldRef::IceThickness),
            Some(SampleValue::None),
            "recorded None must differ from unrecorded"
        );
        assert!(integral.contains(column, FieldRef::IceThickness));
    }

    #[test]
    fn chunk_integral_overwrite_keeps_latest() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut integral = ChunkIntegral::new(pos);
        let column = ColumnPos::new(5, 5);

        integral.set(column, FieldRef::Snowpack, SampleValue::Scalar(1.0));
        integral.set(column, FieldRef::Snowpack, SampleValue::Scalar(2.0));

        assert_eq!(integral.len(), 1);
        assert_eq!(
            integral.get(column, FieldRef::Snowpack),
            Some(SampleValue::Scalar(2.0))
        );
    }

    #[test]
    fn chunk_integral_remove_returns_value() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut integral = ChunkIntegral::new(pos);
        let column = ColumnPos::new(3, 3);

        integral.set(column, FieldRef::Vegetation, SampleValue::Scalar(0.8));
        assert_eq!(
            integral.remove(column, FieldRef::Vegetation),
            Some(SampleValue::Scalar(0.8))
        );
        assert!(!integral.contains(column, FieldRef::Vegetation));
        assert!(integral.remove(column, FieldRef::Vegetation).is_none());

        // 删掉唯一的记录后应为空
        assert!(integral.is_empty());
    }

    #[test]
    fn chunk_integral_iter_yields_all_entries() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut integral = ChunkIntegral::new(pos);

        integral.set(
            ColumnPos::new(0, 0),
            FieldRef::WaterTable,
            SampleValue::Scalar(1.0),
        );
        integral.set(
            ColumnPos::new(1, 0),
            FieldRef::Snowpack,
            SampleValue::Scalar(2.0),
        );

        let entries: Vec<_> = integral.iter().collect();
        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&(
            (ColumnPos::new(0, 0), FieldRef::WaterTable),
            SampleValue::Scalar(1.0)
        )));
        assert!(entries.contains(&(
            (ColumnPos::new(1, 0), FieldRef::Snowpack),
            SampleValue::Scalar(2.0)
        )));
    }

    #[test]
    fn chunk_integral_vector_value_round_trip() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut integral = ChunkIntegral::new(pos);
        let column = ColumnPos::new(4, 4);

        integral.set(
            column,
            FieldRef::FlowVelocity,
            SampleValue::Vector([1.5, -2.5]),
        );
        assert_eq!(
            integral.get(column, FieldRef::FlowVelocity),
            Some(SampleValue::Vector([1.5, -2.5]))
        );
    }

    #[test]
    fn chunk_integral_pos_is_preserved() {
        let pos = ChunkPos { x: -5, z: 8 };
        let integral = ChunkIntegral::new(pos);
        assert_eq!(integral.pos(), pos);
    }

    // --- NaturalIntegral ---

    #[test]
    fn natural_integral_buckets_by_column_chunk() {
        let mut integral = NaturalIntegral::default();

        integral.record(
            ColumnPos::new(0, 0),
            FieldRef::WaterTable,
            SampleValue::Scalar(1.0),
        );
        integral.record(
            ColumnPos::new(15, 15),
            FieldRef::WaterTable,
            SampleValue::Scalar(2.0),
        );
        integral.record(
            ColumnPos::new(16, 0),
            FieldRef::WaterTable,
            SampleValue::Scalar(3.0),
        );

        assert_eq!(integral.chunk_count(), 2, "(0,0) and (15,15) share a chunk");
        assert_eq!(integral.total_len(), 3);
        assert_eq!(integral.chunk(ChunkPos { x: 0, z: 0 }).unwrap().len(), 2);
        assert_eq!(integral.chunk(ChunkPos { x: 1, z: 0 }).unwrap().len(), 1);
    }

    #[test]
    fn natural_integral_get_spans_buckets() {
        let mut integral = NaturalIntegral::default();
        let column = ColumnPos::new(-3, -20);

        integral.record(column, FieldRef::WaterTable, SampleValue::Scalar(7.5));
        assert_eq!(
            integral.get(column, FieldRef::WaterTable),
            Some(SampleValue::Scalar(7.5))
        );
        assert!(integral.get(column, FieldRef::Snowpack).is_none());
    }

    #[test]
    fn natural_integral_negative_columns_bucket_correctly() {
        let mut integral = NaturalIntegral::default();

        // -1 与 -16 同属 chunk -1；-17 进入 chunk -2
        integral.record(
            ColumnPos::new(-1, -1),
            FieldRef::WaterTable,
            SampleValue::Scalar(1.0),
        );
        integral.record(
            ColumnPos::new(-16, -16),
            FieldRef::WaterTable,
            SampleValue::Scalar(2.0),
        );
        integral.record(
            ColumnPos::new(-17, -17),
            FieldRef::WaterTable,
            SampleValue::Scalar(3.0),
        );

        assert_eq!(integral.chunk_count(), 2);
        assert_eq!(integral.chunk(ChunkPos { x: -1, z: -1 }).unwrap().len(), 2);
        assert_eq!(integral.chunk(ChunkPos { x: -2, z: -2 }).unwrap().len(), 1);

        // 负坐标必须能原样取回
        assert_eq!(
            integral.get(ColumnPos::new(-17, -17), FieldRef::WaterTable),
            Some(SampleValue::Scalar(3.0))
        );
    }

    #[test]
    fn natural_integral_none_sample_survives_round_trip() {
        let mut integral = NaturalIntegral::default();
        let column = ColumnPos::new(9, 9);

        assert!(integral.get(column, FieldRef::IceThickness).is_none());

        integral.record(column, FieldRef::IceThickness, SampleValue::None);

        assert_eq!(
            integral.get(column, FieldRef::IceThickness),
            Some(SampleValue::None),
            "recorded None must not collapse into absent"
        );
        assert!(integral.contains(column, FieldRef::IceThickness));
    }

    #[test]
    fn natural_integral_remove_drops_empty_bucket() {
        let mut integral = NaturalIntegral::default();
        let column = ColumnPos::new(4, 4);

        integral.record(column, FieldRef::Vegetation, SampleValue::Scalar(0.5));
        assert_eq!(integral.chunk_count(), 1);

        assert_eq!(
            integral.remove(column, FieldRef::Vegetation),
            Some(SampleValue::Scalar(0.5))
        );
        assert_eq!(integral.chunk_count(), 0, "emptied bucket must be removed");
        assert!(integral.is_empty());
    }

    #[test]
    fn natural_integral_remove_keeps_non_empty_bucket() {
        let mut integral = NaturalIntegral::default();
        let a = ColumnPos::new(1, 1);
        let b = ColumnPos::new(2, 2);

        integral.record(a, FieldRef::WaterTable, SampleValue::Scalar(1.0));
        integral.record(b, FieldRef::WaterTable, SampleValue::Scalar(2.0));

        integral.remove(a, FieldRef::WaterTable);
        assert_eq!(integral.chunk_count(), 1, "bucket still holds column b");
        assert!(integral.get(b, FieldRef::WaterTable).is_some());
    }

    #[test]
    fn natural_integral_remove_missing_returns_none() {
        let mut integral = NaturalIntegral::default();

        // bucket 不存在
        assert!(
            integral
                .remove(ColumnPos::new(0, 0), FieldRef::WaterTable)
                .is_none()
        );

        // bucket 存在但条目不存在
        integral.record(
            ColumnPos::new(0, 0),
            FieldRef::WaterTable,
            SampleValue::Scalar(1.0),
        );
        assert!(
            integral
                .remove(ColumnPos::new(1, 0), FieldRef::WaterTable)
                .is_none()
        );
        assert_eq!(integral.chunk_count(), 1, "failed remove must keep bucket");
    }

    #[test]
    fn natural_integral_chunk_mut_writes_through() {
        let mut integral = NaturalIntegral::default();
        let column = ColumnPos::new(0, 0);

        integral.record(column, FieldRef::WaterTable, SampleValue::Scalar(1.0));
        integral
            .chunk_mut(ChunkPos { x: 0, z: 0 })
            .unwrap()
            .remove(column, FieldRef::WaterTable);

        assert!(integral.get(column, FieldRef::WaterTable).is_none());
    }

    #[test]
    fn natural_integral_iter_chunks_yields_all() {
        let mut integral = NaturalIntegral::default();
        for x in 0..3 {
            integral.record(
                ColumnPos::new(x * 16, 0),
                FieldRef::WaterTable,
                SampleValue::Scalar(x as f32),
            );
        }

        let mut xs: Vec<i32> = integral.iter_chunks().map(|(pos, _)| pos.x).collect();
        xs.sort_unstable();
        assert_eq!(xs, vec![0, 1, 2]);
    }

    #[test]
    fn natural_integral_clear_empties_everything() {
        let mut integral = NaturalIntegral::default();
        integral.record(
            ColumnPos::new(0, 0),
            FieldRef::WaterTable,
            SampleValue::Scalar(1.0),
        );
        integral.record(
            ColumnPos::new(20, 20),
            FieldRef::Snowpack,
            SampleValue::Scalar(2.0),
        );

        assert_eq!(integral.chunk_count(), 2);
        integral.clear();

        assert!(integral.is_empty());
        assert_eq!(integral.chunk_count(), 0);
        assert_eq!(integral.total_len(), 0);
    }

    #[test]
    fn natural_integral_new_equals_default() {
        let a = NaturalIntegral::new();
        let b = NaturalIntegral::default();

        assert!(a.is_empty());
        assert_eq!(a.chunk_count(), b.chunk_count());
        assert_eq!(a.total_len(), b.total_len());
    }

    #[test]
    fn natural_integral_chunk_boundary_is_16() {
        // 分桶边界：相邻一列与跨 16 列必须落在不同 chunk
        let mut integral = NaturalIntegral::default();

        integral.record(
            ColumnPos::new(15, 0),
            FieldRef::WaterTable,
            SampleValue::Scalar(1.0),
        );
        integral.record(
            ColumnPos::new(16, 0),
            FieldRef::WaterTable,
            SampleValue::Scalar(2.0),
        );

        assert_eq!(integral.chunk_count(), 2);
    }
}
