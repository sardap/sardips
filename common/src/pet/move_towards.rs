use bevy::prelude::*;

use crate::velocity::MovementDirection;

pub struct MoveTowardsPlugin;

impl Plugin for MoveTowardsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MoveTowardsEvent>().add_systems(
            FixedUpdate,
            (
                set_target,
                check_target_reached,
                on_remove_moving_towards,
                on_spawn_move_towards,
            ),
        );
    }
}

#[derive(Component)]
pub struct MovingTowards {
    target: Vec2,
}

impl MovingTowards {
    fn new(target: Vec2) -> Self {
        Self { target }
    }
}

#[derive(Debug, Event, Clone, Copy)]
pub struct MoveTowardsEvent {
    pub entity: Entity,
    pub target: Vec2,
}

impl MoveTowardsEvent {
    pub fn new(entity: Entity, target: Vec2) -> Self {
        Self { entity, target }
    }
}

fn set_target(
    mut commands: Commands,
    mut move_target_events_reader: EventReader<MoveTowardsEvent>,
    mut query: Query<(
        &Transform,
        &mut MovementDirection,
        Option<&mut MovingTowards>,
    )>,
) {
    for event in move_target_events_reader.read() {
        if let Ok((transform, mut move_direction, moving_towards)) = query.get_mut(event.entity) {
            let mut direction = event.target - transform.translation.truncate();
            direction = direction.normalize();
            move_direction.direction = direction;

            if let Some(mut moving_towards) = moving_towards {
                moving_towards.target = event.target;
            } else {
                commands
                    .entity(event.entity)
                    .insert(MovingTowards::new(event.target));
            }
        }
    }
}

#[derive(Component)]
pub struct MoveTowardsOnSpawn {
    pub target: Vec2,
}

fn on_spawn_move_towards(
    mut commands: Commands,
    mut query: Query<(Entity, &MoveTowardsOnSpawn)>,
    mut move_target_events: EventWriter<MoveTowardsEvent>,
) {
    for (entity, move_towards) in query.iter_mut() {
        move_target_events.send(MoveTowardsEvent::new(entity, move_towards.target));
        commands.entity(entity).remove::<MoveTowardsOnSpawn>();
    }
}

fn check_target_reached(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &MovingTowards,
        &mut Transform,
        &mut MovementDirection,
    )>,
) {
    for (entity, move_towards, mut transform, mut move_direction) in query.iter_mut() {
        let distance = transform
            .translation
            .truncate()
            .distance(move_towards.target);
        if distance < 5.0 {
            move_direction.direction = Vec2::ZERO;
            transform.translation.x = move_towards.target.x;
            transform.translation.y = move_towards.target.y;
            commands.entity(entity).remove::<MovingTowards>();
        }
    }
}

fn on_remove_moving_towards(
    mut removed: RemovedComponents<MovingTowards>,
    mut query: Query<&mut MovementDirection>,
) {
    for entity in removed.read() {
        if let Ok(mut move_direction) = query.get_mut(entity) {
            move_direction.direction = Vec2::ZERO;
        }
    }
}
