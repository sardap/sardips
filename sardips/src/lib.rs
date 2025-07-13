#![allow(clippy::needless_lifetimes)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(unexpected_cfgs)]
#![feature(const_trait_impl)]
#![feature(const_for)]
#![feature(duration_constructors)]
pub mod accessory;
pub mod age;
pub mod anime;
pub mod debug;
pub mod dynamic_dialogue;
pub mod fact_update;
pub mod food;
pub mod game_zone;
pub mod inventory;
pub mod layering;
pub mod minigames;
pub mod money;
pub mod palettes;
pub mod pet;
pub mod pet_display;
pub mod player;
pub mod sardip_save;
pub mod scenes;
pub mod simulation;
pub mod stock_market;
pub mod stock_ticker;
pub mod thinking;
pub mod tools;

use accessory::AccessoryPlugin;
use age::AgePlugin;
use anime::AnimePlugin;
use bevy::{asset::AssetMetaCheck, prelude::*, window::WindowResolution};
use debug::DebugPlugin;
use dynamic_dialogue::DynamicDialoguePlugin;
use fact_db::FactsPlugin;
use fact_update::FactUpdatePlugin;
use food::{template::FoodTemplatePlugin, FoodPlugin};
use inventory::InventoryPlugin;
use minigames::MinigamePlugin;
use money::MoneyPlugin;
use pet::{dipdex::DipdexPlugin, PetPlugin};
use pet_display::PetPreviewPlugin;
use player::PlayerPlugin;
use sardip_save::SardipSavePlugin;
use scenes::GameScenePlugin;
use shared_deps::bevy_kira_audio::prelude::*;
use shared_deps::bevy_parallax::ParallaxPlugin;
use shared_deps::bevy_prototype_lyon::prelude::*;
use shared_deps::bevy_turborand::prelude::*;
use simulation::{SimulationPlugin, SimulationState};
use stock_market::StockMarketPlugin;
use stock_ticker::StockTickerPlugin;
use thinking::ThinkingPlugin;
use tools::{poop_scooper::PoopScooperPlugin, ToolPlugin};

#[macro_use]
extern crate strum_macros;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate maplit;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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
            shared_deps::avian2d::PhysicsPlugins::new(FixedUpdate),
            shared_deps::avian3d::PhysicsPlugins::new(FixedUpdate),
            shared_deps::avian3d::prelude::PhysicsDebugPlugin::default(),
            FactsPlugin,
            sardips_core::SardipsCorePlugin,
            shared_deps::bevy_rts_camera::RtsCameraPlugin,
            // shared_deps::avian2d::prelude::PhysicsDebugPlugin::default(),
        ))
        .add_plugins((
            SardipSavePlugin,
            PetPlugin,
            SimulationPlugin,
            FoodTemplatePlugin,
            MinigamePlugin,
            MoneyPlugin,
            PlayerPlugin,
            AgePlugin,
        ))
        .add_plugins((
            AnimePlugin,
            ToolPlugin,
            PoopScooperPlugin,
            GameScenePlugin,
            DynamicDialoguePlugin,
            FactUpdatePlugin,
            ThinkingPlugin,
            FoodPlugin,
            StockMarketPlugin,
            StockTickerPlugin,
            DipdexPlugin,
            AccessoryPlugin,
            InventoryPlugin,
            PetPreviewPlugin,
        ));

        // #[cfg(feature = "dev")]
        app.add_plugins(DebugPlugin);
    }
}
