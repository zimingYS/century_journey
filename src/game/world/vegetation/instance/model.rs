//! 定义可持久化树木实例的稳定身份、阶段和模拟时间字段。

use crate::shared::identifier::Identifier;
use crate::shared::voxel::CHUNK_SIZE;
use bevy::math::IVec3;

/// 新建树木实例使用的完整健康值。
const FULL_TREE_HEALTH: u16 = 1_000;

/// 表示树木当前参与权威生命周期规则的语义阶段。
///
/// 存档编解码器会为每个阶段显式映射稳定代码，不能依赖枚举声明顺序作为磁盘协议。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeGrowthStage {
    /// 世界中仍由树苗方块表示、等待首次生长的阶段。
    Sapling,
    /// 已形成较小体素蓝图、等待继续扩展的幼树阶段。
    Young,
    /// 已形成当前完整体素蓝图的成熟树。
    Mature,
}

/// 保存一棵逻辑树的稳定身份与低频模拟状态。
///
/// 树干和树叶仍是普通体素；本类型只在树根所属区块保存一次，不为每个体素创建 ECS
/// 实体。年龄由世界分钟减去出生分钟得到，避免持久化两个会互相漂移的事实源。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeInstance {
    /// 树苗所在的世界方块坐标，也是实例跨阶段保持不变的稳定主键。
    root: IVec3,
    /// 树种的稳定内容标识，不保存会随内容重载变化的运行时 ID。
    species: Identifier,
    /// 生成嵌套树形蓝图的稳定种子。
    shape_seed: u32,
    /// 当前生命周期阶段。
    stage: TreeGrowthStage,
    /// 实例开始被权威模拟的绝对游戏分钟。
    born_at_game_minute: u64,
    /// 当前阶段开始的绝对游戏分钟。
    stage_started_at_game_minute: u64,
    /// 确定性健康刻度；当前规则只创建满健康树，不在渲染帧内更新。
    health: u16,
    /// 最近完成生命周期结算的绝对游戏分钟。
    last_simulated_game_minute: u64,
    /// 下一次低频结算时间；成熟树没有后续规则，因此保持为空。
    next_update_game_minute: Option<u64>,
}

impl TreeInstance {
    /// 为已进入权威模拟的树苗创建实例，并安排首次阶段推进。
    pub(in crate::game::world) fn new_sapling(
        root: IVec3,
        species: Identifier,
        shape_seed: u32,
        game_minute: u64,
        sapling_duration_game_minutes: u64,
    ) -> Self {
        Self {
            root,
            species,
            shape_seed,
            stage: TreeGrowthStage::Sapling,
            born_at_game_minute: game_minute,
            stage_started_at_game_minute: game_minute,
            health: FULL_TREE_HEALTH,
            last_simulated_game_minute: game_minute,
            next_update_game_minute: Some(
                game_minute.saturating_add(sapling_duration_game_minutes),
            ),
        }
    }

    /// 在树苗完整生长成功后创建一棵满健康成熟树。
    #[cfg(test)]
    pub(in crate::game::world) fn new_mature(
        root: IVec3,
        species: Identifier,
        shape_seed: u32,
        game_minute: u64,
    ) -> Self {
        Self {
            root,
            species,
            shape_seed,
            stage: TreeGrowthStage::Mature,
            born_at_game_minute: game_minute,
            stage_started_at_game_minute: game_minute,
            health: FULL_TREE_HEALTH,
            last_simulated_game_minute: game_minute,
            next_update_game_minute: None,
        }
    }

    // 存档恢复逐项接收协议字段，聚合参数会隐藏迁移边界。
    #[allow(clippy::too_many_arguments, reason = "存档字段需要逐项校验")]
    /// 从经过版本迁移的存档字段恢复实例，校验时间顺序并规范化阶段调度。
    pub(in crate::game) fn from_persisted(
        root: IVec3,
        species: Identifier,
        shape_seed: u32,
        stage: TreeGrowthStage,
        born_at_game_minute: u64,
        stage_started_at_game_minute: u64,
        health: u16,
        last_simulated_game_minute: u64,
        next_update_game_minute: Option<u64>,
    ) -> Result<Self, String> {
        if stage_started_at_game_minute < born_at_game_minute {
            return Err("树木阶段开始时间不能早于出生时间".into());
        }
        if last_simulated_game_minute < stage_started_at_game_minute {
            return Err("树木最近结算时间不能早于阶段开始时间".into());
        }
        let next_update_game_minute = match stage {
            TreeGrowthStage::Sapling | TreeGrowthStage::Young => {
                let next_update = next_update_game_minute.unwrap_or(last_simulated_game_minute);
                if next_update < last_simulated_game_minute {
                    return Err("树木下次结算时间不能早于最近结算时间".into());
                }
                Some(next_update)
            }
            TreeGrowthStage::Mature => None,
        };

        Ok(Self {
            root,
            species,
            shape_seed,
            stage,
            born_at_game_minute,
            stage_started_at_game_minute,
            health,
            last_simulated_game_minute,
            next_update_game_minute,
        })
    }

