use bevy::prelude::*;
use shared_deps::bevy_turborand::{DelegatedRng, GlobalRng, RngComponent};

use sardips_core::{
    age_core::Age,
    assets::GameImageAssets,
    breeding_core::Breeds,
    name::EntityName,
    pet_core::{PetKind, PetTemplate, PetTemplateDatabase},
    text_database::TextDatabase,
};

use crate::simulation::{Simulated, SimulationUpdate, EGG_HATCH_ATTEMPT_INTERVAL, MAX_EGG_LIFE};

use super::template::SpawnPetEvent;
use shared_deps::rand::prelude::SliceRandom;

pub struct BreedPlugin;

impl Plugin for BreedPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BreedEvent>().add_systems(
            SimulationUpdate,
            (
                apply_pending_breeds,
                tick_breeds,
                add_ready_to_breed,
                attempt_hatch,
                egg_hatch,
            ),
        );
    }
}

fn breeding_result(
    mut breeding: [PetKind; 2],
    pet_db: &PetTemplateDatabase,
) -> Option<&PetTemplate> {
    if breeding.iter().any(|&kind| kind == PetKind::Blob) {
        return None;
    }

    breeding.sort();
    let breeder_left = breeding[0];
    let breeder_right = breeding[1];

    let kind = match (breeder_left, breeder_right) {
        (PetKind::Object, PetKind::Object) => PetKind::Creature,
        (PetKind::Object, PetKind::Creature) => PetKind::Object,
        (PetKind::Object, PetKind::Supernatural) => PetKind::Supernatural,
        (PetKind::Creature, PetKind::Creature) => PetKind::Supernatural,
        (PetKind::Creature, PetKind::Supernatural) => PetKind::Object,
        (PetKind::Supernatural, PetKind::Supernatural) => PetKind::Creature,
        _ => panic!(
            "Invalid breed pair: {:?}, {:?}",
            breeder_left, breeder_right
        ),
    };

    let possible = pet_db
        .get_by_kind(kind)
        .into_iter()
        .filter(|i| i.starter)
        .collect::<Vec<_>>();

    assert!(
        !possible.is_empty(),
        "No possible breed result for {:?} and {:?}",
        breeder_left,
        breeder_right
    );

    Some(
        *possible
            .choose(&mut shared_deps::rand::thread_rng())
            .unwrap(),
    )
}

/*
        I need to have a ancestor DB  where I can look up parents to get middle names
        Use a random first name for one of the grandparents

fn make_child_name(text_db: &TextDatabase) -> EntityName {

    EntityName::new(text_db.random_given_name_key())
}
*/

fn tick_breeds(mut query: Query<&mut Breeds>, time: Res<Time>) {
    for mut breeds in query.iter_mut() {
        breeds.breed_timer.tick(time.delta());
    }
}

#[derive(Debug, Component, Default, Clone)]
pub struct ReadyToBreed;

fn add_ready_to_breed(
    mut commands: Commands,
    query: Query<(Entity, &Breeds), Without<ReadyToBreed>>,
) {
    for (entity, breeds) in query.iter() {
        if breeds.breed_timer.finished() {
            commands.entity(entity).insert(ReadyToBreed);
        }
    }
}

#[derive(Component)]
pub struct Egg {
    pub contains: String,
}

impl Default for Egg {
    fn default() -> Self {
        Self {
            contains: "Blob".to_string(),
        }
    }
}

#[derive(Component)]
pub struct EggHatchAttempt {
    pub attempt_timer: Timer,
    pub successes: i32,
}

impl Default for EggHatchAttempt {
    fn default() -> Self {
        Self {
            attempt_timer: Timer::new(EGG_HATCH_ATTEMPT_INTERVAL, TimerMode::Repeating),
            successes: 0,
        }
    }
}

fn attempt_hatch(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut EggHatchAttempt, &mut RngComponent, &Age)>,
) {
    for (entity, mut hatch, mut rng, age) in query.iter_mut() {
        hatch.attempt_timer.tick(time.delta());
        if hatch.attempt_timer.finished() || age.0 > MAX_EGG_LIFE {
            if rng.i32(0..100) > 50 {
                continue;
            }
            hatch.successes += 1;
            if hatch.successes >= 3 {
                commands.entity(entity).remove::<EggHatchAttempt>();
            } else {
                hatch.attempt_timer.reset();
            }
        }
    }
}

fn egg_hatch(
    mut commands: Commands,
    mut spawn_pets: EventWriter<SpawnPetEvent>,
    query: Query<(Entity, &Egg, &GlobalTransform), Without<EggHatchAttempt>>,
) {
    for (entity, egg, transform) in query.iter() {
        spawn_pets.send(SpawnPetEvent::Blank((
            transform.translation().xy(),
            egg.contains.clone(),
        )));
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Bundle, Default)]
pub struct EggBundle {
    pub egg: Egg,
    pub egg_hatch: EggHatchAttempt,
    pub age: Age,
    pub simulated: Simulated,
    pub sprite: SpriteBundle,
    pub rng: RngComponent,
    pub name: EntityName,
}

