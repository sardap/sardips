use bevy::prelude::*;

use super::MiniGameState;

pub struct TemplatePlugin;

impl Plugin for TemplatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(TemplateState::default())
            .add_systems(
                OnEnter(MiniGameState::PlayingTemplate),
                (setup_camera, setup_game),
            )
            .add_systems(OnExit(MiniGameState::PlayingTemplate), teardown);
    }
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum TemplateState {
    #[default]
    None,
}

#[derive(Component)]
struct Template;

#[derive(Component)]
struct TemplateCamera;

fn setup_camera(mut commands: Commands) {
    let camera: Entity = commands
        .spawn((Camera2dBundle::default(), Template, TemplateCamera))
        .id();
}

fn setup_game(mut commands: Commands) {}

fn teardown(mut commands: Commands, to_delete: Query<Entity, With<Template>>) {
    for entity in to_delete.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
