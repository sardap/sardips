use std::{collections::HashSet, fmt};

use bevy::prelude::*;
use bevy_turborand::{GlobalRng, RngComponent};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{
    game_zone::random_point_in_game_zone,
    name::{EntityName, SpeciesName},
    sardip_save::PersistentId,
    simulation::{Simulated, SimulationState},
    text_database::text_keys,
};

use super::template::FoodTemplateDatabase;

pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnFoodEvent>().add_systems(
            Update,
            spawn_pending_food.run_if(in_state(SimulationState::Running)),
        );
    }
}

#[derive(Bundle, Default)]
pub struct FoodBundle {
    pub food: Food,
    pub sensations: FoodSensations,
    pub fill_factor: FoodFillFactor,
    pub species_name: SpeciesName,
    pub name: EntityName,
    pub sprite: SpriteBundle,
    pub simulated: Simulated,
    pub id: PersistentId,
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Hash, EnumIter, Serialize, Deserialize, PartialOrd, Ord,
)]
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

#[derive(Debug, Component, Clone, Default, Serialize, Deserialize)]
pub struct FoodSensations {
    pub values: HashSet<FoodSensationType>,
}

#[derive(Debug, Component, Default, Clone, Serialize, Deserialize)]
pub struct FoodFillFactor(pub f32);

#[derive(Debug, Component, Default)]
pub struct Food;

#[derive(Event)]
pub struct SpawnFoodEvent {
    pub name: String,
}

impl SpawnFoodEvent {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

fn spawn_pending_food(
    mut commands: Commands,
    mut events: EventReader<SpawnFoodEvent>,
    mut global_rng: ResMut<GlobalRng>,
    asset_server: Res<AssetServer>,
    food_db: Res<FoodTemplateDatabase>,
) {
    for event in events.read() {
        let mut rng = RngComponent::from(&mut global_rng);

        if let Some(template) = food_db.get(&event.name) {
            template.spawn(
                &mut commands,
                &asset_server,
                random_point_in_game_zone(&mut rng),
            );
        } else {
            error!("No food template found for {}", event.name);
        }
    }
}
