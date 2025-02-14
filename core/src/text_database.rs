use std::{collections::HashMap, str::FromStr};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use shared_deps::bevy_common_assets::ron::RonAssetPlugin;
use shared_deps::rand::seq::SliceRandom;
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
        if key.is_empty() {
            return MISSING_TEXT.clone();
        }
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
        let mut rng = shared_deps::rand::thread_rng();
        self.default_given_names_keys.choose(&mut rng).unwrap()
    }

    pub fn random_surname_key(&self) -> &str {
        let mut rng = shared_deps::rand::thread_rng();
        self.default_surnames_keys.choose(&mut rng).unwrap()
    }

    fn populate_default_name_keys(&mut self) {
        // English is the base language, so we can just use that to find the default name keys
        for key in self.values.get(&Language::English).unwrap().keys() {
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

pub enum WritingSystems {
    Other,
    Korean,
}

pub fn get_writing_system(text: &str) -> WritingSystems {
    if let Some(c) = text.chars().next() {
        // Hangul Symbol Range: 0xAC00 ~ 0xD7A3
        // Hangul Jamo Range: 0x1100 ~ 0x11FF
        // Hangul Compatibility Jamo Range: 0x3130 ~ 0x318F
        // Hangul Jamo Extended-A Range: 0xA960 ~ 0xA97F
        // Hangul Jamo Extended-B Range: 0xD7B0 ~ 0xD7FF
        const RANGES: [std::ops::RangeInclusive<char>; 5] = [
            '\u{AC00}'..='\u{D7A3}',
            '\u{1100}'..='\u{11FF}',
            '\u{3130}'..='\u{318F}',
            '\u{A960}'..='\u{A97F}',
            '\u{D7B0}'..='\u{D7FF}',
        ];

        if RANGES.iter().any(|range| range.contains(&c)) {
            return WritingSystems::Korean;
        }
    }
    WritingSystems::Other
}
