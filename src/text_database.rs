use std::collections::HashMap;

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::GameState;

pub struct TextDatabasePlugin;

impl Plugin for TextDatabasePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<TextDatabase>::new(&["text_database.ron"]))
            .add_systems(OnEnter(GameState::Loading), setup)
            .add_systems(
                Update,
                load_templates.run_if(not(resource_exists::<TextDatabase>)),
            );
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let template_set = TextDatabaseHandle(asset_server.load("text/main.text_database.ron"));
    commands.insert_resource(template_set);
}

fn load_templates(
    mut commands: Commands,
    template_handle: Res<TextDatabaseHandle>,
    mut template_assets: ResMut<Assets<TextDatabase>>,
) {
    if let Some(level) = template_assets.remove(template_handle.0.id()) {
        commands.insert_resource(level);
    }
}

pub mod text_keys {
    pub const MAIN_MENU_TITLE: &str = "main_menu.title";
    pub const MAIN_MENU_PLAY_BUTTON: &str = "main_menu.play_button";
    pub const POOP: &str = "global.poop";
    pub const BACK: &str = "global.back";
    pub const DRAW: &str = "global.draw";
    pub const VICTORY: &str = "global.victory";
    pub const DEFEAT: &str = "global.defeat";
    pub const PET_STARTER_NAME: &str = "pet.starter_name";
    pub const UI_PET_INFO_PANEL_SPECIES: &str = "ui.pet_panel.species";
    pub const UI_PET_INFO_PANEL_AGE: &str = "ui.pet_panel.age";
    pub const UI_PET_PANEL_NO_THOUGHT: &str = "ui.pet_panel.no_thought";
    pub const MINIGAME_SELECT_TIC_TAC_TOE: &str = "minigame_select.tic_tac_toe";
    pub const MINIGAME_SELECT_SPRINT: &str = "minigame_select.sprint";
    pub const MINIGAME_SELECT_HIGHER_LOWER: &str = "minigame_select.higher_lower";
    pub const MINIGAME_SELECT_FOUR_IN_ROW: &str = "minigame_select.four_in_row";
}

#[derive(
    Debug, Component, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Copy, Clone, EnumIter,
)]
pub enum Language {
    #[default]
    English,
    Korean,
}

#[derive(Debug, Resource, Serialize, Deserialize, Asset, TypePath)]
pub struct TextDatabase {
    values: HashMap<Language, HashMap<String, String>>,
}

lazy_static! {
    static ref MISSING_TEXT: String = "".to_string();
}

impl TextDatabase {
    pub fn get(&self, language: Language, key: &str) -> String {
        match self.values.get(&language).unwrap().get(key) {
            Some(val) => val.to_string(),
            None => {
                error!("Key {:?} not found for language {:?}", key, language);
                if language == Language::English {
                    key.to_string()
                } else {
                    self.get(Language::English, key)
                }
            }
        }
    }
}

#[derive(Debug, Resource)]
struct TextDatabaseHandle(Handle<TextDatabase>);
