use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct VelocityPlugin;

impl Plugin for VelocityPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Speed>()
            .add_systems(FixedUpdate, apply_direction);
    }
}

#[derive(Debug, Component, Default, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Speed(pub f32);

#[derive(Debug, Component, Default)]
pub struct MovementDirection {
    pub direction: Vec2,
}

fn apply_direction(
    time: Res<Time>,
    mut query: Query<(&MovementDirection, &Speed, &mut Transform)>,
) {
    for (velocity, speed, mut transform) in query.iter_mut() {
        transform.translation += velocity.direction.extend(0.0) * speed.0 * time.delta_seconds();
    }
}
