use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct VelocityPlugin;

impl Plugin for VelocityPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Speed>()
            .add_systems(
                FixedUpdate,
                (fixed_apply_direction_2d, fixed_apply_direction_3d),
            )
            .add_systems(Update, update_apply_direction_3d);
    }
}

#[derive(Debug, Component, Default, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Speed(pub f32);

#[derive(Debug, Component, Default)]
pub struct MovementDirection2D {
    pub direction: Vec2,
}

fn fixed_apply_direction_2d(
    time: Res<Time>,
    mut query: Query<(&MovementDirection2D, &Speed, &mut Transform), Without<VelocityDeltaUpdate>>,
) {
    for (velocity, speed, mut transform) in query.iter_mut() {
        transform.translation += velocity.direction.extend(0.0) * speed.0 * time.delta_seconds();
    }
}

#[derive(Debug, Component, Default)]
pub struct MovementDirection3D {
    pub direction: Vec3,
}

fn fixed_apply_direction_3d(
    time: Res<Time>,
    mut query: Query<(&MovementDirection3D, &Speed, &mut Transform), Without<VelocityDeltaUpdate>>,
) {
    for (velocity, speed, mut transform) in query.iter_mut() {
        transform.translation += velocity.direction * speed.0 * time.delta_seconds();
    }
}

#[derive(Debug, Component, Default)]
pub struct VelocityDeltaUpdate;

fn update_apply_direction_3d(
    time: Res<Time>,
    mut query: Query<(&MovementDirection3D, &Speed, &mut Transform), With<VelocityDeltaUpdate>>,
) {
    for (velocity, speed, mut transform) in query.iter_mut() {
        transform.translation += velocity.direction * speed.0 * time.delta_seconds();
    }
}
