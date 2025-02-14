use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct MoneyCorePlugin;

impl Plugin for MoneyCorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Money>().register_type::<MoneyHungry>();
    }
}

pub type Money = i64;

#[derive(Debug, Component, Serialize, Deserialize, Clone, Reflect)]
#[reflect(Component)]
pub struct MoneyHungry {
    pub previous_balance: Money,
    pub max_care: Money,
}
