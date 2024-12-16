use bevy::prelude::*;
use shared_deps::bevy_parallax::ParallaxMoveEvent;

pub struct AutoScrollPlugin;

impl Plugin for AutoScrollPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, auto_scroll_system);
    }
}

#[derive(Debug, Component)]
pub struct AutoScroll {
    pub speed: Vec2,
}

impl AutoScroll {
    pub fn new(speed: Vec2) -> Self {
        Self { speed }
    }
}

pub fn auto_scroll_system(
    time: Res<Time>,
    query: Query<(Entity, &AutoScroll), With<Camera>>,
    mut move_event_writer: EventWriter<ParallaxMoveEvent>,
) {
    for (camera, auto_scroll) in query.iter() {
        move_event_writer.send(ParallaxMoveEvent {
            camera,
            translation: auto_scroll.speed * time.delta_seconds(),
            rotation: 0.,
        });
    }
}