    /// 返回树根世界方块坐标。
    pub(in crate::game) const fn root(&self) -> IVec3 {
        self.root
    }

    /// 返回稳定树种标识。
    pub(in crate::game) fn species(&self) -> &Identifier {
        &self.species
    }

    /// 返回确定性树形种子。
    pub(in crate::game) const fn shape_seed(&self) -> u32 {
        self.shape_seed
    }

    /// 返回当前生命周期阶段。
    pub(in crate::game) const fn stage(&self) -> TreeGrowthStage {
        self.stage
    }

    /// 返回实例开始被模拟的绝对游戏分钟。
    pub(in crate::game) const fn born_at_game_minute(&self) -> u64 {
        self.born_at_game_minute
    }

    /// 返回当前阶段开始的绝对游戏分钟。
    pub(in crate::game) const fn stage_started_at_game_minute(&self) -> u64 {
        self.stage_started_at_game_minute
    }

    /// 返回当前确定性健康刻度。
    pub(in crate::game) const fn health(&self) -> u16 {
        self.health
    }

    /// 返回最近一次生命周期结算分钟。
    pub(in crate::game) const fn last_simulated_game_minute(&self) -> u64 {
        self.last_simulated_game_minute
    }

    /// 返回下一次低频生命周期结算分钟。
    pub(in crate::game) const fn next_update_game_minute(&self) -> Option<u64> {
        self.next_update_game_minute
    }

    /// 判断该实例是否已经到达本次低频生命周期结算时间。
    pub(in crate::game::world) fn is_due(&self, game_minute: u64) -> bool {
        self.next_update_game_minute
            .is_some_and(|next_update| next_update <= game_minute)
    }

    /// 在体素替换成功后把树苗推进为幼树，并安排成熟时间。
    pub(in crate::game::world) fn transition_to_young(
        &mut self,
        game_minute: u64,
        young_duration_game_minutes: u64,
    ) -> Result<(), String> {
        if self.stage != TreeGrowthStage::Sapling {
            return Err("只有树苗阶段能够推进为幼树".into());
        }
        self.stage = TreeGrowthStage::Young;
        self.stage_started_at_game_minute = game_minute;
        self.last_simulated_game_minute = game_minute;
        self.next_update_game_minute =
            Some(game_minute.saturating_add(young_duration_game_minutes));
        Ok(())
    }

    /// 在完整体素替换成功后把幼树推进为成熟树并停止定时更新。
    pub(in crate::game::world) fn transition_to_mature(
        &mut self,
        game_minute: u64,
    ) -> Result<(), String> {
        if self.stage != TreeGrowthStage::Young {
            return Err("只有幼树阶段能够推进为成熟树".into());
        }
        self.stage = TreeGrowthStage::Mature;
        self.stage_started_at_game_minute = game_minute;
        self.last_simulated_game_minute = game_minute;
        self.next_update_game_minute = None;
        Ok(())
    }

    /// 在区块未就绪或空间受阻时延后本次更新，避免固定步内反复扫描。
    pub(in crate::game::world) fn defer_update(
        &mut self,
        game_minute: u64,
        retry_interval_game_minutes: u64,
    ) -> Result<(), String> {
        if self.stage == TreeGrowthStage::Mature {
            return Err("成熟树没有可延后的生命周期更新".into());
        }
        self.last_simulated_game_minute = game_minute;
        self.next_update_game_minute =
            Some(game_minute.saturating_add(retry_interval_game_minutes));
        Ok(())
    }

    /// 返回唯一拥有该实例的根区块坐标，负坐标使用欧几里得除法。
    pub(in crate::game) fn owner_chunk(&self) -> IVec3 {
        let chunk_size = CHUNK_SIZE as i32;
        IVec3::new(
            self.root.x.div_euclid(chunk_size),
            self.root.y.div_euclid(chunk_size),
            self.root.z.div_euclid(chunk_size),
        )
    }
}

#[cfg(test)]
#[path = "../../../../../tests/unit/game/world/vegetation/instance/model.rs"]
mod tests;
