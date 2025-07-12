#![allow(clippy::needless_lifetimes)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(unexpected_cfgs)]
#![feature(hash_raw_entry)]
#![feature(const_trait_impl)]
#![feature(let_chains)]
use bevy::prelude::*;
use shared_deps::{
    bevy_turborand::DelegatedRng,
    moonshine_save::{load::LoadPlugin, save::SavePlugin},
};
use std::{ops::Range, time::Duration};

pub mod accessory_core;
pub mod age_core;
pub mod assets;
pub mod autoscroll;
pub mod breeding_core;
pub mod button_hover;
pub mod color_utils;
pub mod food_core;
pub mod fun_core;
pub mod hunger_core;
pub mod interaction;
pub mod loading;
pub mod minigames_core;
pub mod money_core;
pub mod mood_core;
pub mod move_towards;
pub mod name;
pub mod particles;
pub mod persistent_id;
pub mod pet_core;
pub mod rand_utils;
pub mod rotate_static;
pub mod shrink;
pub mod sounds;
pub mod sprite_utils;
pub mod text_database;
pub mod text_translation;
pub mod ui_utils;
pub mod velocity;
pub mod view;
pub mod wrapped_vec;

#[macro_use]
extern crate lazy_static;

pub struct SardipsCorePlugin;

impl Plugin for SardipsCorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(GameState::default())
            .add_plugins((SavePlugin, LoadPlugin, bevy_http_client::HttpClientPlugin))
            .add_plugins((
                persistent_id::PersistentIdPlugin,
                loading::LoadingPlugin,
                particles::ParticlesPlugin,
                shrink::ShrinkPlugin,
                button_hover::ButtonHoverPlugin,
                autoscroll::AutoScrollPlugin,
                sounds::SoundsPlugin,
                velocity::VelocityPlugin,
                interaction::InteractionPlugin,
                minigames_core::MiniGamesCorePlugin,
                text_database::TextDatabasePlugin,
                move_towards::MoveTowardsPlugin,
                text_translation::TextTranslationPlugin,
                age_core::AgeCorePlugin,
                breeding_core::BreedingCorePlugin,
            ))
            .add_plugins((
                food_core::FoodCorePlugin,
                fun_core::FunCorePlugin,
                hunger_core::HungerCorePlugin,
                money_core::MoneyCorePlugin,
                name::NamePlugin,
                pet_core::PetCorePlugin,
                view::ViewPlugin,
                mood_core::MoodCorePlugin,
                accessory_core::AccessoryCorePlugin,
                rotate_static::RotateStaticPlugin,
            ));
    }
}

#[derive(States, Clone, Copy, Default, Eq, PartialEq, Hash, Debug)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    LoadViewScreen,
    ViewScreen,
    MiniGame,
    Template,
    BuyAccessory,
    DipdexView,
    FoodBuy,
    StockBuy,
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

pub fn random_choose<'a, T, R: DelegatedRng>(rng: &mut R, items: &'a [T]) -> &'a T {
    items.get(rng.usize(0..items.len())).unwrap()
}

#[macro_export]
macro_rules! rgba_to_color {
    ($r:expr, $g:expr, $b:expr, $a:expr) => {{
        // Normalize the RGB components to floating-point values in the range [0, 1]
        let normalized_r = $r as f32 / 255.0;
        let normalized_g = $g as f32 / 255.0;
        let normalized_b = $b as f32 / 255.0;
        let normalized_a = $a as f32 / 255.0;

        bevy::color::Color::Srgba(bevy::color::Srgba {
            red: normalized_r,
            green: normalized_g,
            blue: normalized_b,
            alpha: normalized_a,
        })
    }};
}

#[macro_export]
macro_rules! rgb_to_color {
    ($r:expr, $g:expr, $b:expr) => {{
        // Normalize the RGB components to floating-point values in the range [0, 1]
        let normalized_r = $r as f32 / 255.0;
        let normalized_g = $g as f32 / 255.0;
        let normalized_b = $b as f32 / 255.0;
        let normalized_a = 1.;

        bevy::color::Color::Srgba(bevy::color::Srgba {
            red: normalized_r,
            green: normalized_g,
            blue: normalized_b,
            alpha: normalized_a,
        })
    }};
}

pub struct VaryingTimer {
    timer: Timer,
    pub range: Range<u64>,
    pub modifier: f64,
    pub times_finished: u32,
}

impl VaryingTimer {
    pub fn new<T: DelegatedRng>(range: Range<Duration>, rng: &mut T) -> Self {
        let mut result = Self {
            timer: Timer::new(range.start, TimerMode::Once),
            range: range.start.as_micros() as u64..range.end.as_micros() as u64,
            modifier: 1.,
            times_finished: 0,
        };
        result.tick(result.timer.duration(), rng);
        result
    }

    pub fn with_modifier(mut self, modifier: f64) -> Self {
        self.modifier = modifier;
        self
    }

    pub fn tick<T: DelegatedRng>(&mut self, delta: Duration, rng: &mut T) -> &VaryingTimer {
        self.times_finished = self.timer.tick(delta).times_finished_this_tick();
        if self.times_finished > 0 {
            let mut micros = rng.u64(self.range.clone());
            if self.modifier > 1. {
                micros = (micros as f64 / self.modifier) as u64;
            }

            let duration = Duration::from_micros(micros);
            self.timer = Timer::new(duration, TimerMode::Once);
        }
        self
    }

    pub fn just_finished(&self) -> bool {
        self.times_finished > 0
    }

    pub fn times_finished_this_tick(&self) -> u32 {
        self.times_finished
    }
}

pub const fn from_mins(mins: u64) -> Duration {
    Duration::from_secs(60 * mins)
}

pub const fn from_hours(hours: u64) -> Duration {
    from_mins(60 * hours)
}

pub const fn from_days(days: u64) -> Duration {
    from_hours(24 * days)
}

pub const MOOD_HISTORY_UPDATE: Duration = from_mins(5);
pub const BREED_RESET_INTERVAL: Duration = from_mins(30);
