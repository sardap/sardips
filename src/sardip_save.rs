use std::path::Path;

use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng, RngComponent};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    age::Age,
    assets::GameImageAssets,
    dynamic_dialogue::FactDb,
    facts::{EntityFactDatabase, GlobalFactDatabase},
    food::{template::FoodTemplateDatabase, Food},
    money::{MoneyHungry, Wallet},
    name::{EntityName, SpeciesName},
    pet::{
        breeding::Breeds,
        fun::Fun,
        hunger::Hunger,
        mood::{Mood, MoodCategoryHistory},
        poop::{poop_scale, spawn_poop, Cleanliness, Poop, Pooper},
        template::PetTemplateDatabase,
        Pet, PetKind,
    },
    player::Player,
    simulation::{SimTime, SimTimeTrait, SimulationState},
    velocity::Speed,
    VERSION,
};

pub struct SardipSavePlugin;

impl Plugin for SardipSavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(SardipLoadingState::default())
            .add_event::<TriggerSave>()
            // Systems
            .add_systems(
                Update,
                (
                    trigger_save_looping.run_if(in_state(SimulationState::Running)),
                    save_game,
                ),
            )
            .add_systems(OnEnter(SardipLoadingState::Loading), load_game);
    }
}

#[derive(Debug, States, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SardipLoadingState {
    #[default]
    None,
    Loading,
    Loaded,
    Failed,
}

#[derive(Serialize, Deserialize)]
struct SaveGame {
    version: String,
    time_saved: DateTime<Utc>,
    player_save: PlayerSave,
    global_facts: FactDb,
    pets: Vec<SavedPet>,
    poops: Vec<SavedPoop>,
    foods: Vec<SavedFood>,
}

#[derive(Serialize, Deserialize)]
struct PlayerSave {
    wallet: Wallet,
}

#[derive(Serialize, Deserialize)]
pub struct SavedFood {
    pub position: Vec2,
    pub name: SpeciesName,
    pub persistent_id: PersistentId,
}

#[derive(Serialize, Deserialize)]
pub struct SavedPoop {
    pub position: Vec2,
}

#[derive(Serialize, Deserialize, Default)]
pub struct SavedPet {
    pub location: Option<Vec2>,
    pub species_name: SpeciesName,
    pub name: EntityName,
    pub speed: Speed,
    pub fact_db: EntityFactDatabase,
    pub kind: PetKind,
    pub age: Age,
    pub breeds: Option<Breeds>,
    pub mood: Mood,
    pub mood_history: MoodCategoryHistory,
    pub fun: Option<Fun>,
    pub hunger: Option<Hunger>,
    pub pooper: Option<Pooper>,
    pub cleanliness: Option<Cleanliness>,
    pub money_hungry: Option<MoneyHungry>,
}

struct SaveTimer {
    timer: Timer,
}

impl Default for SaveTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(5., TimerMode::Repeating),
        }
    }
}

fn trigger_save_looping(
    time: Res<Time>,
    mut timer: Local<SaveTimer>,
    mut save: EventWriter<TriggerSave>,
) {
    if timer.timer.tick(time.delta()).just_finished() {
        save.send(TriggerSave);
    }
}

fn save_game(
    mut should_save: EventReader<TriggerSave>,
    sim_time: Res<Time<SimTime>>,
    global_facts: Res<GlobalFactDatabase>,
    pet: Query<
        (
            &SpeciesName,
            &EntityName,
            &Speed,
            &EntityFactDatabase,
            &PetKind,
            &Age,
            Option<&Breeds>,
            &Mood,
            &MoodCategoryHistory,
            Option<&Fun>,
            Option<&Hunger>,
            Option<&Pooper>,
            Option<&Cleanliness>,
            Option<&MoneyHungry>,
        ),
        With<Pet>,
    >,
    poops: Query<&Transform, With<Poop>>,
    foods: Query<(&PersistentId, &Transform, &SpeciesName), With<Food>>,
    player: Query<&Wallet, With<Player>>,
) {
    if should_save.is_empty() {
        return;
    }
    should_save.clear();

    debug!("Saving game");

    let player_wallet = player.single();

    let mut save_game = SaveGame {
        version: VERSION.to_string(),
        time_saved: sim_time.last_run(),
        player_save: PlayerSave {
            wallet: player_wallet.clone(),
        },
        global_facts: global_facts.0.clone(),
        pets: Vec::new(),
        poops: Vec::new(),
        foods: Vec::new(),
    };

    for (
        species_name,
        name,
        speed,
        fact_db,
        kind,
        age,
        breeds,
        mood,
        mood_category_history,
        fun,
        hunger,
        pooper,
        cleanliness,
        money_hungry,
    ) in pet.iter()
    {
        save_game.pets.push(SavedPet {
            location: None,
            species_name: species_name.clone(),
            name: name.clone(),
            speed: speed.clone(),
            fact_db: fact_db.clone(),
            kind: kind.clone(),
            age: age.clone(),
            breeds: breeds.cloned(),
            mood: mood.clone(),
            mood_history: mood_category_history.clone(),
            fun: fun.cloned(),
            hunger: hunger.cloned(),
            pooper: pooper.cloned(),
            cleanliness: cleanliness.cloned(),
            money_hungry: money_hungry.cloned(),
        });
    }

    for transform in poops.iter() {
        save_game.poops.push(SavedPoop {
            position: transform.translation.xy(),
        });
    }

    for (id, transform, name) in foods.iter() {
        save_game.foods.push(SavedFood {
            persistent_id: id.clone(),
            position: transform.translation.xy(),
            name: name.clone(),
        });
    }

    let save_path = Path::new(SAVE_PATH);
    let save_file = std::fs::File::create(save_path).unwrap();
    #[cfg(feature = "dev")]
    ron::ser::to_writer_pretty(save_file, &save_game, ron::ser::PrettyConfig::default())
        .expect("Failed to save game");

    #[cfg(not(feature = "dev"))]
    bincode::serialize_into(save_file, &save_game).unwrap();
}

