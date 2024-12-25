use std::time::Duration;

use bevy::prelude::*;
use sardips_core::mood_core::{
    AutoSetMoodImage, MoodCategory, MoodImageIndexes, SatisfactionRating,
};
use shared_deps::serde::{Deserialize, Serialize};

use crate::{
    money::{MoneyHungry, Wallet},
    player::Player,
    simulation::{
        SimulationUpdate, CLEANLINESS_MOOD_UPDATE, FUN_MOOD_UPDATE, HUNGER_MOOD_UPDATE,
        MONEY_MOOD_UPDATE, MOOD_HISTORY_UPDATE,
    },
    view::HasView,
};

use super::{
    fun::Fun,
    hunger::Hunger,
    poop::{Cleanliness, Poop},
};

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

#[derive(Component, Clone, Serialize, Deserialize, Default, Reflect)]
#[reflect(Component)]
pub struct Mood {
    pub hunger: Option<MoodState>,
    pub cleanliness: Option<MoodState>,
    pub fun: Option<MoodState>,
    pub money: Option<MoodState>,
}

impl Mood {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Mood {
    fn get_all(&self) -> Vec<&MoodState> {
        let mut result = Vec::new();

        if let Some(hunger) = &self.hunger {
            result.push(hunger);
        }

        if let Some(cleanliness) = &self.cleanliness {
            result.push(cleanliness);
        }

        if let Some(fun) = &self.fun {
            result.push(fun);
        }

        if let Some(money) = &self.money {
            result.push(money);
        }

        result
    }

    // This isn't working right
    fn get_overall(&self) -> MoodCategory {
        let mut ratings: Vec<SatisfactionRating> = Vec::new();

        for mood in self.get_all() {
            ratings.extend(mood.satisfaction.over_all_array());
        }

        if ratings.is_empty() {
            return MoodCategory::Neutral;
        }

        ratings.sort();

        let median = ratings.len() / 2;
        let median = ratings[median];

        median.into()
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

fn add_hunger_mood(mut query: Query<&mut Mood, Added<Hunger>>) {
    for mut mood in query.iter_mut() {
        if mood.hunger.is_none() {
            mood.hunger = Some(MoodState::new(HUNGER_MOOD_UPDATE));
        }
    }
}

fn update_hunger_mood(time: Res<Time>, mut hungry_moods: Query<(&Hunger, &mut Mood)>) {
    for (hunger, mut mood) in hungry_moods.iter_mut() {
        update_mood(&time, hunger.filled_percent() > 0.15, mood.hunger.as_mut());
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

fn update_overall_mood(mut moods: Query<(&Mood, &mut MoodCategory)>) {
    for (mood, mut category) in moods.iter_mut() {
        let overall = mood.get_overall();
        if *category != overall {
            *category = overall;
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

#[derive(Component, Serialize, Deserialize, Clone, Reflect)]
#[reflect(Component)]
pub struct MoodCategoryHistory {
    pub update_timer: Timer,
    pub history: Vec<MoodCategory>,
}

impl Default for MoodCategoryHistory {
    fn default() -> Self {
        Self {
            update_timer: Timer::new(MOOD_HISTORY_UPDATE, TimerMode::Repeating),
            history: Vec::new(),
        }
    }
}

impl MoodCategoryHistory {
    pub fn median(&self) -> MoodCategory {
        // SLOW POINT BUT SHOULD NOT MATTER
        let mut ratings: Vec<SatisfactionRating> = Vec::new();

        for mood in &self.history {
            ratings.extend(SatisfactionRating::from(*mood).over_all_array());
        }

        if ratings.is_empty() {
            return MoodCategory::Neutral;
        }

        ratings.sort();

        let median = ratings.len() / 2;
        let median = ratings[median];

        median.into()
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
