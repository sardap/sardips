use bevy::prelude::*;
use serde::Deserialize;
use shared_deps::bevy_turborand::{DelegatedRng, RngComponent};

use crate::{
    age::Age,
    name::{EntityName, SpeciesName},
    simulation::{SimulationState, SimulationUpdate},
};

use fact_db::{Concept, Criteria, Criterion, EntityFactDatabase, FactQuery, GlobalFactDatabase};

use super::{
    mood::MoodCategoryHistory,
    template::{EvolvingPet, PetTemplateDatabase, SpawnPetEvent},
    Pet, PetKind,
};

pub struct EvolvePlugin;

impl Plugin for EvolvePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SimulationUpdate, check_evolve).add_systems(
            PreUpdate,
            evolve_pending.run_if(in_state(SimulationState::Running)),
        );
    }
}

#[derive(Deserialize)]
pub struct PossibleEvolution {
    pub criteria: Vec<Criterion>,
    pub species: Vec<String>,
}

impl PossibleEvolution {
    pub fn criteria(&self) -> Criteria {
        Criteria::new(Concept::Evolve, &self.criteria)
    }
}

struct CheckEvolveTimer {
    timer: Timer,
}

impl Default for CheckEvolveTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(60., TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
pub struct ShouldEvolve {
    species: String,
}

impl ShouldEvolve {
    pub fn new(species: impl ToString) -> Self {
        Self {
            species: species.to_string(),
        }
    }
}

fn check_evolve(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: Local<CheckEvolveTimer>,
    global_fact_db: Res<GlobalFactDatabase>,
    pet_template_db: Res<PetTemplateDatabase>,
    mut possible_evolvers: Query<
        (
            Entity,
            &SpeciesName,
            &EntityFactDatabase,
            &mut RngComponent,
            &PetKind,
        ),
        (With<Pet>, Without<ShouldEvolve>),
    >,
) {
    if !timer.timer.tick(time.delta()).just_finished() {
        return;
    }

    for (entity, species_name, entity_fact_db, mut rng, pet_kind) in possible_evolvers.iter_mut() {
        let template = pet_template_db.get_by_name(&species_name.0).unwrap();

        let fact_query = FactQuery::new(Concept::Evolve)
            .add_fact_db(&global_fact_db.0)
            .add_fact_db(&entity_fact_db.0);

        for possible_evolution in &template.possible_evolutions {
            if fact_query.single_criteria(&possible_evolution.criteria()) {
                let selected = if *pet_kind == PetKind::Blob {
                    let starters: Vec<_> = pet_template_db.iter().filter(|t| t.starter).collect();
                    &starters[rng.usize(0..starters.len())].species_name
                } else {
                    &possible_evolution.species[rng.usize(0..possible_evolution.species.len())]
                };

                commands.entity(entity).insert(ShouldEvolve::new(selected));
            }
        }
    }
}

fn evolve_pending(
    mut spawn_pets: EventWriter<SpawnPetEvent>,
    evolvers: Query<
        (
            Entity,
            &ShouldEvolve,
            &Transform,
            &EntityName,
            &Age,
            &MoodCategoryHistory,
            &EntityFactDatabase,
        ),
        With<Pet>,
    >,
) {
    for (entity, should_evolve, transform, entity_name, age, mood_history, fact_db) in
        evolvers.iter()
    {
        debug!(
            "Evolving {} into {}",
            entity_name.full_name(),
            should_evolve.species
        );

        let evolve = EvolvingPet {
            entity,
            location: transform.translation.xy(),
            name: entity_name.clone(),
            age: age.clone(),
            mood_history: mood_history.clone(),
            fact_db: fact_db.clone(),
        };
        spawn_pets.send(SpawnPetEvent::Evolve((
            should_evolve.species.clone(),
            evolve,
        )));
    }
}
