use std::collections::HashSet;

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use serde::{Deserialize, Serialize};

use crate::{
    interaction::Clickable,
    layering,
    name::{EntityName, HasNameTag, NameTag, NameTagBundle, SpeciesName},
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
    pub texture_size: (f32, f32),
    pub sprite_size: TemplateSize,
    pub fill_factor: f32,
}

impl FoodTemplate {
    pub fn spawn(
        &self,
        commands: &mut Commands,
        asset_server: &AssetServer,
        location: Vec2,
    ) -> Entity {
        let custom_size = self.sprite_size.vec2(self.texture_size);

        let entity_id = commands
            .spawn(FoodBundle {
                sensations: FoodSensations {
                    values: self.sensations.clone(),
                },
                species_name: SpeciesName::new(&self.name),
                name: EntityName::new(format!("food.{}", self.name.to_lowercase())),
                sprite: SpriteBundle {
                    transform: Transform::from_translation(Vec3::new(
                        location.x,
                        location.y,
                        layering::view_screen::FOOD,
                    )),
                    sprite: Sprite {
                        custom_size: Some(custom_size),
                        ..default()
                    },
                    texture: asset_server.load(&self.texture),
                    ..default()
                },
                fill_factor: FoodFillFactor(self.fill_factor),
                ..default()
            })
            .insert(Clickable::new(
                Vec2::new(-(custom_size.x / 2.), custom_size.x / 2.),
                Vec2::new(-(custom_size.y / 2.), custom_size.y / 2.),
            ))
            .id();

        let name_tag_id = commands
            .spawn(NameTagBundle {
                text: default(),
                name_tag: NameTag::new().with_font_size(30.),
                ..default()
            })
            .set_parent(entity_id)
            .id();

        commands
            .entity(entity_id)
            .insert(HasNameTag::new(name_tag_id));

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
