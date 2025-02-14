use std::time::Duration;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct AgeCorePlugin;

impl Plugin for AgeCorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Age>();
    }
}

#[derive(Component, Deref, DerefMut, Default, Serialize, Deserialize, Clone, Reflect)]
#[reflect(Component)]
pub struct Age(pub Duration);

impl Age {
    pub fn lived_for_text(&self) -> String {
        let seconds = self.0.as_secs();
        let minutes = seconds / 60;
        let hours = minutes / 60;
        let days = hours / 24;
        let years = days / 365;

        if years > 0 {
            format!("{} y", years)
        } else if days > 0 {
            format!("{} d", days)
        } else if hours > 0 {
            format!("{} h", hours)
        } else if minutes > 0 {
            format!("{} m", minutes)
        } else {
            format!("{} s", seconds)
        }
    }
}