#[derive(Deserialize)]
struct SaveGameVersionOnly {
    version: String,
}

pub fn save_compatibility(a: &str, b: &str) -> bool {
    let a = a.split('.').collect::<Vec<_>>();
    let b = b.split('.').collect::<Vec<_>>();

    // check majors are the same
    if a[0] != b[0] {
        return false;
    }

    // Check minor versions are the same
    if a.len() > 1 && b.len() > 1 && a[1] != b[1] {
        return false;
    }

    true
}

fn load_game(
    mut commands: Commands,
    mut sim_time: ResMut<Time<SimTime>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut rng: ResMut<GlobalRng>,
    mut state: ResMut<NextState<SardipLoadingState>>,
    mut global_fact_db: ResMut<GlobalFactDatabase>,
    asset_server: Res<AssetServer>,
    pet_template_db: Res<PetTemplateDatabase>,
    food_template_db: Res<FoodTemplateDatabase>,
    game_image_assets: Res<GameImageAssets>,
    player: Query<Entity, With<Player>>,
) {
    debug!("Loading game");
    let load_path = Path::new(SAVE_PATH);
    let save_file = match std::fs::read(load_path) {
        Ok(string) => string,
        Err(_) => {
            error!("Failed to open save file");
            state.set(SardipLoadingState::Failed);
            return;
        }
    };

    // Check minor version is the same
    #[cfg(feature = "dev")]
    let version: SaveGame = match ron::de::from_bytes(&save_file) {
        Ok(version) => version,
        Err(_) => {
            error!("Failed to deserialize save file version");
            state.set(SardipLoadingState::Failed);
            return;
        }
    };

    #[cfg(not(feature = "dev"))]
    let version: SaveGameVersionOnly = match bincode::deserialize(&save_file) {
        Ok(version) => version,
        Err(_) => {
            error!("Failed to deserialize save file version");
            state.set(SardipLoadingState::Failed);
            return;
        }
    };

    if save_compatibility(&version.version, VERSION) == false {
        error!(
            "Save file version mismatch: {} != {}",
            version.version, VERSION
        );
        state.set(SardipLoadingState::Failed);
        return;
    }

    #[cfg(feature = "dev")]
    let save_game: SaveGame = match ron::de::from_bytes(&save_file) {
        Ok(save_game) => save_game,
        Err(err) => {
            error!("Failed to deserialize save file {}", err);
            state.set(SardipLoadingState::Failed);
            return;
        }
    };

    #[cfg(not(feature = "dev"))]
    let save_game: SaveGame = match bincode::deserialize(&save_file) {
        Ok(save_game) => save_game,
        Err(_) => {
            error!("Failed to deserialize save file");
            state.set(SardipLoadingState::Failed);
            return;
        }
    };

    sim_time.set_last_run(save_game.time_saved);

    global_fact_db.0 = save_game.global_facts;

    // Load player
    let player_entity = player.single();
    commands
        .entity(player_entity)
        .insert(save_game.player_save.wallet.clone());

    // Load pets
    for saved_pet in save_game.pets {
        let template = pet_template_db
            .get_by_name(&saved_pet.species_name.0)
            .expect("Failed to find pet template");

        template.create_entity_from_saved(
            &mut commands,
            &asset_server,
            &mut rng,
            &mut layouts,
            &saved_pet,
        );
    }

    // Load Foods
    for saved_food in save_game.foods {
        //Get Food template from name
        let template = food_template_db
            .get(&saved_food.name.0)
            .expect("Failed to find food template");

        let entity = template.spawn(&mut commands, &asset_server, saved_food.position);
        commands.entity(entity).insert(saved_food.persistent_id);
    }

    let mut rng = RngComponent::from(&mut rng);
    let mut rng = rng.fork();

    // Load Poops
    for saved_poop in save_game.poops {
        spawn_poop(
            &mut commands,
            &game_image_assets,
            poop_scale(&mut rng),
            saved_poop.position,
        )
    }

    debug!("Game loaded");
    state.set(SardipLoadingState::Loaded);
}

const SAVE_PATH: &str = "sardip_save.sav";

#[derive(Event)]
pub struct TriggerSave;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct PersistentId {
    uuid: uuid::Uuid,
}

impl Default for PersistentId {
    fn default() -> Self {
        Self {
            uuid: uuid::Uuid::new_v4(),
        }
    }
}
