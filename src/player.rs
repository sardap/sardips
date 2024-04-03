use bevy::prelude::*;

use crate::money::Wallet;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Bundle, Default)]
pub struct PlayerBundle {
    pub player: Player,
    pub wallet: Wallet,
}

#[derive(Component, Default)]
pub struct Player;
