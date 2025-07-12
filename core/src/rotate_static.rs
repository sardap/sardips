use bevy::prelude::*;
use shared_deps::bevy_turborand::{DelegatedRng, GenCore, GlobalRng};

pub struct RotateStaticPlugin;

impl Plugin for RotateStaticPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, rotate_static);
    }
}

#[derive(Component)]
pub struct RotateStatic {
    timer: Timer,
}

impl Default for RotateStatic {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
        }
    }
}

fn rotate_static(
    time: Res<Time>,
    mut rand: ResMut<GlobalRng>,
    mut rotate: Query<(&mut TextureAtlas, &mut RotateStatic)>,
) {
    let rand = rand.get_mut();

    for (mut layout, mut rotate) in rotate.iter_mut() {
        if rotate.timer.tick(time.delta()).just_finished() {
            layout.index = rand.gen_usize() % 64;
        }
    }
}
