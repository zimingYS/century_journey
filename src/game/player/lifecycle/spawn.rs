//! 玩家权威生成。
//!
//! 本模块只创建 Game 层需要模拟和持久化的玩家状态；相机、骨架、动画和渲染层级
//! 由 Client 在权威实体生成后附加。

use crate::game::crafting::grid::{ActiveCrafting, PlayerCrafting};
use crate::game::inventory::state::InventoryState;
use crate::game::player::flight::components::PlayerFlight;
use crate::game::player::identity::{LocalPlayer, Player, PlayerId};
use crate::game::player::lifecycle::components::{PlayerLifecycle, RespawnPoint};
use crate::game::player::movement::components::{PlayerAim, PlayerMovement, PlayerVelocity};
use crate::game::player::physics::components::{PlayerCollider, PlayerGravity};
use crate::game::player::survival::environment::EnvironmentExposure;
use crate::game::player::survival::health::Health;
use crate::game::player::survival::hunger::FoodUseState;
use crate::game::player::survival::hunger::Hunger;
use crate::game::player::survival::protection::Defense;
use crate::game::player::survival::thirst::{DrinkUseState, Thirst};
use crate::game::simulation::SimulationTransformHistory;
use bevy::prelude::*;

/// 玩家启动阶段。
///
/// Client 表现系统必须在 `Authority` 阶段后运行，避免依赖插件注册的偶然顺序。
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerStartupSet {
    /// 权威玩家实体及其全部 Game 层组件已经创建。
    Authority,
}

/// 生成本地权威玩家实体系统
/// 此部分仅负责数据生成，不进行渲染
pub fn spawn_authoritative_player_system(
    mut commands: Commands,
    players: Query<(), With<LocalPlayer>>,
) {
    if !players.is_empty() {
        return;
    }

    // 玩家初始出生坐标
    let player_transform = Transform::from_xyz(0.0, 70.0, 0.0);

    // 生成玩家基础实体
    let player = commands
        .spawn((
            Player,
            LocalPlayer,
            PlayerAim::default(),
            PlayerGravity::default(),
            PlayerCollider::default(),
            PlayerMovement::default(),
            PlayerVelocity::default(),
            PlayerFlight::default(),
            FoodUseState::default(),
            DrinkUseState::default(),
            Health::default(),
            Hunger::default(),
            Thirst::default(),
            Defense::default(),
            player_transform,
        ))
        .id();

    commands.entity(player).insert((
        PlayerLifecycle::default(),
        RespawnPoint::default(),
        EnvironmentExposure::default(),
        SimulationTransformHistory::new(player_transform),
        PlayerId::LOCAL,
        InventoryState::default(),
        PlayerCrafting::default(),
        ActiveCrafting::default(),
    ));
}
