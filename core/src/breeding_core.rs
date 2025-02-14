use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::BREED_RESET_INTERVAL;

pub struct BreedingCorePlugin;

impl Plugin for BreedingCorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Breeds>();
    }
}

#[derive(Debug, Component, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Breeds {
    pub breed_timer: Timer,
}

impl Default for Breeds {
    fn default() -> Self {
        Self {
            breed_timer: Timer::new(BREED_RESET_INTERVAL, TimerMode::Once),
        }
    }
}
