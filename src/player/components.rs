use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerMovement{
    pub movement_speed: f32,
    pub sprint_factor: f32,
    pub jump_force: f32
}

impl Default for PlayerMovement{
    fn default() -> Self{
        Self{
            movement_speed: 15.0,
            sprint_factor: 1.5,
            jump_force: 4.5,
        }
    }
}