fn spawn_egg(
    contains: impl ToString,
    commands: &mut Commands,
    global_rng: &mut GlobalRng,
    assets: &GameImageAssets,
    position: Vec2,
    name: EntityName,
) {
    commands.spawn(EggBundle {
        egg: Egg {
            contains: contains.to_string(),
        },
        sprite: SpriteBundle {
            transform: Transform::from_translation(Vec3::new(position.x, position.y, 0.0)),
            texture: assets.egg.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(64.0, 64.0)),
                ..default()
            },
            ..default()
        },
        rng: RngComponent::from(global_rng),
        name,
        ..default()
    });
}

pub fn hatch_egg() {}

#[derive(Event)]
pub struct BreedEvent {
    pub breeding: [Entity; 2],
}

impl BreedEvent {
    pub fn new(a: Entity, b: Entity) -> Self {
        Self { breeding: [a, b] }
    }
}

// Both can spawn breed events at the same time
fn apply_pending_breeds(
    mut commands: Commands,
    mut events: EventReader<BreedEvent>,
    mut global_rng: ResMut<GlobalRng>,
    pet_db: Res<PetTemplateDatabase>,
    game_image_assets: Res<GameImageAssets>,
    text_db: Res<TextDatabase>,
    mut query: Query<(&EntityName, &PetKind, &mut Breeds, &GlobalTransform), With<ReadyToBreed>>,
) {
    // remove doubled breeding events
    let mut seen = std::collections::HashSet::new();
    let mut event_backup = Vec::new();
    for event in events.read() {
        if seen.contains(&event.breeding) {
            continue;
        }
        seen.insert(event.breeding);
        event_backup.push(event);
    }

    for event in event_backup {
        let kinds = {
            let mut kinds = Vec::new();
            for entity in &event.breeding {
                if let Ok((_, kind, _, _)) = query.get(*entity) {
                    kinds.push(*kind);
                } else {
                    error!("Given breeder {:?} is not a breeder", entity);
                    break;
                }
            }
            kinds
        };

        if kinds.len() != event.breeding.len() {
            continue;
        }

        match breeding_result([kinds[0], kinds[1]], &pet_db) {
            Some(template) => {
                // Spawn egg at the midpoint of the two breeders
                let midpoint = (query.get(event.breeding[0]).unwrap().3.translation()
                    + query.get(event.breeding[1]).unwrap().3.translation())
                    / 2.0;

                spawn_egg(
                    &template.species_name,
                    &mut commands,
                    &mut global_rng,
                    &game_image_assets,
                    midpoint.xy(),
                    EntityName::random(&text_db),
                );

                // reset the breed timer
                for entity in &event.breeding {
                    if let Ok((_, _, mut breeds, _)) = query.get_mut(*entity) {
                        breeds.breed_timer.reset();
                        commands.entity(*entity).remove::<ReadyToBreed>();
                    }
                }
            }
            None => {
                error!("Failed to breed {:?} and {:?}", kinds[0], kinds[1]);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use sardips_core::pet_core::{
        PetTemplate, PetTemplateDatabase, PetTemplateImageSet, PreCalculated, TemplateSize,
        TemplateSpeed, WeightType,
    };
    use strum::IntoEnumIterator;

    #[test]
    fn test_all_combos_exist() {
        use super::breeding_result;
        use super::PetKind;

        let mut pet_db = PetTemplateDatabase::default();
        // Put one of every kind into the db
        for kind in PetKind::iter() {
            pet_db.add(PetTemplate {
                species_name: kind.to_string(),
                kind,
                possible_evolutions: vec![],
                image_set: PetTemplateImageSet::default(),
                size: TemplateSize::XY(1, 1),
                weight: WeightType::MiddleWeight,
                speed: TemplateSpeed::Medium,
                breeds: false,
                stomach: None,
                pooper: None,
                cleanliness: None,
                fun: None,
                money_hungry: None,
                starter: true,
                pre_calculated: PreCalculated::default(),
            });
        }

        let breeding_kinds = PetKind::iter()
            .filter(|&kind| kind != PetKind::Blob)
            .collect::<Vec<_>>();

        for kind_left in &breeding_kinds {
            for kind_right in &breeding_kinds {
                let breeding = [*kind_left, *kind_right];
                let result = breeding_result(breeding, &pet_db);
                assert!(
                    result.is_some(),
                    "No result for {:?} and {:?}",
                    kind_left,
                    kind_right
                );
            }
        }
    }
}
