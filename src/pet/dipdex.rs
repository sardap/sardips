use std::collections::HashSet;

use bevy::prelude::*;

use crate::name::SpeciesName;

use super::Pet;

pub struct DipdexPlugin;

impl Plugin for DipdexPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HashSet<String>>()
            .register_type_data::<HashSet<String>, ReflectSerialize>()
            .register_type_data::<HashSet<String>, ReflectDeserialize>()
            .register_type::<DipdexDiscoveredEntries>()
            .add_systems(Update, pet_discovered);
    }
}

#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component)]
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
