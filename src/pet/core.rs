use core::fmt;

use bevy::prelude::*;
use bevy_turborand::RngComponent;
use serde::{Deserialize, Serialize};

use super::mood::{AutoSetMoodImage, Mood, MoodCategory, MoodCategoryHistory, MoodImages};
use super::pet_ai::PetAi;
use crate::age::Age;
use crate::dynamic_dialogue::EntityFactDatabase;
use crate::name::{EntityName, SpeciesName};
use crate::simulation::Simulated;
use crate::thinking::ThinkerBundle;
use crate::velocity::{MovementDirection, Speed};

#[derive(Debug, Component, Default, Clone, Serialize, Deserialize)]
pub struct Pet;

#[derive(Bundle, Default)]
pub struct PetBundle {
    pub pet: Pet,
    pub species_name: SpeciesName,
    pub name: EntityName,
    pub image_set: MoodImages,
    pub auto_mood_image: AutoSetMoodImage,
    pub mood_category: MoodCategory,
    pub mood_category_history: MoodCategoryHistory,
    pub sprite: SpriteSheetBundle,
    pub speed: Speed,
    pub velocity: MovementDirection,
    pub rng: RngComponent,
    pub pet_ai: PetAi,
    pub mood: Mood,
    pub simulated: Simulated,
    pub fact_db: EntityFactDatabase,
    pub think: ThinkerBundle,
    pub kind: PetKind,
    pub age: Age,
}

#[derive(Debug, Component, Default, Serialize, Deserialize, Clone, Copy)]
pub enum PetKind {
    #[default]
    Blob,
    Normal,
    Object,
    Creature,
    Supernatural,
}

impl fmt::Display for PetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PetKind::Normal => write!(f, "Normal"),
            PetKind::Object => write!(f, "Object"),
            PetKind::Blob => write!(f, "Blob"),
            PetKind::Creature => write!(f, "Creature"),
            PetKind::Supernatural => write!(f, "Supernatural"),
        }
    }
}
