use std::time::Duration;

use bevy::prelude::*;
use bevy::utils::HashMap;
use fact_db::EntityFactDatabase;
use shared_deps::bevy_common_assets::ron::RonAssetPlugin;
use shared_deps::serde::{Deserialize, Serialize};

use crate::age::Age;
use crate::food::preferences::{FoodPreference, FoodSensationRating};
use crate::food::FoodSensationType;
use crate::layering;
use crate::money::{Money, MoneyHungry};
use crate::name::{EntityName, SpeciesName};
use sardips_core::{
    mood_core::MoodCategory, text_database::TextDatabase, velocity::Speed, GameState,
};

use super::breeding::Breeds;
use super::evolve::PossibleEvolution;
use super::fun::Fun;
use super::hunger::Hunger;
use super::mood::MoodCategoryHistory;
use super::poop::{Cleanliness, Pooper};
use super::PetBundle;
use super::PetKind;

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

pub const DEFAULT_POOP_TEXTURE: &str = "textures/game/poop.png";

impl TemplatePooper {
    fn pooper(&self) -> Pooper {
        let texture = match &self.texture {
            Some(texture) => texture,
            None => DEFAULT_POOP_TEXTURE,
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
    max_balance: Money,
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
        self.money_hungry.as_ref().map(|money| MoneyHungry {
            previous_balance: 0,
            max_care: money.max_balance,
        })
    }

    fn get_breeds(&self) -> Option<Breeds> {
        if self.breeds {
            Some(Breeds::default())
        } else {
            None
        }
    }

    fn evolve(&self, commands: &mut Commands, evolving: EvolvingPet) {
        // Create new delete old

        commands.entity(evolving.entity).despawn_recursive();

        let new_entity = self.spawn(commands, evolving.location, evolving.name);

        commands.entity(new_entity).insert(evolving.age);
        commands.entity(new_entity).insert(evolving.mood_history);
        commands.entity(new_entity).insert(evolving.fact_db);
    }

    fn spawn(&self, commands: &mut Commands, location: Vec2, name: EntityName) -> Entity {
        let entity_id = commands
            .spawn(PetBundle {
                species_name: SpeciesName::new(&self.species_name),
                name,
                mood_category_history: MoodCategoryHistory::default(),
                fact_db: EntityFactDatabase::default(),
                kind: self.kind,
                speed: Speed(self.speed.value()),
                transform: Transform::from_xyz(location.x, location.y, layering::view_screen::PET),
                ..default()
            })
            .id();

        if let Some(breeds) = self.get_breeds() {
            commands.entity(entity_id).insert(breeds);
        }

        if let Some(hunger) = self.get_hunger() {
            commands.entity(entity_id).insert((
                hunger,
                FoodPreference {
                    sensation_ratings: self.stomach.as_ref().unwrap().sensations.clone(),
                },
            ));
        }

        if let Some(fun) = self.get_fun() {
            commands.entity(entity_id).insert(fun);
        }

        if let Some(pooper) = self.get_pooper() {
            commands.entity(entity_id).insert(pooper);
        }

        if let Some(cleanliness) = self.get_cleanliness() {
            commands.entity(entity_id).insert(cleanliness);
        }

        if let Some(money_hungry) = self.get_money_hungry() {
            commands.entity(entity_id).insert(money_hungry);
        }

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
    Evolve((String, EvolvingPet)),
}

impl SpawnPetEvent {
    fn species_name(&self) -> &str {
        match self {
            SpawnPetEvent::Blank((_, species_name)) => species_name,
            SpawnPetEvent::Evolve((species_name, _)) => species_name,
        }
    }
}

fn spawn_pending_pets(
    mut commands: Commands,
    mut events: EventReader<SpawnPetEvent>,
    pet_template_db: Res<PetTemplateDatabase>,
    text_db: Res<TextDatabase>,
) {
    for event in events.read() {
        info!("Spawning pet: {:?}", event.species_name());
        if let Some(template) = pet_template_db.get_by_name(event.species_name()) {
            match event {
                SpawnPetEvent::Blank((pos, _)) => {
                    template.spawn(&mut commands, *pos, EntityName::random(&text_db));
                }
                SpawnPetEvent::Evolve((_, evolving)) => {
                    template.evolve(&mut commands, evolving.clone());
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
