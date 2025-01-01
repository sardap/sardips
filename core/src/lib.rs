#![allow(clippy::needless_lifetimes)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(unexpected_cfgs)]
use bevy::prelude::*;
use shared_deps::{
    bevy_turborand::DelegatedRng,
    moonshine_save::{load::LoadPlugin, save::SavePlugin},
};
use std::{ops::Range, time::Duration};

pub mod assets;
pub mod autoscroll;
pub mod button_hover;
pub mod interaction;
pub mod loading;
pub mod minigames_core;
pub mod mood_core;
pub mod move_towards;
pub mod particles;
pub mod shrink;
pub mod sounds;
pub mod text_database;
pub mod text_translation;
pub mod velocity;

#[macro_use]
extern crate lazy_static;

pub struct SardipsCorePlugin;

impl Plugin for SardipsCorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(GameState::default())
            .add_plugins((SavePlugin, LoadPlugin))
            .add_plugins((
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
