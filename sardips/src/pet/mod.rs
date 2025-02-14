pub mod breeding;
pub mod core;
pub mod dipdex;
pub mod evolve;
pub mod fun;
pub mod hunger;
pub mod mood;
pub mod pet_ai;
pub mod poop;
pub mod template;
pub mod view;
pub mod wonder;

use bevy::prelude::*;
pub use core::*;
use mood::{Mood, MoodHunger, MoodState};
use view::PetViewPlugin;

use self::{
    breeding::BreedPlugin, evolve::EvolvePlugin, fun::FunPlugin, hunger::HungerPlugin,
    mood::MoodPlugin, pet_ai::PetAiPlugin, poop::PoopPlugin, template::PetTemplatePlugin,
    wonder::WonderPlugin,
};

pub struct PetPlugin;

impl Plugin for PetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Pet>()
            .register_type::<MoodState>()
            .register_type::<Option<MoodState>>()
            .register_type::<Mood>()
            .register_type::<MoodHunger>()
            .add_plugins((
                HungerPlugin,
                MoodPlugin,
                PetAiPlugin,
                PoopPlugin,
                PetTemplatePlugin,
                WonderPlugin,
                FunPlugin,
                EvolvePlugin,
                BreedPlugin,
                PetViewPlugin,
            ));
    }
}
