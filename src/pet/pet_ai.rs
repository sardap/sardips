use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_turborand::{DelegatedRng, RngComponent};

use crate::{
    food::{
        preferences::{FoodPreference, FoodSensationRating},
        Food, FoodSensations,
    },
    name::EntityName,
    SimulationState,
};

use super::{
    breeding::{BreedEvent, ReadyToBreed},
    hunger::{EatFoodEvent, Hunger},
    move_towards::{MoveTowardsEvent, MovingTowards},
    wonder::Wonder,
};

pub struct PetAiPlugin;

impl Plugin for PetAiPlugin {
    fn build(&self, app: &mut App) {
        // This should probably run in sim update
        app.add_systems(
            FixedUpdate,
            (
                tick_cooldowns,
                select_action,
                find_food,
                reached_food,
                eating_food_complete,
                reset_cooldowns,
                breed_find_partner_action,
            )
                .run_if(in_state(SimulationState::Running)),
        );
    }
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct PetAi {
    pub food_cooldown: Timer,
    pub check_breed_cooldown: Timer,
}

impl Default for PetAi {
    fn default() -> Self {
        let mut food_cooldown = Timer::from_seconds(5.0, TimerMode::Once);
        food_cooldown.tick(food_cooldown.duration());

        let mut check_breed_cooldown = Timer::from_seconds(5.0, TimerMode::Once);
        check_breed_cooldown.tick(check_breed_cooldown.duration());
        Self {
            food_cooldown,
            check_breed_cooldown,
        }
    }
}

#[derive(Component)]
struct FindFoodAction;

#[derive(Component)]
struct MovingTowardsFoodAction(Entity);

#[derive(Component)]
struct WaitingToFinishEatingAction {
    target_food: Entity,
}

#[derive(Component)]
struct BreedFindPartnerAction;

fn tick_cooldowns(time: Res<Time>, mut query: Query<&mut PetAi>) {
    for mut pet_ai in query.iter_mut() {
        pet_ai.food_cooldown.tick(time.delta());
        pet_ai.check_breed_cooldown.tick(time.delta());
    }
}

fn reset_cooldowns(mut q_ai: Query<&mut PetAi>, food: Query<Entity, Added<FindFoodAction>>) {
    for entity in food.iter() {
        if let Ok(mut pet_ai) = q_ai.get_mut(entity) {
            pet_ai.food_cooldown.reset();
        }
    }
}

fn select_action(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &EntityName,
            &mut PetAi,
            Option<&Hunger>,
            Option<&ReadyToBreed>,
        ),
        (With<PetAi>, With<Wonder>),
    >,
) {
    for (entity, name, mut pet_ai, hunger, ready_to_breed) in query.iter_mut() {
        if pet_ai.food_cooldown.finished() {
            if let Some(hunger) = hunger {
                if hunger.value < hunger.max * 0.15 {
                    info!(
                        "{} Pet is hungry {}/{} finding food",
                        name,
                        hunger.filled_percent() * 100.,
                        hunger.max
                    );
                    add_action(&mut commands.entity(entity), FindFoodAction);
                    pet_ai.food_cooldown.reset();
                }
            }
        }

        if pet_ai.check_breed_cooldown.finished() {
            if ready_to_breed.is_some() {
                info!("{} Checking for breeding", name);
                add_action(&mut commands.entity(entity), BreedFindPartnerAction);
                pet_ai.check_breed_cooldown.reset();
            }
        }
    }
}

fn replace_action<T: Component, U: Component>(entity_builder: &mut EntityCommands, action: U) {
    entity_builder.remove::<T>();
    entity_builder.insert(action);
}

fn add_action<T: Component>(entity_builder: &mut EntityCommands, action: T) {
    replace_action::<Wonder, T>(entity_builder, action);
}

fn stop_action<T: Component>(entity_builder: &mut EntityCommands) {
    replace_action::<T, Wonder>(entity_builder, Wonder::default());
}

fn find_food(
    mut commands: Commands,
    mut move_towards_events: EventWriter<MoveTowardsEvent>,
    mut query: Query<(Entity, &EntityName, &FoodPreference), With<FindFoodAction>>,
    food: Query<(Entity, &Transform, &FoodSensations), With<Food>>,
) {
    for (entity, entity_name, food_preference) in query.iter_mut() {
        // sort food by tastiness
        let mut food_ratings = Vec::new();
        for (food_entity, trans, sensations) in &food {
            let feeling = food_preference.feeling(sensations);
            if feeling != FoodSensationRating::Despises {
                food_ratings.push((food_entity, food_preference.feeling(sensations), trans));
            }
        }
        food_ratings.sort_by(|a, b| a.1.f32().partial_cmp(&b.1.f32()).unwrap());

        if food_ratings.is_empty() {
            info!("{} No food found", entity_name);
            stop_action::<FindFoodAction>(&mut commands.entity(entity));
            continue;
        }

        let selected_food = food_ratings.first().unwrap();
        info!("{} Found food rating {}", entity_name, selected_food.1);

        replace_action::<FindFoodAction, MovingTowardsFoodAction>(
            &mut commands.entity(entity),
            MovingTowardsFoodAction(selected_food.0),
        );
        move_towards_events.send(MoveTowardsEvent::new(
            entity,
            Vec2::new(selected_food.2.translation.x, selected_food.2.translation.y),
        ));
    }
}

fn reached_food(
    mut commands: Commands,
    mut eating_events: EventWriter<EatFoodEvent>,
    mut query: Query<(Entity, &MovingTowardsFoodAction), Without<MovingTowards>>,
) {
    for (entity, moving_towards_food) in query.iter_mut() {
        eating_events.send(EatFoodEvent::new(moving_towards_food.0, entity));
        replace_action::<MovingTowardsFoodAction, WaitingToFinishEatingAction>(
            &mut commands.entity(entity),
            WaitingToFinishEatingAction {
                target_food: moving_towards_food.0,
            },
        );
    }
}

fn eating_food_complete(
    mut commands: Commands,
    mut query: Query<(Entity, &WaitingToFinishEatingAction)>,
    target_food: Query<Entity>,
) {
    for (entity, waiting_to_finish) in query.iter_mut() {
        if target_food.get(waiting_to_finish.target_food).is_err() {
            stop_action::<WaitingToFinishEatingAction>(&mut commands.entity(entity));
        }
    }
}

fn breed_find_partner_action(
    mut commands: Commands,
    mut breed_events: EventWriter<BreedEvent>,
    ready_to_breed: Query<Entity, With<ReadyToBreed>>,
    mut query: Query<(Entity, &mut RngComponent), With<BreedFindPartnerAction>>,
) {
    for (entity, mut rng) in query.iter_mut() {
        if ready_to_breed.get(entity).is_err() {
            stop_action::<BreedFindPartnerAction>(&mut commands.entity(entity));
            continue;
        }

        // TODO once ancestor DB is ready stop inset
        let possible_partners: Vec<_> = ready_to_breed.iter().filter(|e| *e != entity).collect();
        if possible_partners.is_empty() {
            stop_action::<BreedFindPartnerAction>(&mut commands.entity(entity));
            continue;
        }

        let partner = possible_partners[rng.usize(0..possible_partners.len())];
        breed_events.send(BreedEvent::new(entity, partner));

        // TODO This should be expanded in the future to include moving towards the partner
        stop_action::<BreedFindPartnerAction>(&mut commands.entity(entity));
    }
}
