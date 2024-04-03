use std::{collections::HashSet, fmt};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{name::EntityName, simulation::Simulated};

pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, _: &mut App) {}
}

#[derive(Bundle, Default)]
pub struct FoodBundle {
    pub food: Food,
    pub sensations: FoodSensations,
    pub fill_factor: FoodFillFactor,
    pub name: EntityName,
    pub sprite: SpriteBundle,
    pub simulated: Simulated,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, EnumIter, Serialize, Deserialize)]
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
}

impl fmt::Display for FoodSensationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Component, Clone, Default, Serialize, Deserialize)]
pub struct FoodSensations {
    pub values: HashSet<FoodSensationType>,
}

#[derive(Debug, Component, Default, Clone, Serialize, Deserialize)]
pub struct FoodFillFactor(pub f32);

#[derive(Debug, Component, Default)]
pub struct Food;
