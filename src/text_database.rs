use std::{collections::HashMap, str::FromStr};

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use rand::{seq::SliceRandom, Rng};
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
    let handle = TextDatabaseHandle(asset_server.load("text/main.text_database.ron"));
    commands.insert_resource(handle);
}

fn load_templates(
    mut commands: Commands,
    template_handle: Res<TextDatabaseHandle>,
    mut assets: ResMut<Assets<TextDatabase>>,
) {
    if let Some(mut text_db) = assets.remove(template_handle.0.id()) {
        text_db.populate_default_name_keys();
        commands.insert_resource(text_db);
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

    pub const LOVES: &str = "global.loves";
    pub const LIKES: &str = "global.likes";
    pub const NEUTRAL: &str = "global.neutral";
    pub const DISLIKES: &str = "global.dislikes";
    pub const HATES: &str = "global.hates";
    pub const DESPISES: &str = "global.despises";

    pub const SPICY: &str = "global.spicy";
    pub const COOL: &str = "global.cool";
    pub const ASTRINGENT: &str = "global.astringent";
    pub const UMAMI: &str = "global.umami";
    pub const FATTY: &str = "global.fatty";
    pub const SOUR: &str = "global.sour";
    pub const BITTER: &str = "global.bitter";
    pub const SWEET: &str = "global.sweet";
    pub const SALTY: &str = "global.salty";
    pub const CRUNCHY: &str = "global.crunchy";
    pub const CREAMY: &str = "global.creamy";
    pub const FIZZY: &str = "global.fizzy";
    pub const JUICY: &str = "global.juicy";
    pub const TENDER: &str = "global.tender";
    pub const DRY: &str = "global.dry";
    pub const ELASTIC: &str = "global.elastic";

    pub const UI_PET_INFO_PANEL_SPECIES: &str = "ui.pet_panel.species";
    pub const UI_PET_INFO_PANEL_AGE: &str = "ui.pet_panel.age";
    pub const UI_PET_PANEL_NO_THOUGHT: &str = "ui.pet_panel.no_thought";
    pub const MINIGAME_SELECT_TIC_TAC_TOE: &str = "minigame_select.tic_tac_toe";
    pub const MINIGAME_SELECT_SPRINT: &str = "minigame_select.sprint";
    pub const MINIGAME_SELECT_HIGHER_LOWER: &str = "minigame_select.higher_lower";
    pub const MINIGAME_SELECT_FOUR_IN_ROW: &str = "minigame_select.four_in_row";

    pub const DIPDEX_FOOD_SENSATION_TITLE: &str = "dipdex.food_sensation_title";
    pub const DIPDEX_STOMACH_SIZE: &str = "dipdex.stomach_size";
    pub const DIPDEX_SPEED_TITLE: &str = "dipdex.speed_title";
    pub const DIPDEX_POOP_TITLE: &str = "dipdex.poop_title";
    pub const DIPDEX_CARES_ABOUT_CLEANLINESS: &str = "dipdex.cares_about_cleanliness";
    pub const DIPDEX_DOES_NOT_CARE_ABOUT_CLEANLINESS: &str =
        "dipdex.does_not_care_about_cleanliness";
    pub const DIPDEX_DOES_NOT_POOP: &str = "dipdex.does_not_poop";
    pub const DIPDEX_DESCRIPTION_HEADER: &str = "dipdex.description_header";
    pub const DIPDEX_DESCRIPTION_DOES_NOT_EXIST: &str = "dipdex.description_does_not_exist";
    pub const DIPDEX_STATS_HEADER: &str = "dipdex.stats_header";
    pub const DIPDEX_SPECIES_TITLE: &str = "dipdex.species_title";
}

#[derive(
    Debug, Component, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Copy, Clone, EnumIter,
)]
pub enum Language {
    #[default]
    English,
    Korean,
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "English" => Ok(Language::English),
            "Korean" => Ok(Language::Korean),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

#[derive(Debug, Resource, Serialize, Deserialize, Asset, TypePath, Default)]
pub struct TextDatabase {
    pub values: HashMap<Language, HashMap<String, String>>,
    #[serde(skip)]
    pub default_given_names_keys: Vec<String>,
    #[serde(skip)]
    pub default_surnames_keys: Vec<String>,
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

    pub fn exists(&self, key: &str) -> bool {
        self.values
            .values()
            .any(|language_map| language_map.contains_key(key))
    }

    pub fn random_given_name_key(&self) -> &str {
        let mut rng = rand::thread_rng();
        self.default_given_names_keys.choose(&mut rng).unwrap()
    }

    pub fn random_surname_key(&self) -> &str {
        let mut rng = rand::thread_rng();
        self.default_surnames_keys.choose(&mut rng).unwrap()
    }

    fn populate_default_name_keys(&mut self) {
        // English is the base language, so we can just use that to find the default name keys
        for (key, _) in self.values.get(&Language::English).unwrap() {
            if key.starts_with("names.default.given") {
                self.default_given_names_keys.push(key.clone());
            } else if key.starts_with("names.default.surname") {
                self.default_surnames_keys.push(key.clone());
            }
        }
    }
}

#[derive(Debug, Resource)]
struct TextDatabaseHandle(Handle<TextDatabase>);
