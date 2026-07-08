use crate::shared::components::camera::FpsCamera;
use crate::shared::identifier::Identifier;
use bevy::prelude::*;

pub mod animation;
pub mod hand_view;
pub mod renderer;

#[derive(Component)]
pub struct ViewModelRoot;

#[derive(Component)]
pub struct ViewModelPart;

#[derive(Component)]
pub struct HeldItemEntity {
    pub item_identifier: Identifier,
}

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

#[derive(Resource, Default)]
pub struct ViewModelRenderState {
    pub held_entity: Option<Entity>,
    pub hand_entity: Option<Entity>,
    pub current_item: Option<Identifier>,
}

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
        app.init_resource::<ViewModelRenderState>().add_systems(
            Update,
            (
                renderer::view_model_sync_system,
                animation::view_model_animation_system,
                view_model_visibility_system,
            )
                .chain(),
        );
    }
}

fn view_model_visibility_system(
    camera_query: Query<&FpsCamera, With<Camera3d>>,
    mut view_model_query: Query<&mut Visibility, With<ViewModelPart>>,
) {
    let is_first_person = camera_query
        .single()
        .map(|camera| camera.is_first_person)
        .unwrap_or(true);
    let target = if is_first_person {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    for mut visibility in &mut view_model_query {
        *visibility = target;
    }
}
