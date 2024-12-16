use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    food::{Food, FoodFillFactor},
    layering,
    name::SpeciesName,
    simulation::{SimulationUpdate, HUNGER_TICK_DOWN},
    sounds::{PlaySoundEffect, SoundEffect},
    thinking::TryThinkEvent,
    SimulationState,
};

use fact_db::{Concept, FactDb};

pub struct HungerPlugin;

impl Plugin for HungerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EatFoodEvent>()
            .add_systems(SimulationUpdate, tick_hunger)
            .add_systems(
                FixedUpdate,
                (update_starving, begin_eating_food, eating_food)
                    .run_if(in_state(SimulationState::Running)),
            );
    }
}

#[derive(Debug, Component, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Hunger {
    pub value: f32,
    pub max: f32,
}

impl Hunger {
    pub fn new(max: f32) -> Self {
        Self { value: max, max }
    }

    pub fn with_value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    pub fn decrease(&mut self, amount: f32) {
        self.value -= amount;
        if self.value < 0.0 {
            self.value = 0.0;
        }
    }

    pub fn increase(&mut self, amount: f32) {
        self.value += amount;
        if self.value > self.max {
            self.value = self.max;
        }
    }

    pub fn empty(&self) -> bool {
        self.value <= 0.0
    }

    pub fn filled_percent(&self) -> f32 {
        self.value / self.max
    }
}

#[derive(Component)]
pub struct Starving;

fn tick_hunger(mut commands: Commands, mut query: Query<(Entity, &mut Hunger), Without<Starving>>) {
    for (entity, mut hunger) in query.iter_mut() {
        hunger.decrease(HUNGER_TICK_DOWN);
        if hunger.empty() {
            commands.entity(entity).insert(Starving);
        }
    }
}

fn update_starving(mut commands: Commands, query: Query<(Entity, &Hunger), With<Starving>>) {
    for (entity, hunger) in query.iter() {
        if hunger.value > 0.0 {
            commands.entity(entity).remove::<Starving>();
        }
    }
}

#[derive(Event)]
pub struct EatFoodEvent {
    food: Entity,
    eater: Entity,
}

impl EatFoodEvent {
    pub fn new(food: Entity, eater: Entity) -> Self {
        Self { food, eater }
    }
}

#[derive(Component)]
pub struct Eating {
    timer: Timer,
    target_food: Entity,
}

fn begin_eating_food(
    mut commands: Commands,
    mut try_think_events: EventWriter<TryThinkEvent>,
    mut events: EventReader<EatFoodEvent>,
    mut play_sounds: EventWriter<PlaySoundEffect>,
    mut food: Query<(&mut Transform, &SpeciesName), With<Food>>,
) {
    for event in events.read() {
        if let Ok((mut transform, species_name)) = food.get_mut(event.food) {
            // Enqueue a thought event for the eater
            {
                let mut fact_db = FactDb::default();
                fact_db.add_str("TargetFood", &species_name.0);

                try_think_events.send(
                    TryThinkEvent::new(event.eater, Concept::ThinkStartingEating)
                        .with_facts(fact_db),
                );
            }

            {
                // Remove food since it's being eaten but still want to render
                commands.entity(event.food).remove::<Food>();
                // Remove the tag
                commands.entity(event.food).despawn_descendants();

                transform.translation.z = layering::view_screen::FOOD_EATING;
            }

            play_sounds.send(PlaySoundEffect::new(SoundEffect::Eating));

            commands.entity(event.eater).insert(Eating {
                timer: Timer::from_seconds(3.5, TimerMode::Once),
                target_food: event.food,
            });
        } else {
            error!("Food entity not found: {:?}", event.food);
            continue;
        }
    }
    events.clear();
}

fn eating_food(
    mut commands: Commands,
    time: Res<Time>,
    mut try_think_events: EventWriter<TryThinkEvent>,
    mut eaters: Query<(Entity, &mut Eating, &mut Hunger)>,
    mut foods: Query<(&mut Sprite, &FoodFillFactor, &SpeciesName)>,
) {
    for (entity, mut eating, mut hunger) in eaters.iter_mut() {
        eating.timer.tick(time.delta());

        // Get percentage of timer complete
        let percent = eating.timer.elapsed().as_secs_f32() / eating.timer.duration().as_secs_f32();
        if let Ok((mut sprite, _, _)) = foods.get_mut(eating.target_food) {
            sprite.color = Color::srgba(1.0, 1.0, 1.0, 1.0 - percent);
        }

        if eating.timer.finished() {
            if let Ok((_, food_fill_factor, name)) = foods.get(eating.target_food) {
                let mut fact_db: FactDb = FactDb::default();
                fact_db.add_str("TargetFood", &name.0);

                try_think_events
                    .send(TryThinkEvent::new(entity, Concept::ThinkJustAte).with_facts(fact_db));

                hunger.increase(food_fill_factor.0);
                commands.entity(eating.target_food).despawn_recursive();
            }
            commands.entity(entity).remove::<Eating>();
        }
    }
}
