use std::collections::HashSet;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::name::SpeciesName;

use super::Pet;

pub struct DipdexPlugin;

impl Plugin for DipdexPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, pet_discovered);
    }
}

#[derive(Component, Serialize, Deserialize, Default, Clone)]
pub struct DipdexDiscoveredEntries {
    pub entries: HashSet<String>,
}

fn pet_discovered(
    pet: Query<&SpeciesName, Added<Pet>>,
    mut dipdex: Query<&mut DipdexDiscoveredEntries>,
) {
    for name in pet.iter() {
        for mut dipdex in dipdex.iter_mut() {
            dipdex.entries.insert(name.0.clone());
        }
    }
}
