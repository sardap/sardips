use std::path::Path;

use bevy::prelude::*;
use moonshine_save::prelude::*;

use crate::GameState;

pub struct SardipSavePlugin;

impl Plugin for SardipSavePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SavePlugin, LoadPlugin))
            .insert_state(SardipLoadingState::default())
            .add_systems(
                PreUpdate,
                save_default().into(file_from_resource::<SaveRequest>()),
            )
            .add_systems(PreUpdate, load(file_from_resource::<LoadRequest>()))
            .add_systems(OnEnter(SardipLoadingState::Loading), trigger_load)
            .add_systems(Update, trigger_save.run_if(in_state(GameState::ViewScreen)))
            .add_systems(
                Update,
                post_load.run_if(
                    in_state(SardipLoadingState::Loading)
                        .and_then(not(resource_exists::<LoadRequest>)),
                ),
            );
    }
}

#[derive(Resource)]
struct SaveRequest;

impl GetFilePath for SaveRequest {
    fn path(&self) -> &Path {
        SAVE_PATH.as_ref()
    }
}

#[derive(Resource)]
struct LoadRequest;

impl GetFilePath for LoadRequest {
    fn path(&self) -> &Path {
        SAVE_PATH.as_ref()
    }
}

struct SaveTimer {
    timer: Timer,
}

impl Default for SaveTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(5., TimerMode::Repeating),
        }
    }
}

fn trigger_save(mut commands: Commands, time: Res<Time>, mut save_timer: Local<SaveTimer>) {
    if save_timer.timer.tick(time.delta()).just_finished() {
        commands.insert_resource(SaveRequest);
    }
}

fn trigger_load(mut commands: Commands) {
    commands.insert_resource(LoadRequest);
}

fn post_load(mut state: ResMut<NextState<SardipLoadingState>>) {
    state.set(SardipLoadingState::Loaded);
}

#[derive(Debug, States, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SardipLoadingState {
    #[default]
    None,
    Loading,
    Loaded,
    Failed,
}

const SAVE_PATH: &str = "sardip_save.ron";
