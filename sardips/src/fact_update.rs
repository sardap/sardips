use bevy::prelude::*;
use fact_db::{fact_str_hash, EntityFactDatabase, FactDb, GlobalFactDatabase};

use crate::{
    age::Age,
    food::Food,
    name::{EntityName, SpeciesName},
    pet::{
        breeding::ReadyToBreed,
        hunger::{Hunger, Starving},
        mood::{Mood, MoodCategoryHistory, MoodState},
        poop::Poop,
        Pet, PetKind,
    },
    simulation::{SimulationState, SimulationUpdate},
};
use sardips_core::mood_core::MoodCategory;

// https://www.youtube.com/watch?v=tAbBID3N64A&t=20s
// https://www.gdcvault.com/play/1015317/AI-driven-Dynamic-Dialog-through

pub struct FactUpdatePlugin;

impl Plugin for FactUpdatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                update_mood,
                update_overall_mood,
                update_median_mood,
                update_names_facts,
            ),
        )
        .add_systems(
            SimulationUpdate,
            (
                update_pet_count,
                update_hunger_facts,
                update_starving_fact,
                update_food_count,
                update_species_name,
                update_pet_kind,
                update_poop,
                update_age,
                add_ready_to_breed,
                remove_ready_to_breed,
                update_food_exists,
                update_existing_pets,
            )
                .run_if(in_state(SimulationState::Running)),
        );
    }
}

// ************************************************************************

fn update_hunger_facts(mut query: Query<(&mut EntityFactDatabase, &Hunger), Changed<Hunger>>) {
    for (mut fact_db, hunger) in &mut query {
        fact_db.0.add("Hunger", hunger.filled_percent());
    }
}

fn update_starving_fact(
    mut query: Query<(&mut EntityFactDatabase, Option<&Starving>), With<Hunger>>,
) {
    for (mut fact_db, starving) in query.iter_mut() {
        if starving.is_some() {
            fact_db.0.add("IsStarving", 1.0);
        } else {
            fact_db.0.remove("IStarving");
        }
    }
}

fn update_food_count(mut fact_db: ResMut<GlobalFactDatabase>, query: Query<Entity, With<Food>>) {
    fact_db.0.add("FoodCount", query.iter().count() as f32);
}

fn update_species_name(
    mut query: Query<(&mut EntityFactDatabase, &SpeciesName), Changed<SpeciesName>>,
) {
    for (mut fact_db, name) in query.iter_mut() {
        fact_db.0.add("Species", fact_str_hash(&name.0));
    }
}

fn update_pet_kind(mut query: Query<(&mut EntityFactDatabase, &PetKind), Changed<PetKind>>) {
    for (mut fact_db, kind) in query.iter_mut() {
        fact_db.0.add("Kind", fact_str_hash(kind.to_string()));
    }
}

fn update_poop(mut fact_db: ResMut<GlobalFactDatabase>, query: Query<Entity, With<Poop>>) {
    fact_db.0.add("PoopCount", query.iter().count() as f32);
}

fn update_age(mut query: Query<(&mut EntityFactDatabase, &Age), Changed<Age>>) {
    for (mut fact_db, age) in query.iter_mut() {
        fact_db.0.add("Age", (age.as_secs_f32() / 60.0).round());
    }
}

fn update_sub_mood_fact<T: ToString>(key: T, fact_db: &mut FactDb, mood: &Option<MoodState>) {
    match mood {
        Some(mood) => {
            fact_db.add(key, mood.satisfaction.score());
        }
        None => {
            fact_db.remove(key);
        }
    }
}

fn update_mood(mut query: Query<(&mut EntityFactDatabase, &Mood), Changed<Mood>>) {
    for (mut fact_db, mood) in query.iter_mut() {
        update_sub_mood_fact("MoodHunger", &mut fact_db.0, &mood.hunger);
        update_sub_mood_fact("MoodCleanliness", &mut fact_db.0, &mood.cleanliness);
        update_sub_mood_fact("MoodFun", &mut fact_db.0, &mood.fun);
    }
}

fn update_overall_mood(
    mut query: Query<(&mut EntityFactDatabase, &MoodCategory), Changed<MoodCategory>>,
) {
    for (mut fact_db, mood_category) in query.iter_mut() {
        fact_db.0.add("Mood", mood_category.score());
    }
}

fn update_median_mood(
    mut query: Query<(&mut EntityFactDatabase, &MoodCategoryHistory), Changed<MoodCategoryHistory>>,
) {
    for (mut fact_db, history) in query.iter_mut() {
        fact_db.0.add("MoodHistoryMedian", history.median().score());
    }
}

fn add_ready_to_breed(mut breeders: Query<&mut EntityFactDatabase, Added<ReadyToBreed>>) {
    for mut fact_db in breeders.iter_mut() {
        fact_db.0.add("IsReadyToBreed", 1.0);
    }
}

fn remove_ready_to_breed(
    mut fact_dbs: Query<&mut EntityFactDatabase>,
    mut removed: RemovedComponents<ReadyToBreed>,
) {
    for removed in removed.read() {
        if let Ok(mut fact_db) = fact_dbs.get_mut(removed) {
            fact_db.0.remove("IsReadyToBreed");
        }
    }
}

fn update_names_facts(
    mut query: Query<(&mut EntityFactDatabase, &EntityName), Changed<EntityName>>,
) {
    for (mut fact_db, name) in query.iter_mut() {
        fact_db.0.add("FirstName", fact_str_hash(&name.first_name));
        match &name.middle_name {
            Some(name) => fact_db.0.add("MiddleName", fact_str_hash(name)),
            None => fact_db.0.remove("MiddleName"),
        }
        match &name.last_name {
            Some(name) => fact_db.0.add("LastName", fact_str_hash(name)),
            None => fact_db.0.remove("LastName"),
        }
    }
}

fn update_pet_count(mut fact_db: ResMut<GlobalFactDatabase>, query: Query<Entity, With<Pet>>) {
    fact_db.0.add("PetCount", query.iter().count() as f32);
}

fn update_food_exists(
    mut global_fact_db: ResMut<GlobalFactDatabase>,
    removed: RemovedComponents<Food>,
    added: Query<Entity, Added<Food>>,
    foods: Query<&SpeciesName, With<Food>>,
) {
    const FOOD_PREFIX: &str = "FoodExists";

    if removed.is_empty() && added.iter().count() == 0 {
        return;
    }

    // Remove all existing with prefix
    global_fact_db.0.remove_with_prefix(FOOD_PREFIX);

    for name in foods.iter() {
        let key = format!("{}{}", FOOD_PREFIX, &name.0);
        let value = global_fact_db.0.get(&key) + 1.0;
        global_fact_db.0.add(&key, value);
    }
}

fn update_existing_pets(
    mut global_fact_db: ResMut<GlobalFactDatabase>,
    removed: RemovedComponents<Pet>,
    added: Query<Entity, Added<Pet>>,
    pets: Query<&SpeciesName, With<Pet>>,
) {
    const PET_PREFIX: &str = "PetExists";

    if removed.is_empty() && added.iter().count() == 0 {
        return;
    }

    global_fact_db.0.remove_with_prefix(PET_PREFIX);

    for name in pets.iter() {
        let key = format!("{}{}", PET_PREFIX, name.0);
        let value = global_fact_db.0.get(&key) + 1.0;
        global_fact_db.0.add(&key, value);
    }
}
