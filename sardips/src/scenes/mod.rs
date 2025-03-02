pub mod dipdex_scene;
pub mod food_buy_scene;
pub mod info_panel;
pub mod load_view_screen;
pub mod main_menu;
pub mod minigame_scene;
pub mod stock_scene;
pub mod template_scene;
pub mod view_screen;

use bevy::prelude::*;
use food_buy_scene::FoodBuyScenePlugin;
use info_panel::InfoPanelPlugin;
use stock_scene::StockScenePlugin;

use self::{
    dipdex_scene::DipdexScenePlugin, load_view_screen::LoadViewScreenPlugin,
    main_menu::MainMenuPlugin, minigame_scene::MinigameScenePlugin, view_screen::ViewScreenPlugin,
};

pub struct GameScenePlugin;

impl Plugin for GameScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InfoPanelPlugin,
            ViewScreenPlugin,
            MinigameScenePlugin,
            MainMenuPlugin,
            LoadViewScreenPlugin,
            DipdexScenePlugin,
            FoodBuyScenePlugin,
            StockScenePlugin,
        ));
    }
}
