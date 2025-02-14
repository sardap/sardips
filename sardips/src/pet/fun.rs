use std::collections::HashMap;

use bevy::prelude::*;

use sardips_core::{fun_core::Fun, minigames_core::MiniGameType};

use crate::simulation::{SimulationUpdate, FUN_TICK_DOWN};

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

fn tick_down_fun_mood(mut fun: Query<&mut Fun>) {
    for mut fun in fun.iter_mut() {
        fun.add(-FUN_TICK_DOWN);
    }
}
