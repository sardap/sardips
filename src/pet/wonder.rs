use crate::{game_zone::random_point_in_game_zone, pet::move_towards::MoveTowardsEvent};
use bevy::prelude::*;
use bevy_turborand::prelude::*;

use super::move_towards::MovingTowards;

pub struct WonderPlugin;

impl Plugin for WonderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (new_target, on_remove_wonder));
    }
}

#[derive(Debug, Component, Default)]
pub struct Wonder;

fn new_target(
    mut move_towards_events: EventWriter<MoveTowardsEvent>,
    mut wonderers: Query<(Entity, &mut RngComponent), (With<Wonder>, Without<MovingTowards>)>,
) {
    for (entity, mut rng) in wonderers.iter_mut() {
        let target = random_point_in_game_zone(&mut rng);
        move_towards_events.send(MoveTowardsEvent::new(entity, target));
    }
}

fn on_remove_wonder(mut commands: Commands, mut removed: RemovedComponents<Wonder>) {
    for entity in removed.read() {
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.remove::<MovingTowards>();
        }
    }
}
