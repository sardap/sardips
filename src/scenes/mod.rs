pub mod info_panel;
pub mod load_view_screen;
pub mod loading;
pub mod main_menu;
pub mod minigame_scene;
pub mod view_screen;

use bevy::prelude::*;

use self::{
    load_view_screen::LoadViewScreenPlugin, loading::LoadingScenePlugin, main_menu::MainMenuPlugin,
    minigame_scene::MinigameScenePlugin, view_screen::ViewScreenPlugin,
};

pub struct GameScenePlugin;

impl Plugin for GameScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ViewScreenPlugin,
            MinigameScenePlugin,
            MainMenuPlugin,
            LoadingScenePlugin,
            LoadViewScreenPlugin,
        ));
    }
}
