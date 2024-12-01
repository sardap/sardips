use std::fmt;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct MoneyPlugin;

impl Plugin for MoneyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Money>()
            .register_type::<Wallet>()
            .register_type::<MoneyHungry>();
    }
}

pub type Money = i64;

#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct Wallet {
    pub balance: Money,
}

impl fmt::Display for Wallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.balance as f32 / 100.)
    }
}

#[derive(Debug, Component, Serialize, Deserialize, Clone, Reflect)]
#[reflect(Component)]
pub struct MoneyHungry {
    pub previous_balance: Money,
    pub max_care: Money,
}
