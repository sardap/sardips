use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    text_database::{Language, TextDatabase},
    GameState,
};

pub struct TextTranslationPlugin;

impl Plugin for TextTranslationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (translate_text, language_changed).run_if(not(in_state(GameState::Loading))),
        );
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((Language::English, SelectedLanguageTag));
}

#[derive(Component)]
pub struct SelectedLanguageTag;

#[derive(Component, Default)]
pub struct KeyText {
    pub keys: HashMap<usize, String>,
}

impl KeyText {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    pub fn with<T: ToString>(mut self, index: usize, key: T) -> Self {
        self.keys.insert(index, key.to_string());
        self
    }

    pub fn set<T: ToString>(&mut self, index: usize, key: T) {
        self.keys.insert(index, key.to_string());
    }
}

fn translate_text(
    text_database: Res<TextDatabase>,
    mut text: Query<&mut Text>,
    changed_keys: Query<(Entity, &KeyText), Or<(Changed<KeyText>, Added<KeyText>)>>,
    selected_language: Query<&Language, With<SelectedLanguageTag>>,
) {
    let selected_language = selected_language.single();

    for (entity, key_text) in changed_keys.iter() {
        if let Ok(mut text) = text.get_mut(entity) {
            for (i, section) in text.sections.iter_mut().enumerate() {
                if let Some(key) = key_text.keys.get(&i) {
                    section.value = text_database.get(*selected_language, key);
                }
            }
        }
    }
}

fn language_changed(
    text_database: Res<TextDatabase>,
    changed_languages: Query<&Language, (With<SelectedLanguageTag>, Changed<Language>)>,
    mut text: Query<&mut Text>,
    keys: Query<(Entity, &KeyText)>,
) {
    let language = match changed_languages.get_single() {
        Ok(language) => *language,
        Err(_) => return,
    };

    debug!("Language changed to {:?}", language);

    for (entity, key) in &keys {
        if let Ok(mut text) = text.get_mut(entity) {
            for (i, section) in text.sections.iter_mut().enumerate() {
                if let Some(key) = key.keys.get(&i) {
                    section.value = text_database.get(language, key);
                }
            }
        }
    }
}
