use bevy::prelude::*;
use sardips_core::age_core::Age;

use crate::simulation::SimulationUpdate;

pub struct AgePlugin;

impl Plugin for AgePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SimulationUpdate, tick_ages);
    }
}

fn tick_ages(time: Res<Time>, mut query: Query<&mut Age>) {
    for mut age in query.iter_mut() {
        age.0 += time.delta();
    }
}
