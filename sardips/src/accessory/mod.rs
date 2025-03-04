use bevy::prelude::*;
use sardips_core::{accessory_core::AnchorPointSet, particles::Spewer};
use serde::{Deserialize, Serialize};
use shared_deps::moonshine_save::save::Save;
use view::AccessoryViewPlugin;

pub mod view;

pub struct AccessoryPlugin;

impl Plugin for AccessoryPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Accessory>()
            .add_plugins(AccessoryViewPlugin);
    }
}

pub struct Wearer<'a> {
    pub size: &'a Vec2,
    pub anchor_points: &'a AnchorPointSet,
}

#[derive(Component, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Accessory {
    pub template: String,
    pub tint: Color,
    pub extra_spewers: Vec<Spewer>,
}

impl Accessory {
    pub fn new<T: ToString>(template: T) -> Self {
        Self {
            template: template.to_string(),
            extra_spewers: Vec::new(),
            tint: Color::WHITE,
        }
    }
}

impl Default for Accessory {
    fn default() -> Self {
        Self::new("pink_helmet")
    }
}

#[derive(Bundle, Default)]
pub struct AccessoryBundle {
    pub accessory: Accessory,
    pub save: Save,
}
