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
use crate::money::MoneyHungry;
use crate::name::{EntityName, HasNameTag, NameTag, NameTagBundle, SpeciesName};
use crate::sardip_save::SavedPet;
use crate::text_database::TextDatabase;
use crate::velocity::Speed;
use crate::{layering, GameState};

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
        app.add_event::<SpawnPetEvent>()
            .add_plugins(RonAssetPlugin::<AssetPetTemplateSet>::new(&["pets.ron"]))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                spawn_pending_pets.run_if(not(in_state(GameState::Loading))),
            )
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
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    template_handle: Res<PetTemplateSetHandle>,
    mut template_assets: ResMut<Assets<AssetPetTemplateSet>>,
) {
    if let Some(set) = template_assets.remove(template_handle.0.id()) {
        let mut db = PetTemplateDatabase {
            templates: set.templates,
        };

        db.populate_pre_calculated(&asset_server, &mut layouts);

        commands.insert_resource(db);
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PetTemplateImageSet {
    pub sprite_sheet: String,
    pub tile_size: (u32, u32),
    pub columns: u32,
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

    pub fn key(&self) -> &'static str {
        match self {
            TemplateSpeed::VerySlow => "global.very_slow",
            TemplateSpeed::Slow => "global.slow",
            TemplateSpeed::Medium => "global.medium",
            TemplateSpeed::Fast => "global.fast",
            TemplateSpeed::VeryFast => "global.very_fast",
            TemplateSpeed::BlueBlur => "global.blue_blur",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum TemplateStomachSize {
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

    pub fn key(&self) -> &'static str {
        match self {
            TemplateStomachSize::Tiny => "global.tiny",
            TemplateStomachSize::Small => "global.small",
            TemplateStomachSize::Medium => "global.medium",
            TemplateStomachSize::Large => "global.large",
            TemplateStomachSize::Huge => "global.huge",
            TemplateStomachSize::PaulEatingPizza => "global.paul_eating_pizza",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TemplateStomach {
    pub size: TemplateStomachSize,
    pub sensations: HashMap<FoodSensationType, FoodSensationRating>,
}

#[derive(Serialize, Deserialize)]
pub enum TemplatePoopInterval {
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

    pub fn key(&self) -> &'static str {
        match self {
            TemplatePoopInterval::VeryFrequent => "global.very_frequent",
            TemplatePoopInterval::Frequent => "global.frequent",
            TemplatePoopInterval::Regular => "global.regular",
            TemplatePoopInterval::Infrequent => "global.infrequent",
            TemplatePoopInterval::VeryInfrequent => "global.very_infrequent",
            TemplatePoopInterval::Constipated => "global.constipated",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TemplatePooper {
    pub interval: TemplatePoopInterval,
    #[serde(default)]
    texture: Option<String>,
}

impl TemplatePooper {
    fn pooper(&self) -> Pooper {
        let texture = match &self.texture {
            Some(texture) => texture,
            None => "textures/game/poop.png",
        };

        Pooper::new(self.interval.interval(), texture)
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

#[derive(Serialize, Deserialize)]
pub enum TemplateSize {
    X(u32),
    Y(u32),
    XY(u32, u32),
}

impl TemplateSize {
    pub fn vec2(&self, image_size: (u32, u32)) -> Vec2 {
        match self {
            TemplateSize::X(x) => {
                let x = *x as f32;
                let ratio = x / image_size.0 as f32;
                let y = image_size.1 as f32 * ratio;

                Vec2::new(x, y)
            }
            TemplateSize::Y(y) => {
                let y = *y as f32;
                let max = y.max(image_size.1 as f32);
                let min = y.min(image_size.1 as f32);
                let ratio = min / max;
                let x = image_size.0 as f32 * ratio;

                Vec2::new(x, y)
            }
            TemplateSize::XY(x, y) => Vec2::new(*x as f32, *y as f32),
        }
    }
}

#[derive(Default)]
pub struct PreCalculated {
    pub number: i32,
    pub layout: Handle<TextureAtlasLayout>,
    pub texture: Handle<Image>,
    pub custom_size: Vec2,
}

#[derive(Deserialize, TypePath)]
pub struct PetTemplate {
    pub species_name: String,
    pub kind: PetKind,
    #[serde(default = "default_possible_evolutions")]
    pub possible_evolutions: Vec<PossibleEvolution>,
    pub image_set: PetTemplateImageSet,
    pub size: TemplateSize,
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
    #[serde(skip)]
    pub pre_calculated: PreCalculated,
}

#[derive(Clone)]
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

    fn evolve(&self, commands: &mut Commands, global_rng: &mut GlobalRng, evolving: EvolvingPet) {
        // Create new delete old
        commands.entity(evolving.entity).despawn_recursive();

        self.create_entity_from_saved(
            commands,
            global_rng,
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

    fn create_entity(
        &self,
        commands: &mut Commands,
        global_rng: &mut GlobalRng,
        location: Vec2,
        name: EntityName,
    ) -> Entity {
        self.create_entity_from_saved(
            commands,
            global_rng,
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

    fn create_entity_from_saved(
        &self,
        commands: &mut Commands,
        global_rng: &mut GlobalRng,
        saved: &SavedPet,
    ) -> Entity {
        let mut transform = Transform::from_xyz(0.0, 0.0, layering::view_screen::PET);
        if let Some(location) = saved.location {
            transform.translation.x = location.x;
            transform.translation.y = location.y;
        }

        let custom_size = self.pre_calculated.custom_size;

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
                sprite: SpriteBundle {
                    transform,
                    sprite: Sprite {
                        custom_size: Some(custom_size.clone()),
                        ..default()
                    },
                    texture: self.pre_calculated.texture.clone(),
                    ..default()
                },
                atlas: TextureAtlas {
                    layout: self.pre_calculated.layout.clone(),
                    ..default()
                },
                speed: saved.speed.clone(),
                rng: RngComponent::from(global_rng),
                ..default()
            })
            .insert((
                Wonder,
                Clickable::new(
                    Vec2::new(-(custom_size.x / 2.), custom_size.x / 2.),
                    Vec2::new(-(custom_size.y / 2.), custom_size.y / 2.),
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

    fn populate_pre_calculated(
        &mut self,
        asset_server: &AssetServer,
        layouts: &mut Assets<TextureAtlasLayout>,
    ) {
        for (i, template) in self.templates.iter_mut().enumerate() {
            let layout = TextureAtlasLayout::from_grid(
                UVec2::new(
                    template.image_set.tile_size.0,
                    template.image_set.tile_size.1,
                ),
                template.image_set.columns,
                1,
                None,
                None,
            );
            let layout = layouts.add(layout);
            let texture = asset_server.load(&template.image_set.sprite_sheet);

            let custom_size = template.size.vec2(template.image_set.tile_size);

            template.pre_calculated = PreCalculated {
                layout,
                texture,
                number: i as i32 + 1,
                custom_size,
            };
        }
    }
}

#[derive(Deserialize, Asset, TypePath)]
pub struct AssetPetTemplateSet {
    pub templates: Vec<PetTemplate>,
}

#[derive(Debug, Resource)]
struct PetTemplateSetHandle(Handle<AssetPetTemplateSet>);

#[derive(Event)]
pub enum SpawnPetEvent {
    Blank((Vec2, String)),
    Saved(SavedPet),
    Evolve((String, EvolvingPet)),
}

impl SpawnPetEvent {
    fn species_name(&self) -> &str {
        match self {
            SpawnPetEvent::Blank((_, species_name)) => species_name,
            SpawnPetEvent::Saved(saved) => &saved.species_name.0,
            SpawnPetEvent::Evolve((species_name, _)) => species_name,
        }
    }
}

fn spawn_pending_pets(
    mut commands: Commands,
    mut events: EventReader<SpawnPetEvent>,
    mut global_rng: ResMut<GlobalRng>,
    pet_template_db: Res<PetTemplateDatabase>,
    text_db: Res<TextDatabase>,
) {
    for event in events.read() {
        info!("Spawning pet: {:?}", event.species_name());
        if let Some(template) = pet_template_db.get_by_name(event.species_name()) {
            match event {
                SpawnPetEvent::Blank((pos, _)) => {
                    template.create_entity(
                        &mut commands,
                        &mut global_rng,
                        pos.clone(),
                        EntityName::random(&text_db),
                    );
                }
                SpawnPetEvent::Saved(saved) => {
                    template.create_entity_from_saved(&mut commands, &mut global_rng, saved);
                }
                SpawnPetEvent::Evolve((_, evolving)) => {
                    template.evolve(&mut commands, &mut global_rng, evolving.clone());
                }
            }
        } else {
            error!(
                "Unable to find template for species {}",
                event.species_name()
            );
        }
    }
}
