use bevy::prelude::*;

pub struct SardexPlugin;

impl Plugin for SardexPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
pub struct SardexEntry {
    pub species_key: String,
    pub entry_key: String,
}
