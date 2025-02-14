use std::time::Duration;

use bevy::prelude::*;
use sardips_core::{
    from_hours, from_mins,
    fun_core::Fun,
    hunger_core::Hunger,
    money_core::MoneyHungry,
    mood_core::{
        AutoSetMoodImage, MoodCategory, MoodCategoryHistory, MoodImageIndexes, SatisfactionRating,
    },
    pet_core::Cleanliness,
    view::HasView,
};
use serde::{Deserialize, Serialize};

use crate::{
    money::Wallet,
    player::Player,
    simulation::{
        SimulationUpdate, CLEANLINESS_MOOD_UPDATE, FUN_MOOD_UPDATE, HUNGER_MOOD_UPDATE,
        MONEY_MOOD_UPDATE,
    },
};

use super::poop::Poop;

pub struct MoodPlugin;

impl Plugin for MoodPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            SimulationUpdate,
            (
                update_hunger_mood,
                update_cleanliness_mood,
                update_fun_mood,
                update_money_mood,
                update_overall_mood,
                push_mood_category_history,
            ),
        )
        .add_systems(
            Update,
            (
                update_mood_images,
                add_hunger_mood,
                add_cleanliness_mood,
                add_fun_mood,
                add_money_mood,
            ),
        );
    }
}

#[derive(Clone, Serialize, Deserialize, Reflect, PartialEq)]
#[reflect_value(PartialEq, Serialize, Deserialize)]
pub struct MoodState {
    pub satisfaction: SatisfactionRating,
    pub timer: Timer,
}

impl MoodState {
    pub fn new(duration: Duration) -> Self {
        Self {
            satisfaction: SatisfactionRating::Neutral,
            timer: Timer::new(duration, TimerMode::Repeating),
        }
    }
}

pub const SCORE_QUOTA_PER_MOOD: f32 = 100.0;

#[derive(Clone, Serialize, Deserialize, Reflect)]
enum MoodHungerState {
    Filled,
    Hungry,
    Starving,
}

#[derive(Component, Clone, Serialize, Deserialize, Reflect)]
pub struct MoodHunger {
    check_timer: Timer,
    state: MoodHungerState,
    state_duration: Duration,
    dropdown_counter: Duration,
}

impl MoodHunger {
    pub fn score(&self) -> f32 {
        let duration = self.state_duration;
        match self.state {
            MoodHungerState::Filled => {
                if duration > from_hours(2) {
                    100.
                } else if duration > from_mins(30) {
                    let percent_to_next = duration.as_secs_f32() / from_hours(2).as_secs_f32();
                    80. + (20. * percent_to_next)
                } else {
                    let percent_to_next = duration.as_secs_f32() / from_mins(30).as_secs_f32();
                    60. + (20. * percent_to_next)
                }
            }
            MoodHungerState::Hungry => 20.,
            MoodHungerState::Starving => 0.,
        }
    }

    pub fn current_satisfaction(&self) -> SatisfactionRating {
        let score = self.score();
        if score > 2.5 {
            SatisfactionRating::VerySatisfied
        } else if score > 2.0 {
            SatisfactionRating::Satisfied
        } else if score > 0.5 {
            SatisfactionRating::Neutral
        } else if score > 0. {
            SatisfactionRating::Unsatisfied
        } else {
            SatisfactionRating::VeryUnsatisfied
        }
    }

    fn change_state(&mut self, new_state: MoodHungerState) {
        self.state = new_state;
        self.state_duration = Duration::ZERO;
        self.dropdown_counter = Duration::ZERO;
    }
}

impl Default for MoodHunger {
    fn default() -> Self {
        Self {
            check_timer: Timer::new(HUNGER_MOOD_UPDATE, TimerMode::Repeating),
            state: MoodHungerState::Filled,
            dropdown_counter: Duration::ZERO,
            state_duration: Duration::ZERO,
        }
    }
}

#[derive(Component, Clone, Serialize, Deserialize, Default, Reflect)]
#[reflect(Component)]
pub struct Mood {
    pub cleanliness: Option<MoodState>,
    pub fun: Option<MoodState>,
    pub money: Option<MoodState>,
}

impl Mood {
    pub fn new() -> Self {
        Self::default()
    }
}

fn update_mood(time: &Time, condition: bool, mood: Option<&mut MoodState>) {
    if let Some(mood) = mood {
        if mood.timer.tick(time.delta()).just_finished() {
            if condition {
                mood.satisfaction = mood.satisfaction.step_up();
            } else {
                mood.satisfaction = mood.satisfaction.step_down();
            }
        }
    }
}

fn add_hunger_mood(mut commands: Commands, mut query: Query<Entity, (With<Mood>, Added<Hunger>)>) {
    for entity in query.iter_mut() {
        commands.entity(entity).insert(MoodHunger::default());
    }
}

