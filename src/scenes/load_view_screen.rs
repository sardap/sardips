use bevy::prelude::*;
use bevy_turborand::{GlobalRng, RngComponent};

use crate::{
    food::template::FoodTemplateDatabase, game_zone::random_point_in_game_zone, name::EntityName,
    pet::template::PetTemplateDatabase, player::PlayerBundle, sardip_save::SardipLoadingState,
    text_database::text_keys::PET_STARTER_NAME, GameState,
};
use rand::prelude::IteratorRandom;

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
    food_template_db: Res<FoodTemplateDatabase>,
) {
    let mut rng = RngComponent::from(&mut global_rng);

    pet_template_db
        .get("Blob")
        .expect("Unable to find blob")
        .create_entity(
            &mut commands,
            &asset_server,
            &mut global_rng,
            &mut layouts,
            EntityName::new(PET_STARTER_NAME),
        );

    for _ in 0..10 {
        let template = food_template_db
            .iter()
            .choose(&mut rand::thread_rng())
            .unwrap();

        template.spawn(
            &mut commands,
            &asset_server,
            random_point_in_game_zone(&mut rng),
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
