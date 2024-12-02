use bevy::{prelude::*, window::PrimaryWindow};
use shared_deps::avian2d::prelude::LinearVelocity;

use crate::velocity::Speed;

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_world_mouse)
            .add_systems(
                First,
                (
                    update_world_mouse_positions::<MouseCamera>,
                    handle_interaction,
                )
                    .chain(),
            )
            .add_systems(Update, (attach_to_cursor, move_towards_avian_cursor));
    }
}

#[derive(Component)]
pub struct MouseCamera;

#[derive(Component, Default)]
pub struct WorldMouse {
    pub last_position: Vec2,
}

#[derive(Component, Default)]
pub struct Clickable {
    pub width: Vec2,
    pub height: Vec2,
}

impl Clickable {
    pub fn new(width: Vec2, height: Vec2) -> Self {
        Self { width, height }
    }
}

#[derive(Component)]
pub struct Hovering;

fn spawn_world_mouse(mut commands: Commands) {
    commands.spawn(WorldMouse::default());
}

fn update_world_mouse_positions<C: Component>(
    mut world_mouse: Query<&mut WorldMouse>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<C>>,
) {
    let (camera, camera_transform) = match camera.get_single() {
        Ok(val) => val,
        Err(_) => return,
    };
    let window = match window.get_single() {
        Ok(val) => val,
        Err(_) => return,
    };

    let mut world_mouse = world_mouse.single_mut();

    if let Some(mouse_position) = window.cursor_position().and_then(|cursor| {
        camera
            .viewport_to_world(camera_transform, cursor)
            .map(|ray| ray.origin.truncate())
    }) {
        world_mouse.last_position = mouse_position;
    }
}

fn handle_interaction(
    mut commands: Commands,
    world_mouse: Query<&WorldMouse>,
    clickables: Query<(Entity, &GlobalTransform, &Clickable)>,
) {
    let world_mouse = world_mouse.single();

    for (entity, transform, clickable) in clickables.iter() {
        let x = transform.translation().x;
        let y = transform.translation().y;

        let min = Vec2::new(clickable.width.x, clickable.height.x);
        let max = Vec2::new(clickable.width.y, clickable.height.y);

        if x + min.x <= world_mouse.last_position.x
            && world_mouse.last_position.x <= x + max.x
            && y + min.y <= world_mouse.last_position.y
            && world_mouse.last_position.y <= y + max.y
        {
            commands.entity(entity).insert(Hovering);
        } else {
            commands.entity(entity).remove::<Hovering>();
        }
    }
}

#[derive(Component)]
pub struct AttachToCursor {
    pub attach_x: bool,
    pub attach_y: bool,
}

impl Default for AttachToCursor {
    fn default() -> Self {
        Self {
            attach_x: true,
            attach_y: true,
        }
    }
}

impl AttachToCursor {
    pub fn new() -> Self {
        Self {
            attach_x: false,
            attach_y: false,
        }
    }

    pub fn attach_x(mut self, attach_x: bool) -> Self {
        self.attach_x = attach_x;
        self
    }

    pub fn attach_y(mut self, attach_y: bool) -> Self {
        self.attach_y = attach_y;
        self
    }
}

fn attach_to_cursor(
    mut query: Query<(&mut Transform, &AttachToCursor)>,
    world_mouse: Query<&WorldMouse>,
) {
    let world_mouse = world_mouse.single();

    for (mut transform, attach) in query.iter_mut() {
        if attach.attach_x {
            transform.translation.x = world_mouse.last_position.x;
        }
        if attach.attach_y {
            transform.translation.y = world_mouse.last_position.y;
        }
    }
}

#[derive(Component, Default)]
pub struct MoveTowardsCursor {
    pub x: bool,
    pub y: bool,
}

impl MoveTowardsCursor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_x(mut self, x: bool) -> Self {
        self.x = x;
        self
    }

    pub fn with_y(mut self, y: bool) -> Self {
        self.y = y;
        self
    }
}

fn move_towards_avian_cursor(
    world_mouse: Query<&WorldMouse>,
    mut query: Query<(
        &mut LinearVelocity,
        &Speed,
        &MoveTowardsCursor,
        &GlobalTransform,
    )>,
) {
    let world_mouse = world_mouse.single();

    for (mut velocity, speed, move_towards_cursor, trans) in query.iter_mut() {
        let direction = Vec2::normalize(world_mouse.last_position - trans.translation().xy());
        if move_towards_cursor.x {
            velocity.0.x = direction.x * speed.0;
        }
        if move_towards_cursor.y {
            velocity.0.y = direction.y * speed.0;
        }
    }
}
