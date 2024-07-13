use core::fmt;
use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::text_database::text_keys;

use super::{FoodSensationType, FoodSensations};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Copy, Clone, EnumIter, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoodSensationRating {
    Loves,
    Likes,
    Neutral,
    Dislikes,
    Hates,
    Despises,
}

impl FoodSensationRating {
    pub fn f32(&self) -> f32 {
        match self {
            FoodSensationRating::Loves => 5.0,
            FoodSensationRating::Likes => 3.0,
            FoodSensationRating::Neutral => 0.0,
            FoodSensationRating::Dislikes => -7.0,
            FoodSensationRating::Hates => -10.0,
            FoodSensationRating::Despises => -100.0,
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            FoodSensationRating::Loves => text_keys::LOVES,
            FoodSensationRating::Likes => text_keys::LIKES,
            FoodSensationRating::Neutral => text_keys::NEUTRAL,
            FoodSensationRating::Dislikes => text_keys::DISLIKES,
            FoodSensationRating::Hates => text_keys::HATES,
            FoodSensationRating::Despises => text_keys::DESPISES,
        }
    }
}

impl From<f32> for FoodSensationRating {
    fn from(value: f32) -> Self {
        for rating in FoodSensationRating::iter() {
            if value >= rating.f32() {
                return rating;
            }
        }

        FoodSensationRating::Despises
    }
}

impl fmt::Display for FoodSensationRating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Component, Serialize, Deserialize, Clone)]
pub struct FoodPreference {
    pub sensation_ratings: HashMap<FoodSensationType, FoodSensationRating>,
}

impl FoodPreference {
    pub fn feeling(&self, sensations: &FoodSensations) -> FoodSensationRating {
        let mut overall_rating = 0.;

        for sensation in sensations.values.iter() {
            if let Some(rating) = self.sensation_ratings.get(sensation) {
                overall_rating += rating.f32();
            }
        }

        overall_rating.into()
    }
}
