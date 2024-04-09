use bevy::prelude::*;
use bevy_turborand::{GlobalRng, RngComponent};

use crate::{
    food::template::FoodTemplateDatabase, game_zone::random_point_in_game_zone, name::EntityName,
    pet::template::PetTemplateDatabase, player::PlayerBundle, sardip_save::SardipLoadingState,
    text_database::TextDatabase, GameState,
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
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    mut loading_state: ResMut<NextState<SardipLoadingState>>,
    mut global_rng: ResMut<GlobalRng>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    pet_template_db: Res<PetTemplateDatabase>,
    text_db: Res<TextDatabase>,
) {
    for _ in 0..2 {
        pet_template_db
            .get_by_name("Blob")
            .expect("Unable to find blob")
            .create_entity(
                &mut commands,
                &asset_server,
                &mut global_rng,
                &mut layouts,
                Vec2::new(0.0, 0.0),
                EntityName::random(&text_db),
            );
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
