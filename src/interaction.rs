use bevy::{prelude::*, window::PrimaryWindow};

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_world_mouse)
            .add_systems(
                First,
                (
                    update_world_mouse_positions::<MouseCamera>,
                    handle_interaction::<MouseCamera>,
                )
                    .chain(),
            )
            .add_systems(Update, attach_to_cursor);
    }
}

#[derive(Component)]
pub struct MouseCamera;

#[derive(Component)]
pub struct WorldMouse {
    pub last_position: Vec2,
}

impl WorldMouse {
    pub fn new() -> Self {
        Self {
            last_position: Vec2::ZERO,
        }
    }
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
    commands.spawn(WorldMouse::new());
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

fn handle_interaction<C: Component>(
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
pub struct AttachToCursor;

fn attach_to_cursor(
    mut query: Query<&mut Transform, With<AttachToCursor>>,
    world_mouse: Query<&WorldMouse>,
) {
    let world_mouse = world_mouse.single();

    for mut transform in query.iter_mut() {
        transform.translation.x = world_mouse.last_position.x;
        transform.translation.y = world_mouse.last_position.y;
    }
}
