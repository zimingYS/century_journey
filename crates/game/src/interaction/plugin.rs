use crate::interaction::target::{PlayerTarget, update_player_target};
use crate::sets::GameSet;
use bevy::prelude::*;

pub struct GameInteractionPlugin;

impl Plugin for GameInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerTarget>()
            .add_systems(Update, update_player_target.in_set(GameSet::Interaction));
    }
}
