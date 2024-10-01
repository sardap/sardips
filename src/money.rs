use std::fmt;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct MoneyPlugin;

impl Plugin for MoneyPlugin {
    fn build(&self, _: &mut App) {}
}

pub type Money = i32;

#[derive(Component, Default, Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub balance: Money,
}

impl fmt::Display for Wallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.balance as f32 / 100.)
    }
}

#[derive(Debug, Component, Serialize, Deserialize, Clone)]
pub struct MoneyHungry {
    pub previous_balance: Money,
    pub max_care: Money,
}
