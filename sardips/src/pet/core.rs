use bevy::prelude::*;
use fact_db::EntityFactDatabase;
use sardips_core::age_core::Age;
use sardips_core::mood_core::MoodCategoryHistory;
use sardips_core::name::{EntityName, SpeciesName};
use sardips_core::pet_core::PetKind;
use serde::{Deserialize, Serialize};
use shared_deps::moonshine_save::save::Save;

use super::mood::Mood;
use crate::simulation::Simulated;
use crate::thinking::ThinkerBundle;
use sardips_core::{mood_core::MoodCategory, velocity::Speed};

#[derive(Debug, Component, Default, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Pet;

#[derive(Bundle, Default)]
pub struct PetBundle {
    pub pet: Pet,
    pub species_name: SpeciesName,
    pub name: EntityName,
    pub mood_category: MoodCategory,
    pub mood_category_history: MoodCategoryHistory,
    pub speed: Speed,
    pub mood: Mood,
    pub simulated: Simulated,
    pub fact_db: EntityFactDatabase,
    pub think: ThinkerBundle,
    pub kind: PetKind,
    pub age: Age,
    pub transform: Transform,
    pub save: Save,
}
