use std::collections::HashSet;

use bevy::{prelude::*, utils::HashMap};
use shared_deps::serde::{Deserialize, Serialize};
use shared_deps::{bevy_common_assets::ron::RonAssetPlugin, bevy_kira_audio::AudioSource};

use crate::{text_database::Language, GameState};

pub struct TranslateWordBankPlugin;

impl Plugin for TranslateWordBankPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<WordBank>::new(&["word_banks.ron"]))
            .add_systems(OnEnter(GameState::Loading), setup)
            .add_systems(
                Update,
                load_templates.run_if(not(resource_exists::<WordBank>)),
            );
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = WordBankHandle(asset_server.load("translate/main.word_bank.ron"));
    commands.insert_resource(handle);
}

fn load_templates(
    mut commands: Commands,
    template_handle: Res<WordBankHandle>,
    mut assets: ResMut<Assets<WordBank>>,
) {
    if let Some(word_bank) = assets.remove(template_handle.0.id()) {
        info!("Inserting word bank as resource");
        commands.insert_resource(word_bank);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Word {
    pub word: String,
    pub audio_path: Option<String>,
    pub sentences: Vec<String>,
    #[serde(skip)]
    pub audio: Option<Handle<AudioSource>>,
}

impl Word {
    fn start_load(&mut self, asset_server: &AssetServer) {
        if let Some(path) = &self.audio_path {
            self.audio = Some(asset_server.load(path));
        }
    }

    fn loading_complete(&self, asset_server: &AssetServer) -> bool {
        self.audio
            .as_ref()
            .map(|handle| asset_server.is_loaded_with_dependencies(handle))
            .unwrap_or(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum WordKind {
    #[default]
    Noun,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WordSet {
    pub id: String,
    pub words: HashMap<Language, Word>,
    pub kind: WordKind,
    pub image_paths: Vec<String>,
    #[serde(skip)]
    pub images: Vec<Handle<Image>>,
}

impl WordSet {
    fn start_load(&mut self, asset_server: &AssetServer) {
        self.images = self
            .image_paths
            .iter()
            .map(|path| asset_server.load(path))
            .collect();

        self.words
            .values_mut()
            .for_each(|word| word.start_load(asset_server));
    }

    fn loading_complete(&self, asset_server: &AssetServer) -> bool {
        self.images
            .iter()
            .all(|handle| asset_server.is_loaded_with_dependencies(handle))
            && self
                .words
                .values()
                .all(|word| word.loading_complete(asset_server))
    }
}

#[derive(Debug, Resource, Serialize, Deserialize, Asset, TypePath, Default)]
pub struct WordBank {
    pub sets: Vec<WordSet>,
}

impl WordBank {
    pub fn start_loading(&mut self, asset_server: &AssetServer) {
        self.sets
            .iter_mut()
            .for_each(|set| set.start_load(asset_server));

        let mut ids = HashSet::new();
        for set in &self.sets {
            if !ids.insert(&set.id) {
                panic!("Duplicate word set id: {}", set.id);
            }
        }
    }

    pub fn loading_complete(&self, asset_server: &AssetServer) -> bool {
        self.sets
            .iter()
            .all(|set| set.loading_complete(asset_server))
    }

    pub fn set_for_languages(&self, language: &[Language]) -> Vec<WordSet> {
        let mut sets = vec![];
        for set in &self.sets {
            if language.iter().all(|lang| set.words.contains_key(lang)) {
                sets.push(set.clone());
            }
        }
        sets
    }
}

#[derive(Debug, Resource)]
struct WordBankHandle(Handle<WordBank>);
