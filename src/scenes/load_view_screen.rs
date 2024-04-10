use bevy::prelude::*;

use crate::{
    pet::template::SpawnPetEvent, player::PlayerBundle, sardip_save::SardipLoadingState, GameState,
};

pub struct LoadViewScreenPlugin;

impl Plugin for LoadViewScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LoadViewScreen), setup)
            .add_systems(OnExit(GameState::LoadViewScreen), teardown)
            .add_systems(
                Update,
                setup_new_game
                    .run_if(in_state(GameState::LoadViewScreen))
                    .run_if(in_state(SardipLoadingState::Failed)),
            )
            .add_systems(
                Update,
                setup_complete
                    .run_if(in_state(GameState::LoadViewScreen))
                    .run_if(in_state(SardipLoadingState::Loaded)),
            );
    }
}

fn setup_new_game(
    mut spawn_pets: EventWriter<SpawnPetEvent>,
    mut game_state: ResMut<NextState<GameState>>,
    mut loading_state: ResMut<NextState<SardipLoadingState>>,
) {
    for _ in 0..2 {
        spawn_pets.send(SpawnPetEvent::Blank((
            Vec2::new(0., 0.),
            "Blob".to_string(),
        )));
    }

    game_state.set(GameState::ViewScreen);
    loading_state.set(SardipLoadingState::None);
}

fn setup(mut commands: Commands, mut loading_state: ResMut<NextState<SardipLoadingState>>) {
    commands.spawn(PlayerBundle::default());
    loading_state.set(SardipLoadingState::Loading);
}

fn setup_complete(
    mut game_state: ResMut<NextState<GameState>>,
    mut loading_state: ResMut<NextState<SardipLoadingState>>,
) {
    game_state.set(GameState::ViewScreen);
    loading_state.set(SardipLoadingState::None);
}

fn teardown() {}
