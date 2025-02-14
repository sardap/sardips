use bevy::prelude::*;
use sardips_core::food_core::{FoodFillFactor, FoodSensations, FoodTemplate, FoodTemplateDatabase};
use sardips_core::name::{EntityName, SpeciesName};
use serde::{Deserialize, Serialize};
use shared_deps::bevy_common_assets::ron::RonAssetPlugin;

use crate::layering;

use super::FoodBundle;

pub struct FoodTemplatePlugin;

impl Plugin for FoodTemplatePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<AssetFoodTemplateSet>::new(&["foods.ron"]))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                load_templates.run_if(not(resource_exists::<FoodTemplateDatabase>)),
            );
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let template_set = FoodTemplateSetHandle(asset_server.load("foods/complete.foods.ron"));
    commands.insert_resource(template_set);
}

fn load_templates(
    mut commands: Commands,
    template_handle: Res<FoodTemplateSetHandle>,
    mut template_assets: ResMut<Assets<AssetFoodTemplateSet>>,
) {
    if let Some(templates) = template_assets.remove(template_handle.0.id()) {
        commands.insert_resource(FoodTemplateDatabase {
            templates: templates.templates,
        });
    }
}

pub fn spawn_food(template: &FoodTemplate, commands: &mut Commands, location: Vec2) -> Entity {
    let entity_id = commands
        .spawn(FoodBundle {
            sensations: FoodSensations {
                values: template.sensations.clone(),
            },
            location: Transform::from_translation(Vec3::new(
                location.x,
                location.y,
                layering::view_screen::FOOD,
            )),
            species_name: SpeciesName::new(&template.name),
            name: EntityName::new(format!("food.{}", template.name.to_lowercase())),
            fill_factor: FoodFillFactor(template.fill_factor),
            ..default()
        })
        .id();

    entity_id
}

#[derive(Asset, Serialize, Deserialize, TypePath)]
pub struct AssetFoodTemplateSet {
    pub templates: Vec<FoodTemplate>,
}

#[derive(Debug, Resource)]
struct FoodTemplateSetHandle(Handle<AssetFoodTemplateSet>);
