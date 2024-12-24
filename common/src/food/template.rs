use bevy::{prelude::*, utils::HashSet};
use shared_deps::bevy_common_assets::ron::RonAssetPlugin;
use shared_deps::serde::{Deserialize, Serialize};

use crate::{
    layering,
    name::{EntityName, SpeciesName},
    pet::template::TemplateSize,
};

use super::{FoodBundle, FoodFillFactor, FoodSensationType, FoodSensations};

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

#[derive(Serialize, Deserialize)]
pub struct FoodTemplate {
    pub name: String,
    pub sensations: HashSet<FoodSensationType>,
    pub texture: String,
    pub texture_size: (u32, u32),
    pub sprite_size: TemplateSize,
    pub fill_factor: f32,
}

impl FoodTemplate {
    pub fn spawn(&self, commands: &mut Commands, location: Vec2) -> Entity {
        let entity_id = commands
            .spawn(FoodBundle {
                sensations: FoodSensations {
                    values: self.sensations.clone(),
                },
                location: Transform::from_translation(Vec3::new(
                    location.x,
                    location.y,
                    layering::view_screen::FOOD,
                )),
                species_name: SpeciesName::new(&self.name),
                name: EntityName::new(format!("food.{}", self.name.to_lowercase())),
                fill_factor: FoodFillFactor(self.fill_factor),
                ..default()
            })
            .id();

        entity_id
    }
}

#[derive(Resource)]
pub struct FoodTemplateDatabase {
    pub templates: Vec<FoodTemplate>,
}

impl FoodTemplateDatabase {
    pub fn iter(&self) -> impl Iterator<Item = &FoodTemplate> {
        self.templates.iter()
    }

    pub fn get(&self, name: &str) -> Option<&FoodTemplate> {
        self.templates.iter().find(|template| template.name == name)
    }
}

#[derive(Asset, Serialize, Deserialize, TypePath)]
pub struct AssetFoodTemplateSet {
    pub templates: Vec<FoodTemplate>,
}

#[derive(Debug, Resource)]
struct FoodTemplateSetHandle(Handle<AssetFoodTemplateSet>);
