pub mod ancestor;
pub mod breeding;
pub mod core;
pub mod dipdex;
pub mod evolve;
pub mod fun;
pub mod hunger;
pub mod mood;
pub mod move_towards;
pub mod pet_ai;
pub mod poop;
pub mod template;
pub mod wonder;

use bevy::prelude::*;
pub use core::*;

use self::{
    breeding::BreedPlugin, evolve::EvolvePlugin, fun::FunPlugin, hunger::HungerPlugin,
    mood::MoodPlugin, move_towards::MoveTowardsPlugin, pet_ai::PetAiPlugin, poop::PoopPlugin,
    template::PetTemplatePlugin, wonder::WonderPlugin,
};

pub struct PetPlugin;

impl Plugin for PetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            HungerPlugin,
            MoodPlugin,
            MoveTowardsPlugin,
            PetAiPlugin,
            PoopPlugin,
            PetTemplatePlugin,
            WonderPlugin,
            FunPlugin,
            EvolvePlugin,
            BreedPlugin,
        ));
    }
}
