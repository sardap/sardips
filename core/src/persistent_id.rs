use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};
use shared_deps::moonshine_save::save::Save;

use crate::GameState;

pub struct PersistentIdPlugin;

impl Plugin for PersistentIdPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PersistentId>()
            .register_type::<PersistentIdGenerator>()
            .register_type::<PersistentIdMapping>()
            .add_systems(
                OnExit(GameState::Loading),
                (add_persistent_id_generator, add_persistent_id_mapping),
            )
            .add_systems(
                First,
                (add_persistent_id, populate_persistent_id_mapping).run_if(run_if_resource_exists),
            );
    }
}

fn run_if_resource_exists(
    persistent_id_generator: Option<Res<PersistentIdGenerator>>,
    persistent_id_mapping: Option<Res<PersistentIdMapping>>,
) -> bool {
    persistent_id_generator.is_some() && persistent_id_mapping.is_some()
}

#[derive(
    Debug,
    Component,
    Default,
    Reflect,
    Serialize,
    Deserialize,
    Hash,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Copy,
    Clone,
)]
#[reflect(Component, Serialize, Deserialize, Hash)]
// YOU SHOULD NOT SET THIS it's for tests
pub struct PersistentId(u64);

impl PersistentId {
    pub fn value(&self) -> u64 {
        self.0
    }
}

#[derive(Resource, Default, Reflect, Serialize, Deserialize)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct PersistentIdGenerator {
    next_id: u64,
}

impl PersistentIdGenerator {
    pub fn next_id(&mut self) -> PersistentId {
        let id = self.next_id;
        self.next_id += 1;
        PersistentId(id)
    }
}

fn add_persistent_id_generator(mut commands: Commands) {
    commands.insert_resource(PersistentIdGenerator { next_id: 1 });
}

fn add_persistent_id(
    mut commands: Commands,
    mut persistent_id_generator: ResMut<PersistentIdGenerator>,
    mut persistent_id_mapping: ResMut<PersistentIdMapping>,
    to_populate: Query<Entity, (With<Save>, Without<PersistentId>)>,
) {
    for entity in to_populate.iter() {
        let next_id = persistent_id_generator.next_id();
        persistent_id_mapping
            .persistent_to_entity
            .insert(next_id.0, entity);
        commands.entity(entity).insert(next_id);
    }
}

#[derive(Resource, Default, Reflect, Serialize, Deserialize)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct PersistentIdMapping {
    persistent_to_entity: HashMap<u64, Entity>,
}

impl PersistentIdMapping {
    pub fn get(&self, id: PersistentId) -> Entity {
        // Just don't fail
        *self.persistent_to_entity.get(&id.0).unwrap()
    }

    // Fuck it
    pub fn insert(&mut self, entity: Entity, per_id: PersistentId) {
        self.persistent_to_entity.insert(per_id.0, entity);
    }
}

fn add_persistent_id_mapping(mut commands: Commands) {
    commands.insert_resource(PersistentIdMapping::default());
}

fn populate_persistent_id_mapping(
    mut persistent_id_mapping: ResMut<PersistentIdMapping>,
    to_update: Query<(Entity, &PersistentId), Or<(Added<PersistentId>, Changed<PersistentId>)>>,
) {
    for (entity, persistent_id) in &to_update {
        // This doubles up work for the gen but handles entities that are loaded in
        // I can't figure how to add a loaded component to components there were loaded Feel like an idiot
        persistent_id_mapping.insert(entity, *persistent_id);
    }
}
