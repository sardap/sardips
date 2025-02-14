use bevy::prelude::*;
use sardips_core::{
    food_core::FoodTemplateDatabase,
    interaction::Clickable,
    name::{HasNameTag, NameTag, NameTagBundle, SpeciesName},
    view::EntityView,
};

use crate::{layering, simulation::Simulated};

use super::Food;

#[derive(Debug, Component, Default)]
pub struct FoodView;

#[derive(Bundle)]
pub struct FoodViewBundle {
    pub view: EntityView,
    pub food_view: FoodView,
    pub sprite: SpriteBundle,
    pub clickable: Clickable,
    pub simulated: Simulated,
}

pub fn spawn_food_view(
    mut commands: Commands,
    template_db: Res<FoodTemplateDatabase>,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &Transform, &SpeciesName), Added<Food>>,
) {
    for (food, location, name) in query.iter() {
        let entry = template_db.get(&name.0).unwrap();

        let custom_size = entry.sprite_size.vec2(entry.texture_size);

        let entity_id = commands
            .spawn(FoodViewBundle {
                view: EntityView { entity: food },
                food_view: FoodView,
                sprite: SpriteBundle {
                    transform: Transform::from_translation(Vec3::new(
                        location.translation.x,
                        location.translation.y,
                        layering::view_screen::FOOD,
                    )),
                    sprite: Sprite {
                        custom_size: Some(custom_size),
                        ..default()
                    },
                    texture: asset_server.load(&entry.texture),
                    ..default()
                },
                clickable: Clickable::new(
                    Vec2::new(-(custom_size.x / 2.), custom_size.x / 2.),
                    Vec2::new(-(custom_size.y / 2.), custom_size.y / 2.),
                ),
                simulated: Simulated,
            })
            .id();

        let name_tag_id = commands
            .spawn(NameTagBundle {
                name_tag: NameTag::new().with_font_size(30.),
                ..default()
            })
            .set_parent(entity_id)
            .id();

        commands
            .entity(entity_id)
            .insert(HasNameTag::new(name_tag_id));
    }
}
