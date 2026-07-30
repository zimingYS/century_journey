//! 生成不读取世界状态的确定性树体素蓝图，供生成期和运行时共同消费。

use bevy::prelude::IVec3;
use std::collections::HashSet;

/// 构建简单树形所需的尺寸范围。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::game::world) struct TreeBlueprintParameters {
    /// 树干高度闭区间的下界。
    pub trunk_height_min: u8,
    /// 树干高度闭区间的上界。
    pub trunk_height_max: u8,
    /// 树冠半径闭区间的下界。
    pub crown_radius_min: u8,
    /// 树冠半径闭区间的上界。
    pub crown_radius_max: u8,
}

impl TreeBlueprintParameters {
    /// 返回当前世界生成所使用的小树尺寸范围。
    pub const fn generated_tree() -> Self {
        Self {
            trunk_height_min: 4,
            trunk_height_max: 6,
            crown_radius_min: 2,
            crown_radius_max: 3,
        }
    }
}

/// 树形蓝图中的一次不重复体素写入。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::game::world) struct TreeVoxel {
    /// 目标世界方块坐标。
    pub world_pos: IVec3,
    /// 应写入的运行时方块 ID。
    pub block_id: u16,
}

/// 一棵树的完整、确定性且无重复体素计划。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::game::world) struct TreeBlueprint {
    voxels: Vec<TreeVoxel>,
}

impl TreeBlueprint {
    /// 根据树干起点、形状哈希和尺寸范围生成简单树形。
    ///
    /// 树干先写入占位集合，树冠随后只补充未占用坐标，因此树叶不会覆盖树干。
    pub fn generate(
        trunk_anchor: IVec3,
        shape_hash: u32,
        trunk_block_id: u16,
        leaves_block_id: u16,
        parameters: TreeBlueprintParameters,
    ) -> Self {
        debug_assert!(parameters.trunk_height_min <= parameters.trunk_height_max);
        debug_assert!(parameters.crown_radius_min <= parameters.crown_radius_max);

        let trunk_height = choose_dimension(
            parameters.trunk_height_min,
            parameters.trunk_height_max,
            shape_hash,
        ) as i32;
        let crown_radius = choose_dimension(
            parameters.crown_radius_min,
            parameters.crown_radius_max,
            shape_hash >> 8,
        ) as i32;
        let crown_center = trunk_anchor + IVec3::Y * trunk_height;

        let mut occupied = HashSet::new();
        let mut voxels = Vec::new();
        for dy in 0..trunk_height {
            let world_pos = trunk_anchor + IVec3::Y * dy;
            occupied.insert(world_pos);
            voxels.push(TreeVoxel {
                world_pos,
                block_id: trunk_block_id,
            });
        }

        for dx in -crown_radius..=crown_radius {
            for dy in -crown_radius..=crown_radius {
                for dz in -crown_radius..=crown_radius {
                    if dx * dx + dy * dy + dz * dz > crown_radius * crown_radius {
                        continue;
                    }
                    let world_pos = crown_center + IVec3::new(dx, dy, dz);
                    if occupied.insert(world_pos) {
                        voxels.push(TreeVoxel {
                            world_pos,
                            block_id: leaves_block_id,
                        });
                    }
                }
            }
        }

        Self { voxels }
    }

    /// 按稳定的树干优先顺序返回全部体素写入。
    pub fn voxels(&self) -> &[TreeVoxel] {
        &self.voxels
    }
}

fn choose_dimension(minimum: u8, maximum: u8, random_bits: u32) -> u8 {
    let span = u32::from(maximum - minimum) + 1;
    minimum + (random_bits % span) as u8
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/structure/tree.rs"]
mod tests;
