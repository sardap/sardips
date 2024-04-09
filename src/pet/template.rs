use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_turborand::{GlobalRng, RngComponent};
use serde::{Deserialize, Serialize};

use crate::age::Age;
use crate::facts::EntityFactDatabase;
use crate::food::preferences::{FoodPreference, FoodSensationRating};
use crate::food::FoodSensationType;
use crate::interaction::Clickable;
use crate::layering;
use crate::money::MoneyHungry;
use crate::name::{EntityName, HasNameTag, NameTag, NameTagBundle, SpeciesName};
use crate::sardip_save::SavedPet;
use crate::velocity::Speed;

use super::breeding::Breeds;
use super::evolve::PossibleEvolution;
use super::fun::Fun;
use super::hunger::Hunger;
use super::mood::{MoodCategory, MoodCategoryHistory, MoodImages};
use super::poop::{Cleanliness, Pooper};
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
    if let Some(set) = template_assets.remove(template_handle.0.id()) {
        commands.insert_resource(PetTemplateDatabase {
            templates: set.templates,
        });
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PetTemplateImageSet {
    pub sprite_sheet: String,
    pub tile_size: (f32, f32),
    pub columns: usize,
    pub column_mood_map: HashMap<MoodCategory, u32>,
}

fn default_speed() -> TemplateSpeed {
    TemplateSpeed::Medium
}

fn default_possible_evolutions() -> Vec<PossibleEvolution> {
    vec![]
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum TemplateSpeed {
    VerySlow,
    Slow,
    Medium,
    Fast,
    VeryFast,
    BlueBlur,
}

impl TemplateSpeed {
    fn value(&self) -> f32 {
        match self {
            TemplateSpeed::VerySlow => 20.0,
            TemplateSpeed::Slow => 35.0,
            TemplateSpeed::Medium => 50.0,
            TemplateSpeed::Fast => 70.0,
            TemplateSpeed::VeryFast => 90.0,
            TemplateSpeed::BlueBlur => 130.0,
        }
    }
}

#[derive(Serialize, Deserialize)]
enum TemplateStomachSize {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    PaulEatingPizza,
}

impl TemplateStomachSize {
    fn value(&self) -> f32 {
        match self {
            TemplateStomachSize::Tiny => 50.0,
            TemplateStomachSize::Small => 80.0,
            TemplateStomachSize::Medium => 100.0,
            TemplateStomachSize::Large => 120.0,
            TemplateStomachSize::Huge => 150.0,
            TemplateStomachSize::PaulEatingPizza => 300.0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TemplateStomach {
    size: TemplateStomachSize,
    sensations: HashMap<FoodSensationType, FoodSensationRating>,
}

#[derive(Serialize, Deserialize)]
enum TemplatePoopInterval {
    VeryFrequent,
    Frequent,
    Regular,
    Infrequent,
    VeryInfrequent,
    Constipated,
}

impl TemplatePoopInterval {
    fn interval(&self) -> Duration {
        match self {
            TemplatePoopInterval::VeryFrequent => Duration::from_secs(10 * 60),
            TemplatePoopInterval::Frequent => Duration::from_secs(25 * 60),
            TemplatePoopInterval::Regular => Duration::from_secs(60 * 60),
            TemplatePoopInterval::Infrequent => Duration::from_secs(80 * 60),
            TemplatePoopInterval::VeryInfrequent => Duration::from_secs(120 * 60),
            TemplatePoopInterval::Constipated => Duration::from_secs(24 * 60 * 60),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TemplatePooper {
    interval: TemplatePoopInterval,
}

impl TemplatePooper {
    fn pooper(&self) -> Pooper {
        Pooper::new(self.interval.interval())
    }
}

#[derive(Serialize, Deserialize)]
pub struct TemplateFun {}

#[derive(Serialize, Deserialize)]
pub struct TemplateCleanliness {}

#[derive(Serialize, Deserialize)]
pub struct TemplateMoneyHungry {
    max_balance: i32,
}

fn default_breeds() -> bool {
    true
}

#[derive(Deserialize, TypePath)]
pub struct PetTemplate {
    pub species_name: String,
    pub kind: PetKind,
    #[serde(default = "default_possible_evolutions")]
    pub possible_evolutions: Vec<PossibleEvolution>,
    pub image_set: PetTemplateImageSet,
    pub size: (f32, f32),
    #[serde(default)]
    pub starter: bool,
    #[serde(default = "default_speed")]
    pub speed: TemplateSpeed,
    #[serde(default = "default_breeds")]
    pub breeds: bool,
    #[serde(default)]
    pub stomach: Option<TemplateStomach>,
    #[serde(default)]
    pub pooper: Option<TemplatePooper>,
    #[serde(default)]
    pub cleanliness: Option<TemplateCleanliness>,
    #[serde(default)]
    pub fun: Option<TemplateFun>,
    #[serde(default)]
    pub money_hungry: Option<TemplateMoneyHungry>,
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
            .map(|stomach| Hunger::new(stomach.size.value()))
    }

    fn get_pooper(&self) -> Option<Pooper> {
        self.pooper.as_ref().map(|template| template.pooper())
    }

    fn get_cleanliness(&self) -> Option<Cleanliness> {
        if self.cleanliness.is_some() {
            Some(Cleanliness)
        } else {
            None
        }
    }

    fn get_fun(&self) -> Option<Fun> {
        if self.fun.is_some() {
            Some(Fun::default())
        } else {
            None
        }
    }

    fn get_money_hungry(&self) -> Option<MoneyHungry> {
        if let Some(money) = &self.money_hungry {
            Some(MoneyHungry {
                previous_balance: 0,
                max_care: money.max_balance,
            })
        } else {
            None
        }
    }

    fn get_breeds(&self) -> Option<Breeds> {
        if self.breeds {
            Some(Breeds::default())
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
                speed: Speed(self.speed.value()),
                kind: self.kind,
                breeds: self.get_breeds(),
                fun: self.get_fun(),
                hunger: self.get_hunger(),
                pooper: self.get_pooper(),
                cleanliness: self.get_cleanliness(),
                money_hungry: self.get_money_hungry(),
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
        location: Vec2,
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
                speed: Speed(self.speed.value()),
                kind: self.kind,
                location: Some(location),
                breeds: self.get_breeds(),
                fun: self.get_fun(),
                hunger: self.get_hunger(),
                pooper: self.get_pooper(),
                cleanliness: self.get_cleanliness(),
                money_hungry: self.get_money_hungry(),
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

        if let Some(breeds) = &saved.breeds {
            commands.entity(entity_id).insert(breeds.clone());
        }

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

#[derive(Resource, Default)]
pub struct PetTemplateDatabase {
    templates: Vec<PetTemplate>,
}

impl PetTemplateDatabase {
    pub fn get_by_name<T: ToString>(&self, species_name: T) -> Option<&PetTemplate> {
        let species_name = species_name.to_string();
        self.templates
            .iter()
            .find(|template| template.species_name == species_name)
    }

    pub fn get_by_kind(&self, kind: PetKind) -> Vec<&PetTemplate> {
        self.templates
            .iter()
            .filter(|template| template.kind == kind)
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &PetTemplate> {
        self.templates.iter()
    }

    pub fn add(&mut self, template: PetTemplate) {
        self.templates.push(template);
    }
}

#[derive(Deserialize, Asset, TypePath)]
pub struct AssetPetTemplateSet {
    pub templates: Vec<PetTemplate>,
}

#[derive(Debug, Resource)]
struct PetTemplateSetHandle(Handle<AssetPetTemplateSet>);
