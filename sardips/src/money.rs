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

pub fn money_display(money: Money) -> String {
    format!("{:.2}", money as f32 / 100.)
}

pub fn money_aberration_display<T: Into<i128>>(money: T) -> String {
    let money: i128 = money.into();
    const SETS: [(i128, &str); 5] = [
        (100000000000000, "T"),
        (100000000000, "B"),
        (100000000, "M"),
        (100000, "K"),
        (1, ""),
    ];

    for (set, suffix) in SETS.iter() {
        if money.abs() > *set {
            let sign = if money < 0 { "-" } else { "" };
            return format!(
                "{}{:.0}{}",
                sign,
                (money as f32 / *set as f32).abs(),
                suffix
            );
        }
    }

    format!("{}", money)
}

pub fn money_aberration_decimal_display<T: Into<i128>>(money: T) -> String {
    let money: i128 = money.into();
    const SETS: [(i128, &str); 5] = [
        (100000000000000, "T"),
        (100000000000, "B"),
        (100000000, "M"),
        (100000, "K"),
        (1, ""),
    ];

    for (set, suffix) in SETS.iter() {
        if money.abs() > *set {
            return format!("{:.4}{}", (money as f32 / *set as f32), suffix);
        }
    }

    format!("{}", money)
}
