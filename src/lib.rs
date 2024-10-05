#![allow(clippy::large_enum_variant)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![feature(const_trait_impl)]
#![feature(const_for)]
pub mod age;
pub mod anime;
pub mod assets;
pub mod autoscroll;
pub mod button_hover;
pub mod debug;
pub mod dynamic_dialogue;
pub mod facts;
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
pub mod stock_market;
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
use facts::FactsPlugin;
use food::{template::FoodTemplatePlugin, FoodPlugin};
use interaction::InteractionPlugin;
use minigames::MinigamePlugin;
use money::MoneyPlugin;
use pet::{dipdex::DipdexPlugin, PetPlugin};
use player::PlayerPlugin;
use sardip_save::SardipSavePlugin;
use scenes::GameScenePlugin;
use simulation::{SimulationPlugin, SimulationState};
use sounds::SoundsPlugin;
use stock_market::StockMarketPlugin;
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
    Template,
    DipdexView,
    FoodBuy,
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
        app.add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
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
        .insert_state(GameState::default())
        .add_plugins((
            PetPlugin,
            SardipSavePlugin,
            SimulationPlugin,
            AutoScrollPlugin,
            TextDatabasePlugin,
            ButtonHoverPlugin,
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
            FactsPlugin,
            ThinkingPlugin,
            TextTranslationPlugin,
            FoodPlugin,
            StockMarketPlugin,
            DipdexPlugin,
        ));

        // #[cfg(feature = "dev")]
        app.add_plugins(DebugPlugin);
    }
}
