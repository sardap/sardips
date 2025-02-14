use std::time::Duration;

use bevy::prelude::*;

pub struct ShrinkPlugin;

impl Plugin for ShrinkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_shrink);
    }
}

#[derive(Component)]
pub struct Shrinking {
    starting_size: Vec2,
    time: Timer,
}

impl Shrinking {
    pub fn new(size: Vec2, duration: Duration) -> Self {
        Self {
            starting_size: size,
            time: Timer::new(duration, TimerMode::Once),
        }
    }
}

fn update_shrink(
    mut commands: Commands,
    time: Res<Time>,
    mut shrink: Query<(Entity, &mut Shrinking, &mut Transform)>,
) {
    for (entity, mut shrink, mut trans) in &mut shrink {
        if shrink.time.tick(time.delta()).finished() {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        let percent = shrink.time.elapsed().as_secs_f32() / shrink.time.duration().as_secs_f32();
        trans.scale = Vec3::new(
            shrink.starting_size.x * (1. - percent),
            shrink.starting_size.y * (1. - percent),
            1.,
        );
    }
}
