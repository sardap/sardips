use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use serde::{Deserialize, Serialize};
use shared_deps::bevy_turborand::DelegatedRng;
use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::pet_core::TemplateSize;

pub struct FoodCorePlugin;

impl Plugin for FoodCorePlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(
    Reflect,
    Debug,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Hash,
    EnumIter,
    Serialize,
    Deserialize,
    PartialOrd,
    Ord,
)]
#[reflect_value(PartialEq, Serialize, Deserialize)]
pub enum FoodSensationType {
    Spicy,
    Cool,
    // A drying, puckering mouthfeel
    Astringent,
    // Often described as savory or meaty
    Umami,
    Fatty,
    Sour,
    Bitter,
    Sweet,
    Salty,
    Crunchy,
    Creamy,
    Fizzy,
    Juicy,
    Tender,
    Dry,
    Elastic,
}

impl FoodSensationType {
    pub fn short_string(&self) -> &'static str {
        match self {
            FoodSensationType::Spicy => "Spi",
            FoodSensationType::Cool => "Coo",
            FoodSensationType::Astringent => "Ast",
            FoodSensationType::Umami => "Uma",
            FoodSensationType::Fatty => "Fat",
            FoodSensationType::Sour => "Sou",
            FoodSensationType::Bitter => "Bit",
            FoodSensationType::Sweet => "Swe",
            FoodSensationType::Salty => "Sal",
            FoodSensationType::Crunchy => "Cru",
            FoodSensationType::Creamy => "Cre",
            FoodSensationType::Fizzy => "Fiz",
            FoodSensationType::Juicy => "Jui",
            FoodSensationType::Tender => "Ten",
            FoodSensationType::Dry => "Dry",
            FoodSensationType::Elastic => "Ela",
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            FoodSensationType::Spicy => text_keys::SPICY,
            FoodSensationType::Cool => text_keys::COOL,
            FoodSensationType::Astringent => text_keys::ASTRINGENT,
            FoodSensationType::Umami => text_keys::UMAMI,
            FoodSensationType::Fatty => text_keys::FATTY,
            FoodSensationType::Sour => text_keys::SOUR,
            FoodSensationType::Bitter => text_keys::BITTER,
            FoodSensationType::Sweet => text_keys::SWEET,
            FoodSensationType::Salty => text_keys::SALTY,
            FoodSensationType::Crunchy => text_keys::CRUNCHY,
            FoodSensationType::Creamy => text_keys::CREAMY,
            FoodSensationType::Fizzy => text_keys::FIZZY,
            FoodSensationType::Juicy => text_keys::JUICY,
            FoodSensationType::Tender => text_keys::TENDER,
            FoodSensationType::Dry => text_keys::DRY,
            FoodSensationType::Elastic => text_keys::ELASTIC,
        }
    }
}

impl fmt::Display for FoodSensationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Component, Clone, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct FoodSensations {
    pub values: HashSet<FoodSensationType>,
}

#[derive(Debug, Component, Default, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct FoodFillFactor(pub f32);

#[derive(Debug, Copy, Clone, EnumIter, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoodSensationRating {
    Loves,
    Likes,
    Neutral,
    Dislikes,
    Hates,
    Despises,
}

impl FoodSensationRating {
    pub fn f32(&self) -> f32 {
        match self {
            FoodSensationRating::Loves => 5.0,
            FoodSensationRating::Likes => 3.0,
            FoodSensationRating::Neutral => 0.0,
            FoodSensationRating::Dislikes => -7.0,
            FoodSensationRating::Hates => -10.0,
            FoodSensationRating::Despises => -100.0,
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            FoodSensationRating::Loves => text_keys::LOVES,
            FoodSensationRating::Likes => text_keys::LIKES,
            FoodSensationRating::Neutral => text_keys::NEUTRAL,
            FoodSensationRating::Dislikes => text_keys::DISLIKES,
            FoodSensationRating::Hates => text_keys::HATES,
            FoodSensationRating::Despises => text_keys::DESPISES,
        }
    }
}

impl From<f32> for FoodSensationRating {
    fn from(value: f32) -> Self {
        for rating in FoodSensationRating::iter() {
            if value >= rating.f32() {
                return rating;
            }
        }

        FoodSensationRating::Despises
    }
}

impl fmt::Display for FoodSensationRating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Component, Serialize, Deserialize, Clone)]
pub struct FoodPreference {
    pub sensation_ratings: HashMap<FoodSensationType, FoodSensationRating>,
}

impl FoodPreference {
    pub fn feeling(&self, sensations: &FoodSensations) -> FoodSensationRating {
        let mut overall_rating = 0.;

        for sensation in sensations.values.iter() {
            if let Some(rating) = self.sensation_ratings.get(sensation) {
                overall_rating += rating.f32();
            }
        }

        overall_rating.into()
    }
}

#[derive(Serialize, Deserialize)]
pub struct FoodTemplate {
    pub name: String,
    pub sensations: HashSet<FoodSensationType>,
    pub texture: String,
    pub texture_size: (u32, u32),
    pub sprite_size: TemplateSize,
    pub fill_factor: f32,
}

#[derive(Resource)]
pub struct FoodTemplateDatabase {
    pub templates: Vec<FoodTemplate>,
}

impl FoodTemplateDatabase {
    pub fn iter(&self) -> impl Iterator<Item = &FoodTemplate> {
        self.templates.iter()
    }

    pub fn get(&self, name: &str) -> Option<&FoodTemplate> {
        self.templates.iter().find(|template| template.name == name)
    }

    pub fn random<R: DelegatedRng>(&self, rng: &mut R) -> &FoodTemplate {
        let index = rng.usize(0..self.templates.len());
        &self.templates[index]
    }
}

#[derive(Asset, Serialize, Deserialize, TypePath)]
pub struct AssetFoodTemplateSet {
    pub templates: Vec<FoodTemplate>,
}
