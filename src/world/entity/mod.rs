pub mod dropped_item;

use bevy::prelude::*;

pub struct EntityPlugin;
impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) { app
        .add_plugins(dropped_item::DroppedItemPlugin); 
    }
}
