use std::collections::HashMap;

use bevy::prelude::*;
use shared_deps::regex::Regex;

use crate::{
    GameState,
    text_database::{Language, TextDatabase},
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

#[derive(PartialEq, Eq)]
pub enum KeyString {
    Direct(String),
    Format(String),
    Value((String, Vec<String>)),
}

lazy_static! {
    static ref FORMAT_RE: Regex = Regex::new(r"~(.*?)~").unwrap();
}

impl KeyString {
    pub fn direct<T: ToString>(key: T) -> Self {
        KeyString::Direct(key.to_string())
    }

    pub fn format<T: ToString>(str: T) -> Self {
        KeyString::Format(str.to_string())
    }

    pub fn value<T: ToString>(key: T, values: &[T]) -> Self {
        KeyString::Value((
            key.to_string(),
            values.iter().map(|v| v.to_string()).collect(),
        ))
    }

    pub fn resolve_string(&self, text_database: &TextDatabase, language: Language) -> String {
        match self {
            KeyString::Direct(key) => text_database.get(language, key),
            KeyString::Format(str) => {
                let result = FORMAT_RE
                    .replace_all(str, |caps: &shared_deps::regex::Captures| {
                        let key = caps.get(1).unwrap().as_str();
                        text_database.get(language, key)
                    })
                    .to_string();
                result
            }
            KeyString::Value((key, values)) => {
                let mut result = text_database.get(language, key);
                for (i, value) in values.iter().enumerate() {
                    result = result.replace(&format!("{{{}}}", i), value);
                }
                result
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::text_database::{Language, TextDatabase};

    #[test]
    fn test_resolve_format_string() {
        let mut text_db = TextDatabase::default();
        text_db.values.insert(
            Language::English,
            vec![
                ("global.foo".to_string(), "foo".to_string()),
                ("global.bar".to_string(), "bar".to_string()),
            ]
            .into_iter()
            .collect(),
        );
        text_db.values.insert(
            Language::Korean,
            vec![("global.bar".to_string(), "바".to_string())]
                .into_iter()
                .collect(),
        );

        let string = "~global.foo~: ~global.bar~";

        let key_string = super::KeyString::Format(string.to_string());

        assert_eq!(
            key_string.resolve_string(&text_db, Language::English),
            "foo: bar"
        );
        assert_eq!(
            key_string.resolve_string(&text_db, Language::Korean),
            "foo: 바"
        );
    }
}

#[derive(Component, Default)]
pub struct KeyText {
    pub keys: HashMap<usize, KeyString>,
}

impl KeyText {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    pub fn with<T: ToString>(mut self, index: usize, key: T) -> Self {
        self.keys.insert(index, KeyString::Direct(key.to_string()));
        self
    }

    pub fn with_format<T: ToString>(mut self, index: usize, str: T) -> Self {
        self.keys.insert(index, KeyString::Format(str.to_string()));
        self
    }

    pub fn with_value<T: ToString>(mut self, index: usize, key: T, values: &[T]) -> Self {
        self.keys.insert(
            index,
            KeyString::Value((
                key.to_string(),
                values.iter().map(|v| v.to_string()).collect(),
            )),
        );
        self
    }

    pub fn set<T: ToString>(&mut self, index: usize, key: T) {
        self.keys.insert(index, KeyString::Direct(key.to_string()));
    }

    pub fn set_section(&mut self, index: usize, key: KeyString) {
        self.keys.insert(index, key);
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
                    section.value = key.resolve_string(&text_database, *selected_language);
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
    let selected_language = match changed_languages.get_single() {
        Ok(language) => *language,
        Err(_) => return,
    };

    debug!("Language changed to {:?}", selected_language);

    for (entity, key) in &keys {
        if let Ok(mut text) = text.get_mut(entity) {
            for (i, section) in text.sections.iter_mut().enumerate() {
                if let Some(key) = key.keys.get(&i) {
                    section.value = key.resolve_string(&text_database, selected_language);
                }
            }
        }
    }
}
