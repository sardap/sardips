use std::collections::HashMap;

use bevy::prelude::*;
use shared_deps::serde::{Deserialize, Serialize};

use crate::{
    minigames::MiniGameType,
    simulation::{SimulationUpdate, FUN_TICK_DOWN},
};

pub struct FunPlugin;

impl Plugin for FunPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SimulationUpdate, tick_down_fun_mood);
    }
}

#[derive(Default)]
pub enum MinigamePreference {
    Likes,
    #[default]
    Neutral,
    Hates,
}

impl MinigamePreference {
    pub fn fun_modifier(&self) -> f32 {
        match self {
            MinigamePreference::Likes => 1.5,
            MinigamePreference::Neutral => 1.0,
            MinigamePreference::Hates => 0.5,
        }
    }
}

#[derive(Component)]
pub struct MinigamePreferences(pub HashMap<MiniGameType, MinigamePreference>);

#[derive(Component, Default, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Fun {
    pub value: f32,
}

impl Fun {
    pub fn filled(&self) -> bool {
        self.value > 5.0
    }

    pub fn add(&mut self, amount: f32) {
        self.value += amount;
        if self.value < 0.0 {
            self.value = 0.0;
        }
    }
}

fn tick_down_fun_mood(mut fun: Query<&mut Fun>) {
    for mut fun in fun.iter_mut() {
        fun.add(-FUN_TICK_DOWN);
    }
}
