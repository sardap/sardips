use std::fmt;

use bevy::prelude::*;
use sardips_core::money_core::Money;

pub struct MoneyPlugin;

impl Plugin for MoneyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Wallet>();
    }
}

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
