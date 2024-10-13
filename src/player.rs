use bevy::prelude::*;
use moonshine_save::save::Save;

use crate::{money::Wallet, pet::dipdex::DipdexDiscoveredEntries, sardip_save::SardipLoadingState};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .add_systems(OnEnter(SardipLoadingState::Loaded), spawn_player);
    }
}

#[derive(Bundle, Default)]
pub struct PlayerBundle {
    pub player: Player,
    pub wallet: Wallet,
    pub entries: DipdexDiscoveredEntries,
    pub save: Save,
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

fn spawn_player(mut commands: Commands, player: Query<Entity, With<Player>>) {
    if player.iter().next().is_some() {
        return;
    }

    commands.spawn(PlayerBundle::default());
}
