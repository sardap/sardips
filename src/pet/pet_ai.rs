use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::{
    food::{
        preferences::{FoodPreference, FoodSensationRating},
        Food, FoodSensations,
    },
    name::EntityName,
    SimulationState,
};

use super::{
    hunger::{EatFoodEvent, Hunger},
    move_towards::{MoveTowardsEvent, MovingTowards},
    wonder::Wonder,
};

pub struct PetAiPlugin;

impl Plugin for PetAiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                tick_cooldowns,
                select_action,
                find_food,
                reached_food,
                eating_food_complete,
                reset_cooldowns,
            )
                .run_if(in_state(SimulationState::Running)),
        );
    }
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct PetAi {
    pub food_cooldown: Timer,
}

impl Default for PetAi {
    fn default() -> Self {
        let mut food_cooldown = Timer::from_seconds(5.0, TimerMode::Once);
        food_cooldown.tick(food_cooldown.duration());
        Self { food_cooldown }
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

fn tick_cooldowns(time: Res<Time>, mut query: Query<&mut PetAi>) {
    for mut pet_ai in query.iter_mut() {
        pet_ai.food_cooldown.tick(time.delta());
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
    query: Query<(Entity, &EntityName, &PetAi, Option<&Hunger>), (With<PetAi>, With<Wonder>)>,
) {
    for (entity, name, pet_ai, hunger) in query.iter() {
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
                }
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
