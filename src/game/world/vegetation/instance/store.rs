//! 维护按根区块分桶、按根坐标有序的树木实例集合。

use super::TreeInstance;
use bevy::math::IVec3;
use std::collections::HashMap;

/// `WorldState` 内部拥有的树木实例索引。
///
/// 两层 HashMap 提供常数时间定位；生成存档快照时再按根坐标排序，避免遍历顺序进入协议。
#[derive(Debug, Default)]
pub(in crate::game::world) struct TreeInstanceStore {
    chunks: HashMap<IVec3, HashMap<IVec3, TreeInstance>>,
}

impl TreeInstanceStore {
    /// 插入新实例；同一根坐标已存在时保持原值并返回错误。
    pub(in crate::game::world) fn insert(&mut self, instance: TreeInstance) -> Result<(), String> {
        let owner = instance.owner_chunk();
        let root = instance.root();
        let bucket = self.chunks.entry(owner).or_default();
        if bucket.contains_key(&root) {
            return Err(format!("树根 {root:?} 已存在逻辑实例"));
        }
        bucket.insert(root, instance);
        Ok(())
    }

    /// 原子替换一个根区块的实例；归属错误或重复根不会修改旧数据。
    pub(in crate::game::world) fn replace_chunk(
        &mut self,
        chunk_position: IVec3,
        instances: Vec<TreeInstance>,
    ) -> Result<(), String> {
        let mut rebuilt = HashMap::new();
        for instance in instances {
            if instance.owner_chunk() != chunk_position {
                return Err(format!(
                    "树根 {:?} 不属于存档区块 {chunk_position:?}",
                    instance.root()
                ));
            }
            let root = instance.root();
            if rebuilt.insert(root, instance).is_some() {
                return Err(format!("区块 {chunk_position:?} 包含重复树根 {root:?}"));
            }
        }

        if rebuilt.is_empty() {
            self.chunks.remove(&chunk_position);
        } else {
            self.chunks.insert(chunk_position, rebuilt);
        }
        Ok(())
    }

    /// 返回指定根坐标的实例。
    pub(in crate::game::world) fn get(&self, root: IVec3) -> Option<&TreeInstance> {
        let owner = owner_chunk(root);
        self.chunks.get(&owner)?.get(&root)
    }

    /// 返回指定根坐标的可变实例，供生命周期在体素提交后更新阶段。
    pub(in crate::game::world) fn get_mut(&mut self, root: IVec3) -> Option<&mut TreeInstance> {
        let owner = owner_chunk(root);
        self.chunks.get_mut(&owner)?.get_mut(&root)
    }

    /// 返回已到结算时间的有序根坐标，只扫描当前已加载的实例分桶。
    pub(in crate::game::world) fn due_roots(&self, game_minute: u64) -> Vec<IVec3> {
        let mut roots = self
            .chunks
            .values()
            .flat_map(|bucket| bucket.values())
            .filter(|instance| instance.is_due(game_minute))
            .map(TreeInstance::root)
            .collect::<Vec<_>>();
        roots.sort_by_key(|root| (root.x, root.y, root.z));
        roots
    }

    /// 删除指定根坐标的实例，并在分桶为空后释放分桶。
    pub(in crate::game::world) fn remove(&mut self, root: IVec3) -> Option<TreeInstance> {
        let owner = self.get(root)?.owner_chunk();
        let bucket = self.chunks.get_mut(&owner)?;
        let removed = bucket.remove(&root);
        if bucket.is_empty() {
            self.chunks.remove(&owner);
        }
        removed
    }

    /// 克隆指定区块的有序实例快照，供存档和异步任务取得独立所有权。
    pub(in crate::game::world) fn snapshot_chunk(
        &self,
        chunk_position: IVec3,
    ) -> Vec<TreeInstance> {
        let mut instances = self
            .chunks
            .get(&chunk_position)
            .map(|bucket| bucket.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        instances.sort_by_key(|instance| {
            let root = instance.root();
            (root.x, root.y, root.z)
        });
        instances
    }

    /// 取走指定根区块的全部实例，供区块卸载与保存保持同一所有权边界。
    pub(in crate::game::world) fn take_chunk(
        &mut self,
        chunk_position: IVec3,
    ) -> Vec<TreeInstance> {
        let mut instances = self
            .chunks
            .remove(&chunk_position)
            .map(|bucket| bucket.into_values().collect::<Vec<_>>())
            .unwrap_or_default();
        instances.sort_by_key(|instance| {
            let root = instance.root();
            (root.x, root.y, root.z)
        });
        instances
    }
}

fn owner_chunk(root: IVec3) -> IVec3 {
    let chunk_size = crate::shared::voxel::CHUNK_SIZE as i32;
    IVec3::new(
        root.x.div_euclid(chunk_size),
        root.y.div_euclid(chunk_size),
        root.z.div_euclid(chunk_size),
    )
}

#[cfg(test)]
#[path = "../../../../../tests/unit/game/world/vegetation/instance/store.rs"]
mod tests;
