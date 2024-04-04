use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_turborand::{GlobalRng, RngComponent};
use serde::{Deserialize, Serialize};

use crate::age::Age;
use crate::dynamic_dialogue::EntityFactDatabase;
use crate::food::preferences::{FoodPreference, FoodSensationRating};
use crate::food::FoodSensationType;
use crate::interaction::Clickable;
use crate::layering;
use crate::name::{EntityName, HasNameTag, NameTag, NameTagBundle, SpeciesName};
use crate::sardip_save::SavedPet;
use crate::velocity::Speed;

use super::evolve::PossibleEvolution;
use super::fun::Fun;
use super::hunger::Hunger;
use super::mood::{Mood, MoodCategory, MoodCategoryHistory, MoodImages};
use super::poop::{Cleanliness, Diarrhea, Pooper};
use super::PetKind;
use super::{wonder::Wonder, PetBundle};

pub struct PetTemplatePlugin;

impl Plugin for PetTemplatePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<AssetPetTemplateSet>::new(&["pets.ron"]))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                load_templates.run_if(not(resource_exists::<PetTemplateDatabase>)),
            );
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let template_set = PetTemplateSetHandle(asset_server.load("pets/complete.pets.ron"));
    commands.insert_resource(template_set);
}

fn load_templates(
    mut commands: Commands,
    template_handle: Res<PetTemplateSetHandle>,
    mut template_assets: ResMut<Assets<AssetPetTemplateSet>>,
) {
    if let Some(level) = template_assets.remove(template_handle.0.id()) {
        commands.insert_resource(PetTemplateDatabase {
            templates: level.templates,
        });
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PetTemplateImageSet {
    pub sprite_sheet: String,
    pub tile_size: (f32, f32),
    pub columns: usize,
    pub column_mood_map: HashMap<MoodCategory, u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Stomach {
    size: f32,
    sensations: HashMap<FoodSensationType, FoodSensationRating>,
}

fn default_speed() -> f32 {
    70.0
}

fn default_possible_evolutions() -> Vec<PossibleEvolution> {
    vec![]
}

#[derive(Deserialize, TypePath)]
pub struct PetTemplate {
    species_name: String,
    kind: PetKind,
    image_set: PetTemplateImageSet,
    size: (f32, f32),
    #[serde(default = "default_possible_evolutions")]
    pub possible_evolutions: Vec<PossibleEvolution>,
    #[serde(default = "default_speed")]
    speed: f32,
    stomach: Option<Stomach>,
    poop_interval: Option<f32>,
    has_cleanliness: bool,
    has_fun: bool,
}

pub struct EvolvingPet {
    pub entity: Entity,
    pub location: Vec2,
    pub name: EntityName,
    pub age: Age,
    pub mood_history: MoodCategoryHistory,
    pub fact_db: EntityFactDatabase,
}

impl PetTemplate {
    fn get_hunger(&self) -> Option<Hunger> {
        self.stomach
            .as_ref()
            .map(|stomach| Hunger::new(stomach.size))
    }

    fn get_pooper(&self) -> Option<Pooper> {
        self.poop_interval
            .map(|interval| Pooper::new(Duration::from_secs_f32(interval * 60.)))
    }

    fn get_cleanliness(&self) -> Option<Cleanliness> {
        if self.has_cleanliness {
            Some(Cleanliness)
        } else {
            None
        }
    }

    fn get_fun(&self) -> Option<Fun> {
        if self.has_fun {
            Some(Fun::default())
        } else {
            None
        }
    }

    pub fn evolve(
        &self,
        commands: &mut Commands,
        asset_server: &AssetServer,
        global_rng: &mut GlobalRng,
        text_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        evolving: EvolvingPet,
    ) {
        // Create new delete old
        commands.entity(evolving.entity).despawn_recursive();

        self.create_entity_from_saved(
            commands,
            asset_server,
            global_rng,
            text_atlas_layouts,
            &SavedPet {
                location: Some(evolving.location),
                species_name: SpeciesName::new(&self.species_name),
                speed: Speed(self.speed),
                kind: self.kind,
                fun: self.get_fun(),
                hunger: self.get_hunger(),
                pooper: self.get_pooper(),
                cleanliness: self.get_cleanliness(),
                // Copied from evolving
                name: evolving.name,
                age: evolving.age,
                mood_history: evolving.mood_history,
                fact_db: evolving.fact_db,
                ..default()
            },
        );
    }

    pub fn create_entity(
        &self,
        commands: &mut Commands,
        asset_server: &AssetServer,
        global_rng: &mut GlobalRng,
        text_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        name: EntityName,
    ) -> Entity {
        self.create_entity_from_saved(
            commands,
            asset_server,
            global_rng,
            text_atlas_layouts,
            &SavedPet {
                species_name: SpeciesName::new(&self.species_name),
                name,
                speed: Speed(self.speed),
                kind: self.kind,
                fun: self.get_fun(),
                hunger: self.get_hunger(),
                pooper: self.get_pooper(),
                cleanliness: self.get_cleanliness(),
                ..default()
            },
        )
    }

    pub fn create_entity_from_saved(
        &self,
        commands: &mut Commands,
        asset_server: &AssetServer,
        global_rng: &mut GlobalRng,
        text_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        saved: &SavedPet,
    ) -> Entity {
        let layout = TextureAtlasLayout::from_grid(
            Vec2::new(self.image_set.tile_size.0, self.image_set.tile_size.1),
            self.image_set.columns,
            1,
            None,
            None,
        );
        let layout = text_atlas_layouts.add(layout);

        let mut transform = Transform::from_xyz(0.0, 0.0, layering::view_screen::PET);
        if let Some(location) = saved.location {
            transform.translation.x = location.x;
            transform.translation.y = location.y;
        }

        let entity_id = commands
            .spawn(PetBundle {
                species_name: saved.species_name.clone(),
                name: saved.name.clone(),
                image_set: MoodImages::new(&self.image_set.column_mood_map),
                age: saved.age.clone(),
                mood: saved.mood.clone(),
                mood_category_history: saved.mood_history.clone(),
                fact_db: saved.fact_db.clone(),
                kind: saved.kind.clone(),
                sprite: SpriteSheetBundle {
                    transform,
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(self.size.0, self.size.1)),
                        ..default()
                    },
                    atlas: TextureAtlas {
                        layout: layout.clone(),
                        ..default()
                    },
                    texture: asset_server.load(&self.image_set.sprite_sheet),
                    ..default()
                },
                speed: saved.speed.clone(),
                rng: RngComponent::from(global_rng),
                ..default()
            })
            .insert((
                Wonder,
                Clickable::new(
                    Vec2::new(-(self.size.0 / 2.), self.size.0 / 2.),
                    Vec2::new(-(self.size.1 / 2.), self.size.1 / 2.),
                ),
            ))
            .id();

        if let Some(hunger) = &saved.hunger {
            commands.entity(entity_id).insert((
                hunger.clone(),
                FoodPreference {
                    sensation_ratings: self.stomach.as_ref().unwrap().sensations.clone(),
                },
            ));
        }

        if let Some(fun) = &saved.fun {
            commands.entity(entity_id).insert(fun.clone());
        }

        if let Some(pooper) = &saved.pooper {
            commands.entity(entity_id).insert(pooper.clone());
        }

        if let Some(cleanliness) = &saved.cleanliness {
            commands.entity(entity_id).insert(cleanliness.clone());
        }

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

        entity_id
    }
}

#[derive(Resource)]
pub struct PetTemplateDatabase {
    templates: Vec<PetTemplate>,
}

impl PetTemplateDatabase {
    pub fn get<T: ToString>(&self, species_name: T) -> Option<&PetTemplate> {
        let species_name = species_name.to_string();
        self.templates
            .iter()
            .find(|template| template.species_name == species_name)
    }

    pub fn iter(&self) -> impl Iterator<Item = &PetTemplate> {
        self.templates.iter()
    }
}

#[derive(Deserialize, Asset, TypePath)]
pub struct AssetPetTemplateSet {
    pub templates: Vec<PetTemplate>,
}

#[derive(Debug, Resource)]
struct PetTemplateSetHandle(Handle<AssetPetTemplateSet>);
