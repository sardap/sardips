use crate::{
    GameState,
    assets::{
        AudioAssets, BackgroundTexturesAssets, DipdexImageAssets, EndlessShooterAssets, FontAssets,
        FourInRowAssets, GameImageAssets, HigherLowerAssets, TicTacToeAssets, TranslateAssets,
        ViewScreenImageAssets,
    },
};
use bevy::prelude::*;
use bevy_asset_loader::loading_state::{
    LoadingState, LoadingStateAppExt, config::ConfigureLoadingState,
};

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
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
                .load_collection::<TranslateAssets>()
                .load_collection::<DipdexImageAssets>(),
        );
    }
}
