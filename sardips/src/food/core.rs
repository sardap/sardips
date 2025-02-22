use bevy::{prelude::*, utils::HashSet};
use sardips_core::food_core::{
    FoodFillFactor, FoodSensationType, FoodSensations, FoodTemplateDatabase,
};
use sardips_core::name::{EntityName, SpeciesName};
use shared_deps::bevy_turborand::{GlobalRng, RngComponent};
use shared_deps::moonshine_save::save::Save;

use crate::{
    game_zone::random_point_in_game_zone,
    simulation::{Simulated, SimulationState},
};

use super::template::spawn_food;
use super::view::spawn_food_view;

pub struct FoodPlugin;

impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnFoodEvent>()
            .register_type::<Food>()
            .register_type::<FoodSensationType>()
            .register_type::<std::collections::HashSet<String>>()
            .register_type::<HashSet<FoodSensationType>>()
            .register_type_data::<HashSet<FoodSensationType>, ReflectSerialize>()
            .register_type_data::<HashSet<FoodSensationType>, ReflectDeserialize>()
            .register_type::<FoodSensations>()
            .register_type::<FoodFillFactor>()
            .register_type::<FoodDiscoveredEntries>()
            .add_systems(
                Update,
                (add_starting_food_discovered_entries, spawn_pending_food)
                    .run_if(resource_exists::<FoodTemplateDatabase>),
            )
            .add_systems(
                Update,
                spawn_food_view.run_if(in_state(SimulationState::Running)),
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
            spawn_food(template, &mut commands, random_point_in_game_zone(&mut rng));
        } else {
            error!("No food template found for {}", event.name);
        }
    }
}

#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct FoodDiscoveredEntries {
    pub entries: std::collections::HashSet<String>,
}

fn add_starting_food_discovered_entries(
    food_db: Res<FoodTemplateDatabase>,
    global_rng: ResMut<GlobalRng>,
    mut added: Query<&mut FoodDiscoveredEntries, Added<FoodDiscoveredEntries>>,
) {
    let rng = global_rng.into_inner();

    for mut discovered in added.iter_mut() {
        while discovered.entries.len() < 10 {
            let to_add = food_db.random(rng);
            discovered.entries.insert(to_add.name.clone());
        }
    }
}
