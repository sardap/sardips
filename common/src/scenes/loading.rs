use crate::{
    assets::{
        AudioAssets, BackgroundTexturesAssets, DipdexImageAssets, EndlessShooterAssets, FontAssets,
        FourInRowAssets, GameImageAssets, HigherLowerAssets, TicTacToeAssets,
        ViewScreenImageAssets,
    },
    GameState,
};
use bevy::prelude::*;
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState, LoadingStateAppExt,
};

pub struct LoadingScenePlugin;

impl Plugin for LoadingScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::MainMenu)
                .load_collection::<BackgroundTexturesAssets>()
                .load_collection::<FontAssets>()
                .load_collection::<AudioAssets>()
                .load_collection::<ViewScreenImageAssets>()
                .load_collection::<GameImageAssets>()
                .load_collection::<TicTacToeAssets>()
                .load_collection::<HigherLowerAssets>()
                .load_collection::<FourInRowAssets>()
                .load_collection::<EndlessShooterAssets>()
                .load_collection::<DipdexImageAssets>(),
        );
    }
}
