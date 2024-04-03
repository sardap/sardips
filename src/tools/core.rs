use bevy::prelude::*;

use crate::interaction::WorldMouse;

pub struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, attach_to_cursor);
    }
}

pub const TOOL_SIZE: Vec2 = Vec2::new(32.0, 32.0);

#[derive(Component)]
pub struct Tool;

fn attach_to_cursor(
    mut tool_query: Query<(&Tool, &mut Transform)>,
    world_mouse: Query<&WorldMouse>,
) {
    let world_mouse = world_mouse.single();

    for (_, mut transform) in tool_query.iter_mut() {
        transform.translation.x = world_mouse.last_position.x;
        transform.translation.y = world_mouse.last_position.y;
    }
}
