use bevy::prelude::*;

pub mod renderer;
pub mod animation;

/// 第一人称物品根节点
#[derive(Component)]
pub struct ViewModelRoot;

/// 当前手持物品实体标记
#[derive(Component)]
pub struct HeldItemEntity {
    pub item_identifier: String,
}

/// 动画状态组件
#[derive(Component)]
pub struct ViewModelAnimator {
    pub equip_progress: f32,
    pub swing_progress: f32,
    pub use_progress: f32,
    pub idle_phase: f32,
}

impl Default for ViewModelAnimator {
    fn default() -> Self {
        Self {
            equip_progress: 1.0,
            swing_progress: 0.0,
            use_progress: 0.0,
            idle_phase: 0.0,
        }
    }
}

/// 持久化渲染状态
#[derive(Resource, Default)]
pub struct ViewModelRenderState {
    pub held_entity: Option<Entity>,
    pub hand_entity: Option<Entity>,
    pub current_item: Option<String>,
}

/// 动画类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewAnimation {
    Idle,
    Swing,
    Use,
    Eat,
    Spyglass,
}

pub struct ViewModelPlugin;

impl Plugin for ViewModelPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ViewModelRenderState>()
            .add_systems(Update, (
                renderer::view_model_sync_system,
                animation::view_model_animation_system,
            ));
    }
}