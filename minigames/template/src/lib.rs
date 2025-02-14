use bevy::prelude::*;
use sardips_core::{
    despawn_all,
    minigames_core::{MiniGameCompleted, MiniGameResult, MiniGameState, MiniGameType},
};

pub struct ReplaceMePlugin;

impl Plugin for ReplaceMePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(ReplaceMeState::default())
            .add_systems(OnEnter(MiniGameState::PlayingReplaceMe), on_start_playing)
            .add_systems(
                OnExit(MiniGameState::PlayingReplaceMe),
                despawn_all::<ReplaceMe>,
            )
            .add_systems(OnEnter(ReplaceMeState::Loading), setup_loading)
            .add_systems(
                Update,
                check_loading.run_if(in_state(ReplaceMeState::Loading)),
            )
            .add_systems(
                OnExit(ReplaceMeState::Loading),
                despawn_all::<ReplaceMeLoading>,
            )
            .add_systems(OnEnter(ReplaceMeState::Playing), setup_camera_and_ui)
            .add_systems(
                OnExit(ReplaceMeState::Playing),
                despawn_all::<ReplaceMePlaying>,
            )
            .add_systems(OnEnter(ReplaceMeState::Score), setup_score_screen)
            .add_systems(OnExit(ReplaceMeState::Score), despawn_all::<ReplaceMeScore>)
            .add_systems(
                Update,
                quit_button_pressed.run_if(in_state(ReplaceMeState::Score)),
            )
            .add_systems(
                OnEnter(ReplaceMeState::Exit),
                (despawn_all::<ReplaceMe>, on_exit),
            );
    }
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum ReplaceMeState {
    #[default]
    None,
    Loading,
    Playing,
    Score,
    Exit,
}

#[derive(Component)]
struct ReplaceMe;

fn on_start_playing(mut state: ResMut<NextState<ReplaceMeState>>) {
    state.set(ReplaceMeState::Loading);
}

#[derive(Component)]
struct ReplaceMeLoading;

fn setup_loading() {}

fn check_loading(mut state: ResMut<NextState<ReplaceMeState>>) {
    state.set(ReplaceMeState::Playing);
}

#[derive(Component)]
struct ReplaceMePlaying;

fn setup_camera_and_ui() {}

#[derive(Component)]
struct ReplaceMeScore;

fn setup_score_screen() {}

#[derive(Component)]
struct QuitButton;

fn quit_button_pressed(
    mut state: ResMut<NextState<ReplaceMeState>>,
    quit_buttons: Query<&Interaction, (With<QuitButton>, Changed<Interaction>)>,
) {
    for interaction in &quit_buttons {
        if interaction == &Interaction::Pressed {
            state.set(ReplaceMeState::Exit);
        }
    }
}

fn on_exit(
    mut state: ResMut<NextState<MiniGameState>>,
    mut event_writer: EventWriter<MiniGameCompleted>,
) {
    state.set(MiniGameState::None);

    let score = 0.;

    event_writer.send(MiniGameCompleted {
        game_type: MiniGameType::Translate,
        result: if score > 10000. {
            MiniGameResult::Lose
        } else if score > 5000. {
            MiniGameResult::Draw
        } else {
            MiniGameResult::Win
        },
    });
}
