use core::fmt;

use bevy::prelude::*;
use bevy_turborand::RngComponent;
use serde::{Deserialize, Serialize};

use crate::{assets::FontAssets, text_database::TextDatabase, text_translation::KeyText};

pub struct NamePlugin;

impl Plugin for NamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_name_tag, update_text_color, add_section_to_text)
                .run_if(resource_exists::<FontAssets>),
        );
    }
}

#[derive(Debug, Component, Serialize, Deserialize, Clone, Default)]
pub struct EntityName {
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
}

impl EntityName {
    pub fn new(first_name: impl Into<String>) -> Self {
        Self {
            first_name: first_name.into(),
            middle_name: None,
            last_name: None,
        }
    }

    pub fn random(text_db: &TextDatabase) -> Self {
        let first_name = text_db.random_given_name_key();
        let middle_name = text_db.random_given_name_key();
        let last_name = text_db.random_surname_key();

        Self::new(first_name)
            .with_middle_name(middle_name)
            .with_last_name(last_name)
    }

    pub fn with_middle_name(mut self, middle_name: impl Into<String>) -> Self {
        self.middle_name = Some(middle_name.into());
        self
    }

    pub fn with_last_name(mut self, last_name: impl Into<String>) -> Self {
        self.last_name = Some(last_name.into());
        self
    }

    pub fn full_name(&self) -> String {
        let mut result = String::new();
        result.push_str(&self.first_name);
        if let Some(middle_name) = &self.middle_name {
            result.push_str(" ");
            result.push_str(middle_name);
        }
        if let Some(last_name) = &self.last_name {
            result.push_str(" ");
            result.push_str(last_name);
        }
        result
    }

    pub fn initials(&self) -> String {
        let mut result = String::new();

        if let Some(first_name) = self.first_name.chars().next() {
            result.push(first_name);
        }

        if let Some(middle_name) = &self.middle_name {
            if let Some(middle_name) = middle_name.chars().next() {
                result.push('.');
                result.push(middle_name);
            }
        }

        if let Some(last_name) = &self.last_name {
            if let Some(last_name) = last_name.chars().next() {
                result.push('.');
                result.push(last_name);
            }
        }

        result
    }
}

impl fmt::Display for EntityName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full_name())
    }
}

#[derive(Debug, Component, Serialize, Deserialize, Clone, Default)]
pub struct SpeciesName(pub String);

impl SpeciesName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn key(&self) -> String {
        format!("species.{}", self.0.to_lowercase())
    }
}

#[derive(Debug, Component)]
pub struct NameTag {
    pub font_size: f32,
    pub color: Color,
    pub offset_y: Option<f32>,
}

impl NameTag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_offset_y(mut self, offset_y: f32) -> Self {
        self.offset_y = Some(offset_y);
        self
    }
}

impl Default for NameTag {
    fn default() -> Self {
        Self {
            font_size: 40.0,
            color: Color::BLACK,
            offset_y: None,
        }
    }
}

#[derive(Bundle, Default)]
pub struct NameTagBundle {
    pub text: Text2dBundle,
    pub name_tag: NameTag,
    pub key_text: KeyText,
}

#[derive(Component)]
pub struct HasNameTag {
    pub name_tag_entity: Entity,
}

impl HasNameTag {
    pub fn new(name_tag_entity: Entity) -> Self {
        Self { name_tag_entity }
    }
}

fn add_section_to_text(font_assets: Res<FontAssets>, mut texts: Query<&mut Text, Added<NameTag>>) {
    for mut text in texts.iter_mut() {
        if text.sections.len() > 0 {
            continue;
        }

        text.sections.push(TextSection {
            value: "".to_string(),
            style: TextStyle {
                font: font_assets.main_font.clone(),
                font_size: 40.0,
                color: Color::BLACK,
            },
        });
    }
}

fn update_name_tag(
    names: Query<(&HasNameTag, &EntityName, &Sprite), Or<(Changed<EntityName>, Added<EntityName>)>>,
    mut q_texts: Query<(&mut KeyText, &mut Transform, &NameTag)>,
) {
    for (has_name_tag, name, sprite) in names.iter() {
        if let Ok((mut key, mut transform, name_tag)) =
            q_texts.get_mut(has_name_tag.name_tag_entity)
        {
            let y_offset = if let Some(y) = name_tag.offset_y {
                y
            } else {
                let sprite_height = match sprite.custom_size {
                    Some(size) => size.y * 0.75,
                    None => 50.,
                };
                sprite_height + 0.5
            };

            transform.translation = Vec3::new(0., y_offset, 0.);

            key.set(0, name.first_name.to_owned());
        }
    }
}

fn update_text_color(mut q_texts: Query<(&mut Text, &NameTag), Changed<NameTag>>) {
    for (mut text, name_tag) in q_texts.iter_mut() {
        if text.sections.len() > 0 {
            text.sections[0].style.color = name_tag.color;
        }
    }
}
