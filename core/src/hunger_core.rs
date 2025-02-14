use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct HungerCorePlugin;

impl Plugin for HungerCorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Hunger>();
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
