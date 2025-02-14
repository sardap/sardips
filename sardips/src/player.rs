use bevy::prelude::*;
use shared_deps::moonshine_save::save::Save;

use crate::{
    money::Wallet, pet::dipdex::DipdexDiscoveredEntries, sardip_save::SardipLoadingState,
    stock_market::SharePortfolio,
};

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
    pub share_portfolio: SharePortfolio,
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

    commands.spawn(PlayerBundle {
        share_portfolio: SharePortfolio::new_player_portfolio(),
        ..default()
    });
}