fn update_hunger_mood(time: Res<Time>, mut hungry_moods: Query<(&Hunger, &mut MoodHunger)>) {
    for (hunger, mut mood) in hungry_moods.iter_mut() {
        match mood.state {
            MoodHungerState::Filled => {
                mood.state_duration += time.delta();
                if hunger.filled_percent() < 0.10 {
                    mood.dropdown_counter += time.delta();
                    if mood.dropdown_counter > from_mins(20) {
                        mood.change_state(MoodHungerState::Hungry);
                    }
                }
            }
            MoodHungerState::Hungry => {
                mood.state_duration += time.delta();
                if hunger.filled_percent() < 0.05 {
                    mood.dropdown_counter += time.delta();
                    if mood.dropdown_counter > from_hours(1) {
                        mood.change_state(MoodHungerState::Starving);
                    }
                } else if hunger.filled_percent() > 0.10 {
                    mood.change_state(MoodHungerState::Filled);
                }
            }
            MoodHungerState::Starving => {
                mood.state_duration += time.delta();
                if hunger.filled_percent() > 0.05 {
                    mood.change_state(MoodHungerState::Hungry);
                }
            }
        }
    }
}

fn add_cleanliness_mood(mut query: Query<&mut Mood, Added<Cleanliness>>) {
    for mut mood in query.iter_mut() {
        if mood.cleanliness.is_none() {
            mood.cleanliness = Some(MoodState::new(CLEANLINESS_MOOD_UPDATE));
        }
    }
}

fn update_cleanliness_mood(
    time: Res<Time>,
    mut moods: Query<&mut Mood, With<Cleanliness>>,
    poops: Query<&Poop>,
) {
    let poop_count = poops.iter().count();

    for mut mood in moods.iter_mut() {
        update_mood(&time, poop_count < 1, mood.cleanliness.as_mut());
    }
}

fn add_fun_mood(mut query: Query<&mut Mood, Added<Fun>>) {
    for mut mood in query.iter_mut() {
        if mood.fun.is_none() {
            mood.fun = Some(MoodState::new(FUN_MOOD_UPDATE));
        }
    }
}

fn update_fun_mood(time: Res<Time>, mut fun_moods: Query<(&Fun, &mut Mood)>) {
    for (fun, mut mood) in fun_moods.iter_mut() {
        update_mood(&time, fun.filled(), mood.fun.as_mut());
    }
}

fn add_money_mood(mut query: Query<&mut Mood, Added<MoneyHungry>>) {
    for mut mood in query.iter_mut() {
        if mood.money.is_none() {
            mood.money = Some(MoodState::new(MONEY_MOOD_UPDATE));
        }
    }
}

fn update_money_mood(
    time: Res<Time>,
    player_wallet: Query<&Wallet, With<Player>>,
    mut money_moods: Query<(&mut MoneyHungry, &mut Mood)>,
) {
    let player_wallet = player_wallet.single();

    for (mut money, mut mood) in money_moods.iter_mut() {
        if let Some(mood) = &mut mood.money {
            if mood.timer.tick(time.delta()).just_finished() {
                if player_wallet.balance >= money.max_care
                    || money.previous_balance < player_wallet.balance
                {
                    mood.satisfaction = mood.satisfaction.step_up();
                } else {
                    mood.satisfaction = mood.satisfaction.step_down();
                }
                money.previous_balance = player_wallet.balance;
            }
        }
    }
}

fn update_overall_mood(
    mut moods: Query<(Option<&MoodHunger>, &mut MoodCategory), Changed<MoodHunger>>,
) {
    for (hunger_mood, mut category) in moods.iter_mut() {
        let mut max_possible = 0.;
        let mut score_sum = 0.;
        if let Some(hunger_mood) = hunger_mood {
            score_sum += hunger_mood.score();
            max_possible += SCORE_QUOTA_PER_MOOD;
        }

        let percent_fulfilled = score_sum / max_possible;

        let new_mood = if percent_fulfilled > 0.8 {
            MoodCategory::Ecstatic
        } else if percent_fulfilled > 0.6 {
            MoodCategory::Happy
        } else if percent_fulfilled > 0.4 {
            MoodCategory::Neutral
        } else if percent_fulfilled > 0.2 {
            MoodCategory::Sad
        } else {
            MoodCategory::Despairing
        };

        if *category != new_mood {
            *category = new_mood;
        }
    }
}

fn update_mood_images(
    mut pets: Query<
        (Entity, &MoodCategory, &MoodImageIndexes, Option<&HasView>),
        (With<AutoSetMoodImage>, Changed<MoodCategory>),
    >,
    mut texture_atlas: Query<&mut TextureAtlas>,
) {
    for (entity, category, mood_images, has_view) in pets.iter_mut() {
        let entity = has_view
            .map(|has_view| has_view.view_entity)
            .unwrap_or(entity);

        if let Ok(mut atlas) = texture_atlas.get_mut(entity) {
            atlas.index = mood_images.get_index_for_mood(*category);
        }
    }
}

fn push_mood_category_history(
    time: Res<Time>,
    mut query: Query<(&MoodCategory, &mut MoodCategoryHistory)>,
) {
    for (category, mut history) in query.iter_mut() {
        if history.update_timer.tick(time.delta()).just_finished() {
            history.history.push(*category);
        }
    }
}
