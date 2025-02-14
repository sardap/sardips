use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct FunCorePlugin;

impl Plugin for FunCorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Fun>();
    }
}

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
