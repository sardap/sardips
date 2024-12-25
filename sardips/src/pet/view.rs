use bevy::prelude::*;

use crate::{
    layering,
    name::{HasNameTag, NameTag, NameTagBundle, SpeciesName},
    simulation::Simulated,
    view::EntityView,
};

use sardips_core::{
    interaction::Clickable,
    mood_core::{AutoSetMoodImage, MoodImageIndexes},
};

use super::{template::PetTemplateDatabase, Pet};

pub struct PetViewPlugin;

impl Plugin for PetViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            spawn_pet_view.run_if(resource_exists::<PetTemplateDatabase>),
        );
    }
}

#[derive(Component, Default)]
pub struct PetView;

#[derive(Bundle)]
pub struct PetViewBundle {
    pub view: EntityView,
    pub pet_view: PetView,
    pub sprite: SpriteBundle,
    pub atlas: TextureAtlas,
    pub clickable: Clickable,
    pub image_set: MoodImageIndexes,
    pub auto_mood_image: AutoSetMoodImage,
    pub simulated: Simulated,
}

fn spawn_pet_view(
    mut commands: Commands,
    pet_db: Res<PetTemplateDatabase>,
    pet: Query<(Entity, &SpeciesName, &Transform), Added<Pet>>,
) {
    for (pet_entity, species_name, transform) in pet.iter() {
        if let Some(template) = pet_db.get_by_name(&species_name.0) {
            let custom_size = template.pre_calculated.custom_size;

            let entity_id = commands
                .spawn(PetViewBundle {
                    view: EntityView { entity: pet_entity },
                    pet_view: PetView,
                    sprite: SpriteBundle {
                        transform: Transform::from_xyz(
                            transform.translation.x,
                            transform.translation.y,
                            layering::view_screen::PET,
                        ),
                        sprite: Sprite {
                            custom_size: Some(custom_size),
                            ..default()
                        },
                        texture: template.pre_calculated.texture.clone(),
                        ..default()
                    },
                    atlas: TextureAtlas {
                        layout: template.pre_calculated.layout.clone(),
                        ..default()
                    },
                    clickable: Clickable::new(
                        Vec2::new(-(custom_size.x / 2.), custom_size.x / 2.),
                        Vec2::new(-(custom_size.y / 2.), custom_size.y / 2.),
                    ),
                    image_set: MoodImageIndexes::new(&template.image_set.column_mood_map),
                    auto_mood_image: AutoSetMoodImage,
                    simulated: Simulated,
                })
                .id();

            let name_tag_id = commands
                .spawn(NameTagBundle {
                    text: default(),
                    name_tag: NameTag::new().with_font_size(40.0),
                    ..default()
                })
                .set_parent(entity_id)
                .id();

            commands
                .entity(entity_id)
                .insert(HasNameTag::new(name_tag_id));
        } else {
            error!("No template found for pet species: {}", &species_name.0);
        }
    }
}
