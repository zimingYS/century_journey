use bevy::prelude::SystemSet;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,
    Interaction,
    Player,
    Domain,
    Ui,
}
