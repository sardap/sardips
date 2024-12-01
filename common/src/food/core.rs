use std::fmt;

use bevy::{prelude::*, utils::HashSet};
use bevy_turborand::{GlobalRng, RngComponent};
use moonshine_save::save::Save;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{
    game_zone::random_point_in_game_zone,
    name::{EntityName, SpeciesName},
    simulation::{Simulated, SimulationState},
};
use text_keys;

use super::{template::FoodTemplateDatabase, view::spawn_food_view};

pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnFoodEvent>()
            .register_type::<Food>()
            .register_type::<FoodSensationType>()
            .register_type::<HashSet<FoodSensationType>>()
            .register_type_data::<HashSet<FoodSensationType>, ReflectSerialize>()
            .register_type_data::<HashSet<FoodSensationType>, ReflectDeserialize>()
            .register_type::<FoodSensations>()
            .register_type::<FoodFillFactor>()
            .add_systems(
                Update,
                (spawn_pending_food, spawn_food_view).run_if(in_state(SimulationState::Running)),
            );
    }
}

#[derive(Bundle, Default)]
pub struct FoodBundle {
    pub food: Food,
    pub location: Transform,
    pub sensations: FoodSensations,
    pub fill_factor: FoodFillFactor,
    pub species_name: SpeciesName,
    pub name: EntityName,
    pub simulated: Simulated,
    pub save: Save,
}

// TODO: Implement despawn_food_view
// fn despawn_food_view() {}

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

#[derive(Debug, Component, Default, Reflect)]
#[reflect(Component)]
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
    food_db: Res<FoodTemplateDatabase>,
) {
    for event in events.read() {
        let mut rng = RngComponent::from(&mut global_rng);

        if let Some(template) = food_db.get(&event.name) {
            template.spawn(&mut commands, random_point_in_game_zone(&mut rng));
        } else {
            error!("No food template found for {}", event.name);
        }
    }
}
