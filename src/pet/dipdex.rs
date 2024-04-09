use bevy::prelude::*;

pub struct DipdexPlugin;

impl Plugin for DipdexPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
pub struct SardexEntry {
    pub species_key: String,
    pub entry_key: String,
}
