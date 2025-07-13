use std::{fmt, time::Duration};

use bevy::{prelude::*, utils::HashMap};
use fact_db::{Concept, Criteria, Criterion, EntityFactDatabase};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{
    accessory_core::AnchorPointSet,
    age_core::Age,
    breeding_core::Breeds,
    food_core::{FoodSensationRating, FoodSensationType},
    fun_core::Fun,
    hunger_core::Hunger,
    money_core::{Money, MoneyHungry},
    mood_core::{MoodCategory, MoodCategoryHistory},
    name::EntityName,
};

pub struct PetCorePlugin;

impl Plugin for PetCorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Pooper>()
            .register_type::<Cleanliness>()
            .register_type::<Diarrhea>();
    }
}

#[derive(
    Debug,
    Component,
    Default,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumIter,
)]
pub enum PetKind {
    #[default]
    Blob,
    Object,
    Creature,
    Supernatural,
}

impl fmt::Display for PetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PetKind::Object => write!(f, "Object"),
            PetKind::Blob => write!(f, "Blob"),
            PetKind::Creature => write!(f, "Creature"),
            PetKind::Supernatural => write!(f, "Supernatural"),
        }
    }
}

#[derive(Deserialize)]
pub struct PossibleEvolution {
    pub criteria: Vec<Criterion>,
    pub species: Vec<String>,
}

impl PossibleEvolution {
    pub fn criteria(&self) -> Criteria {
        Criteria::new(Concept::Evolve, &self.criteria)
    }
}

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Clone, Copy)]
pub enum WeightType {
    SuperHeavyWeight,
    HeavyWeight,
    #[default]
    MiddleWeight,
    LightWeight,
    FeatherWeight,
    FlyWeight,
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
    pub fn value(&self) -> f32 {
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

#[derive(Component, Default, Serialize, Deserialize, Clone, Reflect)]
#[reflect(Component)]
pub struct Cleanliness;

#[derive(Component, Default, Serialize, Deserialize, Clone, Reflect)]
#[reflect(Component)]
pub struct Diarrhea;

#[derive(Component, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Pooper {
    pub interval: Duration,
    pub poop_timer: Timer,
    pub texture: String,
}

impl Pooper {
    pub fn new(poop_interval: Duration, texture: impl ToString) -> Self {
        Self {
            interval: poop_interval,
            poop_timer: Timer::new(poop_interval, TimerMode::Repeating),
            texture: texture.to_string(),
        }
    }
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

#[derive(Default, Clone)]
pub struct PreCalculated {
    pub number: i32,
    pub layout: Handle<TextureAtlasLayout>,
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
    pub anchor_points: AnchorPointSet,
    pub weight: WeightType,
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
    pub fn get_hunger(&self) -> Option<Hunger> {
        self.stomach
            .as_ref()
            .map(|stomach| Hunger::new(stomach.size.value()))
    }

    pub fn get_pooper(&self) -> Option<Pooper> {
        self.pooper.as_ref().map(|template| template.pooper())
    }

    pub fn get_cleanliness(&self) -> Option<Cleanliness> {
        if self.cleanliness.is_some() {
            Some(Cleanliness)
        } else {
            None
        }
    }

    pub fn get_fun(&self) -> Option<Fun> {
        if self.fun.is_some() {
            Some(Fun::default())
        } else {
            None
        }
    }

    pub fn get_money_hungry(&self) -> Option<MoneyHungry> {
        self.money_hungry.as_ref().map(|money| MoneyHungry {
            previous_balance: 0,
            max_care: money.max_balance,
        })
    }

    pub fn get_breeds(&self) -> Option<Breeds> {
        if self.breeds {
            Some(Breeds::default())
        } else {
            None
        }
    }
}

#[derive(Resource, Default)]
pub struct PetTemplateDatabase {
    pub templates: Vec<PetTemplate>,
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

    pub fn populate_pre_calculated(&mut self, layouts: &mut Assets<TextureAtlasLayout>) {
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

            let custom_size = template.size.vec2(template.image_set.tile_size);

            template.pre_calculated = PreCalculated {
                layout,
                number: i as i32 + 1,
                custom_size,
            };
        }
    }
}
