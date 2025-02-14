use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

use crate::MOOD_HISTORY_UPDATE;

pub struct MoodCorePlugin;

impl Plugin for MoodCorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SatisfactionRating>()
            .register_type::<MoodCategory>()
            .register_type::<Vec<MoodCategory>>()
            .register_type::<MoodCategoryHistory>();
    }
}

#[derive(Component, Default)]
pub struct AutoSetMoodImage;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, PartialOrd, Ord, Reflect,
)]
#[reflect_value(PartialEq, Serialize, Deserialize)]
pub enum SatisfactionRating {
    VerySatisfied,
    Satisfied,
    #[default]
    Neutral,
    Unsatisfied,
    VeryUnsatisfied,
}

impl SatisfactionRating {
    pub fn over_all_array(&self) -> &[SatisfactionRating] {
        match self {
            SatisfactionRating::VeryUnsatisfied => &VERY_UNSATISFIED,
            SatisfactionRating::Unsatisfied => &UNSATISFIED,
            SatisfactionRating::Neutral => &NEUTRAL,
            SatisfactionRating::Satisfied => &SATISFIED,
            SatisfactionRating::VerySatisfied => &VERY_SATISFIED,
        }
    }

    pub fn step_down(self) -> Self {
        match self {
            SatisfactionRating::VerySatisfied => SatisfactionRating::Satisfied,
            SatisfactionRating::Satisfied => SatisfactionRating::Neutral,
            SatisfactionRating::Neutral => SatisfactionRating::Unsatisfied,
            SatisfactionRating::Unsatisfied => SatisfactionRating::VeryUnsatisfied,
            SatisfactionRating::VeryUnsatisfied => SatisfactionRating::VeryUnsatisfied,
        }
    }

    pub fn step_up(self) -> Self {
        match self {
            SatisfactionRating::VerySatisfied => SatisfactionRating::VerySatisfied,
            SatisfactionRating::Satisfied => SatisfactionRating::VerySatisfied,
            SatisfactionRating::Neutral => SatisfactionRating::Satisfied,
            SatisfactionRating::Unsatisfied => SatisfactionRating::Neutral,
            SatisfactionRating::VeryUnsatisfied => SatisfactionRating::Unsatisfied,
        }
    }

    pub fn atlas_index(&self) -> usize {
        match self {
            SatisfactionRating::VeryUnsatisfied => 0,
            SatisfactionRating::Unsatisfied => 1,
            SatisfactionRating::Neutral => 2,
            SatisfactionRating::Satisfied => 3,
            SatisfactionRating::VerySatisfied => 4,
        }
    }

    pub fn score(&self) -> f32 {
        match self {
            SatisfactionRating::VeryUnsatisfied => 1.,
            SatisfactionRating::Unsatisfied => 1.25,
            SatisfactionRating::Neutral => 1.5,
            SatisfactionRating::Satisfied => 1.75,
            SatisfactionRating::VerySatisfied => 2.,
        }
    }
}

#[derive(
    Debug, Component, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, Reflect,
)]
#[reflect(Component)]
pub enum MoodCategory {
    Ecstatic,
    Happy,
    #[default]
    Neutral,
    Sad,
    Despairing,
}

impl MoodCategory {
    pub fn score(&self) -> f32 {
        let satisfaction = SatisfactionRating::from(*self);
        satisfaction.score()
    }
}

impl From<SatisfactionRating> for MoodCategory {
    fn from(val: SatisfactionRating) -> Self {
        match val {
            SatisfactionRating::VeryUnsatisfied => MoodCategory::Despairing,
            SatisfactionRating::Unsatisfied => MoodCategory::Sad,
            SatisfactionRating::Neutral => MoodCategory::Neutral,
            SatisfactionRating::Satisfied => MoodCategory::Happy,
            SatisfactionRating::VerySatisfied => MoodCategory::Ecstatic,
        }
    }
}

const VERY_SATISFIED: [SatisfactionRating; 3] = [
    SatisfactionRating::Satisfied,
    SatisfactionRating::VerySatisfied,
    SatisfactionRating::VerySatisfied,
];

const SATISFIED: [SatisfactionRating; 2] =
    [SatisfactionRating::Neutral, SatisfactionRating::Satisfied];

const NEUTRAL: [SatisfactionRating; 1] = [SatisfactionRating::Neutral];

const UNSATISFIED: [SatisfactionRating; 2] =
    [SatisfactionRating::Neutral, SatisfactionRating::Unsatisfied];

const VERY_UNSATISFIED: [SatisfactionRating; 3] = [
    SatisfactionRating::Unsatisfied,
    SatisfactionRating::VeryUnsatisfied,
    SatisfactionRating::VeryUnsatisfied,
];

impl From<MoodCategory> for SatisfactionRating {
    fn from(mood: MoodCategory) -> Self {
        match mood {
            MoodCategory::Ecstatic => SatisfactionRating::VerySatisfied,
            MoodCategory::Happy => SatisfactionRating::Satisfied,
            MoodCategory::Neutral => SatisfactionRating::Neutral,
            MoodCategory::Sad => SatisfactionRating::Unsatisfied,
            MoodCategory::Despairing => SatisfactionRating::VeryUnsatisfied,
        }
    }
}

#[derive(Debug, Component, Default, Clone, Copy)]
pub struct MoodImageIndexes {
    indexes: [usize; 5],
}

impl MoodImageIndexes {
    pub fn new(map: &HashMap<MoodCategory, u32>) -> Self {
        let mut indexes = [0; 5];
        for (mood, index) in map {
            indexes[MoodImageIndexes::get_index(*mood)] = *index as usize;
        }

        Self { indexes }
    }

    fn get_index(mood: MoodCategory) -> usize {
        match mood {
            MoodCategory::Neutral => 0,
            MoodCategory::Happy => 1,
            MoodCategory::Ecstatic => 2,
            MoodCategory::Sad => 3,
            MoodCategory::Despairing => 4,
        }
    }

    pub fn get_index_for_mood(&self, mood: MoodCategory) -> usize {
        self.indexes[MoodImageIndexes::get_index(mood)]
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
