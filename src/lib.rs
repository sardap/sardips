#![feature(const_mut_refs)]
#![feature(effects)]
#![feature(const_trait_impl)]
#![feature(const_for)]
pub mod age;
pub mod anime;
pub mod assets;
pub mod autoscroll;
pub mod button_hover;
pub mod debug;
pub mod dynamic_dialogue;
pub mod food;
pub mod game_zone;
pub mod interaction;
pub mod layering;
pub mod minigames;
pub mod money;
pub mod name;
pub mod palettes;
pub mod pet;
pub mod player;
pub mod sardip_save;
pub mod scenes;
pub mod simulation;
pub mod sounds;
pub mod text_database;
pub mod text_translation;
pub mod thinking;
pub mod tools;
pub mod velocity;

use crate::name::NamePlugin;
use age::AgePlugin;
use anime::AnimePlugin;
use autoscroll::AutoScrollPlugin;
use bevy::{asset::AssetMetaCheck, prelude::*, window::WindowResolution};
use bevy_kira_audio::prelude::*;
use bevy_parallax::ParallaxPlugin;
use bevy_prototype_lyon::prelude::*;
use bevy_turborand::prelude::*;
use button_hover::ButtonHoverPlugin;
use debug::DebugPlugin;
use dynamic_dialogue::DynamicDialoguePlugin;
use food::template::FoodTemplatePlugin;
use interaction::InteractionPlugin;
use minigames::MinigamePlugin;
use money::MoneyPlugin;
use pet::PetPlugin;
use player::PlayerPlugin;
use sardip_save::SardipSavePlugin;
use scenes::GameScenePlugin;
use simulation::{SimulationPlugin, SimulationState};
use sounds::SoundsPlugin;
use text_database::TextDatabasePlugin;
use text_translation::TextTranslationPlugin;
use thinking::ThinkingPlugin;
use tools::{poop_scooper::PoopScooperPlugin, ToolPlugin};
use velocity::VelocityPlugin;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate maplit;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(States, Clone, Copy, Default, Eq, PartialEq, Hash, Debug)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    LoadViewScreen,
    ViewScreen,
    MiniGame,
}

pub fn despawn_all<C: Component>(mut commands: Commands, query: Query<Entity, With<C>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn remove_resource<T: Resource>(mut commands: Commands) {
    debug!("Removing resource: {:?}", std::any::type_name::<T>());
    commands.remove_resource::<T>();
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state::<GameState>(GameState::default())
            .insert_resource(AssetMetaCheck::Never)
            .add_plugins((
                DefaultPlugins.set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(500.0, 700.0),
                        title: format!("{} v{}", NAME, VERSION),
                        canvas: Some("#bevy".to_owned()),
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                }),
                ParallaxPlugin,
                RngPlugin::default(),
                AudioPlugin,
                ShapePlugin,
            ))
            .add_plugins((
                SardipSavePlugin,
                SimulationPlugin,
                AutoScrollPlugin,
                TextDatabasePlugin,
                ButtonHoverPlugin,
                PetPlugin,
                NamePlugin,
                VelocityPlugin,
                FoodTemplatePlugin,
                InteractionPlugin,
                MinigamePlugin,
                MoneyPlugin,
                PlayerPlugin,
                AgePlugin,
            ))
            .add_plugins((
                AnimePlugin,
                ToolPlugin,
                PoopScooperPlugin,
                SoundsPlugin,
                GameScenePlugin,
                DynamicDialoguePlugin,
                ThinkingPlugin,
                TextTranslationPlugin,
            ));

        #[cfg(feature = "dev")]
        app.add_plugins(DebugPlugin);
    }
}